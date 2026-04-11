//! PTY session lifecycle plus per-session [`OutputBuffer`] ownership.
//!
//! The bridge owns a map from `session_id` → [`TerminalSession`]. Each
//! session holds:
//!   * A locked output buffer (in-memory ring + disk spill).
//!   * Optional PTY handles (`Master` writer + child) when running.
//!   * Config (`buffer_max_lines`, `disk_cap_mib`).
//!
//! Incoming bytes from the PTY are chunked into lines and pushed through the
//! buffer; the reader thread coalesces lines into ~16 ms batches before
//! forwarding to a channel so the Tauri layer can emit `pty_output` events
//! in one IPC payload per tick (FR-081).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use chrono::Utc;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use uuid::Uuid;

use crate::error::{SpecLensError, SpecLensResult};
use crate::models::{HighlightMatch, OutputLine, OutputStream, PtySessionDescriptor};
use crate::services::highlight_rules::{default_compiled, first_match, CompiledRule};
use crate::services::output_buffer::{
    session_spill_path, OutputBuffer, MIN_BUFFER_LINES, MIN_DISK_CAP_MIB,
};

/// Minimal handle kept per session. The concrete `portable_pty` types are
/// stored behind `Option<Box<dyn ...>>` so tests can construct sessions
/// without spawning a real PTY (see `#[cfg(test)]` helpers below).
pub struct TerminalSession {
    pub descriptor: PtySessionDescriptor,
    pub buffer: Arc<Mutex<OutputBuffer>>,
    pub config: TerminalConfig,
    #[allow(clippy::type_complexity)]
    writer: Option<Mutex<Box<dyn Write + Send>>>,
    master: Option<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalConfig {
    pub buffer_max_lines: usize,
    pub disk_cap_mib: u64,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            buffer_max_lines: MIN_BUFFER_LINES * 5, // 5_000 lines
            disk_cap_mib: MIN_DISK_CAP_MIB * 4,     // 64 MiB
        }
    }
}

/// Event emitted from the reader thread to the Tauri layer. The layer
/// re-emits it as `pty_output` etc.
#[derive(Debug, Clone)]
pub enum BridgeEvent {
    Lines {
        session_id: Uuid,
        lines: Vec<OutputLine>,
    },
    Spilled {
        session_id: Uuid,
        evicted_count: u64,
    },
    Closed {
        session_id: Uuid,
    },
}

/// Global registry of live terminal sessions.
pub struct TerminalBridge {
    sessions: Mutex<HashMap<Uuid, Arc<TerminalSession>>>,
    logs_root: PathBuf,
    rules: &'static [CompiledRule],
}

impl TerminalBridge {
    pub fn new(logs_root: PathBuf) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            logs_root,
            rules: default_compiled(),
        }
    }

    pub fn session_count(&self) -> usize {
        self.sessions.lock().map(|m| m.len()).unwrap_or(0)
    }

    pub fn get(&self, session_id: &Uuid) -> SpecLensResult<Arc<TerminalSession>> {
        let map = self
            .sessions
            .lock()
            .map_err(|_| SpecLensError::Internal("terminal sessions mutex poisoned".into()))?;
        map.get(session_id)
            .cloned()
            .ok_or_else(|| SpecLensError::PtySessionNotFound(session_id.to_string()))
    }

    /// Insert a pre-built session. Used by `attach()` and by tests.
    pub fn insert(&self, session: TerminalSession) -> SpecLensResult<Arc<TerminalSession>> {
        let arc = Arc::new(session);
        let mut map = self
            .sessions
            .lock()
            .map_err(|_| SpecLensError::Internal("terminal sessions mutex poisoned".into()))?;
        map.insert(arc.descriptor.session_id, arc.clone());
        Ok(arc)
    }

    pub fn remove(&self, session_id: &Uuid) -> SpecLensResult<()> {
        let mut map = self
            .sessions
            .lock()
            .map_err(|_| SpecLensError::Internal("terminal sessions mutex poisoned".into()))?;
        map.remove(session_id);
        Ok(())
    }

    /// Append a plain text chunk (newline delimited) to the session buffer
    /// as if it arrived from the PTY. Used by tests and by any future
    /// in-process producers (e.g. pre-recorded replays).
    pub fn ingest_chunk(
        &self,
        session_id: &Uuid,
        chunk: &str,
        stream: OutputStream,
    ) -> SpecLensResult<Vec<OutputLine>> {
        let session = self.get(session_id)?;
        let mut pushed = Vec::new();
        let mut buffer = session
            .buffer
            .lock()
            .map_err(|_| SpecLensError::Internal("output buffer mutex poisoned".into()))?;
        for line_text in chunk.split_inclusive('\n') {
            let trimmed = line_text.trim_end_matches('\n').to_string();
            let highlight: Option<HighlightMatch> = first_match(&trimmed, self.rules);
            let line = OutputLine {
                seq: 0,
                ts: now_millis(),
                stream,
                text: trimmed,
                ansi_spans: None,
                highlight,
                step_id: None,
            };
            let (seq, _) = buffer.push(line.clone())?;
            let mut stored = line;
            stored.seq = seq;
            pushed.push(stored);
        }
        Ok(pushed)
    }

    /// Spawn a real PTY running the user's shell, and kick off a reader
    /// thread that batches incoming lines and forwards `BridgeEvent` values
    /// on `event_sink`. Returns the session descriptor so the caller can
    /// echo it back to the frontend.
    pub fn attach(
        &self,
        project_id: Uuid,
        window_id: String,
        cwd: PathBuf,
        shell: Option<String>,
        config: TerminalConfig,
        event_sink: std::sync::mpsc::Sender<BridgeEvent>,
    ) -> SpecLensResult<Arc<TerminalSession>> {
        let session_id = Uuid::new_v4();
        let shell = shell.unwrap_or_else(default_shell);
        let pty_system = NativePtySystem::default();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| SpecLensError::PtySpawnFailed(e.to_string()))?;

        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(&cwd);
        // Set TERM so colorised output behaves sanely. Do not inherit arbitrary
        // env vars; `portable-pty` already populates the minimum required set.
        cmd.env("TERM", "xterm-256color");

        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| SpecLensError::PtySpawnFailed(e.to_string()))?;
        // Drop the slave side so closing the master tears the child down.
        drop(pair.slave);

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| SpecLensError::PtySpawnFailed(e.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| SpecLensError::PtySpawnFailed(e.to_string()))?;

        let descriptor = PtySessionDescriptor {
            session_id,
            window_id,
            project_id,
            cwd: cwd.display().to_string(),
            shell,
            started_at: Utc::now(),
        };

        let spill = session_spill_path(&self.logs_root, &session_id.to_string());
        let buffer = Arc::new(Mutex::new(OutputBuffer::new(
            config.buffer_max_lines,
            config.disk_cap_mib,
            spill,
        )));

        let session = TerminalSession {
            descriptor: descriptor.clone(),
            buffer: buffer.clone(),
            config,
            writer: Some(Mutex::new(writer)),
            master: Some(Mutex::new(pair.master)),
        };
        let arc = self.insert(session)?;

        // Launch reader thread. It batches lines and forwards them every
        // ~16 ms per FR-081.
        let rules = self.rules;
        let buffer_clone = buffer;
        let sink = event_sink;
        thread::spawn(move || {
            reader_loop(session_id, reader, buffer_clone, rules, sink);
        });

        Ok(arc)
    }
}

impl TerminalSession {
    /// Write bytes to the PTY master. No-op (and returns an error) when the
    /// session was constructed without a live PTY (tests).
    pub fn write(&self, bytes: &[u8]) -> SpecLensResult<()> {
        let writer = self
            .writer
            .as_ref()
            .ok_or_else(|| SpecLensError::PtyWriteFailed("session has no PTY writer".into()))?;
        let mut guard = writer
            .lock()
            .map_err(|_| SpecLensError::Internal("pty writer mutex poisoned".into()))?;
        guard
            .write_all(bytes)
            .map_err(|e| SpecLensError::PtyWriteFailed(e.to_string()))?;
        guard
            .flush()
            .map_err(|e| SpecLensError::PtyWriteFailed(e.to_string()))?;
        Ok(())
    }

    /// Resize the PTY window. No-op when the session has no live PTY.
    pub fn resize(&self, cols: u16, rows: u16) -> SpecLensResult<()> {
        if let Some(master) = &self.master {
            let guard = master
                .lock()
                .map_err(|_| SpecLensError::Internal("pty master mutex poisoned".into()))?;
            guard
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|e| SpecLensError::Internal(format!("pty resize failed: {e}")))?;
        }
        Ok(())
    }
}

/// Background thread that reads from the PTY master, splits into lines,
/// batches them into ~16 ms windows, and forwards [`BridgeEvent::Lines`] to
/// the sink. Emits [`BridgeEvent::Closed`] on EOF / IO error.
fn reader_loop(
    session_id: Uuid,
    mut reader: Box<dyn Read + Send>,
    buffer: Arc<Mutex<OutputBuffer>>,
    rules: &'static [CompiledRule],
    sink: std::sync::mpsc::Sender<BridgeEvent>,
) {
    const BATCH_WINDOW: Duration = Duration::from_millis(16);
    let mut carry = String::new();
    let mut pending: Vec<OutputLine> = Vec::with_capacity(32);
    let mut last_flush = Instant::now();
    let mut chunk = [0u8; 8 * 1024];
    loop {
        let n = match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        carry.push_str(&String::from_utf8_lossy(&chunk[..n]));
        // Split off complete lines from `carry`.
        while let Some(idx) = carry.find('\n') {
            let line_text: String = carry.drain(..=idx).collect();
            let trimmed = line_text
                .trim_end_matches('\n')
                .trim_end_matches('\r')
                .to_string();
            let highlight = first_match(&trimmed, rules);
            let line = OutputLine {
                seq: 0,
                ts: now_millis(),
                stream: OutputStream::Stdout,
                text: trimmed,
                ansi_spans: None,
                highlight,
                step_id: None,
            };
            if let Ok(mut buf) = buffer.lock() {
                if let Ok((seq, _spilled)) = buf.push(line.clone()) {
                    let mut stored = line;
                    stored.seq = seq;
                    pending.push(stored);
                }
            }
        }
        if last_flush.elapsed() >= BATCH_WINDOW && !pending.is_empty() {
            let batch = std::mem::take(&mut pending);
            let _ = sink.send(BridgeEvent::Lines {
                session_id,
                lines: batch,
            });
            last_flush = Instant::now();
        }
    }
    if !pending.is_empty() {
        let _ = sink.send(BridgeEvent::Lines {
            session_id,
            lines: pending,
        });
    }
    let _ = sink.send(BridgeEvent::Closed { session_id });
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn default_shell() -> String {
    if cfg!(target_os = "windows") {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
}

impl TerminalSession {
    /// Build a headless session (no live PTY). Useful for tests and for any
    /// future in-process producers (e.g. replaying a recorded log into the
    /// same pipeline that a real session would follow).
    pub fn headless(
        project_id: Uuid,
        window_id: &str,
        spill: PathBuf,
        config: TerminalConfig,
    ) -> Self {
        let buffer = Arc::new(Mutex::new(OutputBuffer::new(
            config.buffer_max_lines,
            config.disk_cap_mib,
            spill,
        )));
        Self {
            descriptor: PtySessionDescriptor {
                session_id: Uuid::new_v4(),
                window_id: window_id.to_string(),
                project_id,
                cwd: ".".to_string(),
                shell: "test-shell".to_string(),
                started_at: Utc::now(),
            },
            buffer,
            config,
            writer: None,
            master: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn ingest_chunk_splits_lines_and_flags_highlights() {
        let tmp = TempDir::new().unwrap();
        let bridge = TerminalBridge::new(tmp.path().to_path_buf());
        let cfg = TerminalConfig::default();
        let session =
            TerminalSession::headless(Uuid::new_v4(), "w1", tmp.path().join("s.ndjson"), cfg);
        let session_id = session.descriptor.session_id;
        bridge.insert(session).unwrap();

        let pushed = bridge
            .ingest_chunk(
                &session_id,
                "plain line\nERROR: boom\nPASS all green\n",
                OutputStream::Stdout,
            )
            .unwrap();
        assert_eq!(pushed.len(), 3);
        assert!(pushed[0].highlight.is_none());
        assert_eq!(
            pushed[1].highlight.as_ref().unwrap().rule_id,
            "error-prefix"
        );
        assert_eq!(
            pushed[2].highlight.as_ref().unwrap().rule_id,
            "success-marker"
        );
    }

    #[test]
    fn sessions_are_isolated_by_id() {
        let tmp = TempDir::new().unwrap();
        let bridge = TerminalBridge::new(tmp.path().to_path_buf());
        let cfg = TerminalConfig::default();
        let a = TerminalSession::headless(Uuid::new_v4(), "w", tmp.path().join("a.ndjson"), cfg);
        let b = TerminalSession::headless(Uuid::new_v4(), "w", tmp.path().join("b.ndjson"), cfg);
        let a_id = a.descriptor.session_id;
        let b_id = b.descriptor.session_id;
        bridge.insert(a).unwrap();
        bridge.insert(b).unwrap();
        assert_eq!(bridge.session_count(), 2);

        bridge
            .ingest_chunk(&a_id, "only in A\n", OutputStream::Stdout)
            .unwrap();
        let a_session = bridge.get(&a_id).unwrap();
        let b_session = bridge.get(&b_id).unwrap();
        assert_eq!(a_session.buffer.lock().unwrap().memory_len(), 1);
        assert_eq!(b_session.buffer.lock().unwrap().memory_len(), 0);

        bridge.remove(&a_id).unwrap();
        assert_eq!(bridge.session_count(), 1);
    }

    #[test]
    fn slice_reads_transparently_after_spill() {
        let tmp = TempDir::new().unwrap();
        let bridge = TerminalBridge::new(tmp.path().to_path_buf());
        let cfg = TerminalConfig {
            buffer_max_lines: 1_000, // honour validator floor
            disk_cap_mib: 16,
        };
        let session =
            TerminalSession::headless(Uuid::new_v4(), "w", tmp.path().join("s.ndjson"), cfg);
        let session_id = session.descriptor.session_id;
        bridge.insert(session).unwrap();

        // Push past the buffer by ingesting many lines.
        let bulk: String = (0..1_200).map(|i| format!("line-{i}\n")).collect();
        bridge
            .ingest_chunk(&session_id, &bulk, OutputStream::Stdout)
            .unwrap();

        let session = bridge.get(&session_id).unwrap();
        let buffer = session.buffer.lock().unwrap();
        assert_eq!(buffer.memory_len(), 1_000);
        assert!(buffer.spill_bytes() > 0);
        // Slice 0..1_200 must return all 1_200 lines regardless of tier.
        let all = buffer.slice(0, 1_200).unwrap();
        assert_eq!(all.len(), 1_200);
        assert_eq!(all.first().unwrap().text, "line-0");
        assert_eq!(all.last().unwrap().text, "line-1199");
    }
}
