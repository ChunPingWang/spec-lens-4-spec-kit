//! In-memory ring buffer plus append-only spill file for PTY output lines.
//!
//! Each terminal session owns one [`OutputBuffer`]. Incoming lines are
//! appended with a monotonically increasing `seq`; when the buffer exceeds
//! `max_lines` the oldest entries are evicted from memory and re-hydrated
//! from `.speclens/logs/<sessionId>.ndjson` on demand. See
//! `specs/001-speclens-desktop/data-model.md §13` and US3 tasks T082 / T092.

use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::error::{SpecLensError, SpecLensResult};
use crate::models::OutputLine;

/// Absolute floor / ceiling from FR-048. Contract tests lock this.
pub const MIN_BUFFER_LINES: usize = 1_000;
pub const MAX_BUFFER_LINES: usize = 200_000;
pub const MIN_DISK_CAP_MIB: u64 = 16;
pub const MAX_DISK_CAP_MIB: u64 = 8_192;

/// A bounded ring buffer over [`OutputLine`] with transparent overflow to a
/// newline-delimited JSON spill file.
pub struct OutputBuffer {
    ring: VecDeque<OutputLine>,
    max_lines: usize,
    next_seq: u64,
    spill_path: PathBuf,
    spill_writer: Option<File>,
    spill_bytes: u64,
    disk_cap_bytes: u64,
    /// First seq that still lives in memory; anything below this must be
    /// read from the spill file.
    memory_start_seq: u64,
}

impl OutputBuffer {
    /// Create a new buffer. `spill_path` is created lazily on first overflow.
    pub fn new(max_lines: usize, disk_cap_mib: u64, spill_path: PathBuf) -> Self {
        Self {
            ring: VecDeque::with_capacity(max_lines.min(4_096)),
            max_lines,
            next_seq: 0,
            spill_path,
            spill_writer: None,
            spill_bytes: 0,
            disk_cap_bytes: disk_cap_mib.saturating_mul(1024 * 1024),
            memory_start_seq: 0,
        }
    }

    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }

    pub fn memory_len(&self) -> usize {
        self.ring.len()
    }

    pub fn spill_bytes(&self) -> u64 {
        self.spill_bytes
    }

    /// Append a line, assigning it the next sequence number. Returns the
    /// assigned seq and a boolean indicating whether the append triggered a
    /// spill to disk (`true` == a line was evicted to the NDJSON file).
    pub fn push(&mut self, mut line: OutputLine) -> SpecLensResult<(u64, bool)> {
        line.seq = self.next_seq;
        self.next_seq += 1;
        self.ring.push_back(line);
        let spilled = if self.ring.len() > self.max_lines {
            let evicted = self.ring.pop_front().expect("len > max_lines guarantees");
            self.write_spill(&evicted)?;
            self.memory_start_seq = self.ring.front().map(|l| l.seq).unwrap_or(self.next_seq);
            true
        } else if self.ring.len() == 1 {
            self.memory_start_seq = self.ring[0].seq;
            false
        } else {
            false
        };
        Ok((self.next_seq - 1, spilled))
    }

    /// Return lines with `from_seq <= seq < to_seq`. Transparently merges
    /// in-memory entries with any on-disk predecessors so callers never need
    /// to know about the spill tier.
    pub fn slice(&self, from_seq: u64, to_seq: u64) -> SpecLensResult<Vec<OutputLine>> {
        if to_seq <= from_seq {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        // On-disk tier first.
        if from_seq < self.memory_start_seq {
            let upper = to_seq.min(self.memory_start_seq);
            out.extend(self.read_spill_range(from_seq, upper)?);
        }
        // In-memory tier.
        let mem_from = from_seq.max(self.memory_start_seq);
        if mem_from < to_seq {
            for line in &self.ring {
                if line.seq >= mem_from && line.seq < to_seq {
                    out.push(line.clone());
                }
            }
        }
        Ok(out)
    }

    /// Plain-text search across both tiers. Returns matching lines in seq
    /// order. Callers decide whether to treat `needle` as regex.
    pub fn search_plain(
        &self,
        needle: &str,
        case_sensitive: bool,
    ) -> SpecLensResult<Vec<OutputLine>> {
        let cmp = |haystack: &str| -> bool {
            if case_sensitive {
                haystack.contains(needle)
            } else {
                haystack.to_lowercase().contains(&needle.to_lowercase())
            }
        };
        let mut out = Vec::new();
        if self.memory_start_seq > 0 && self.spill_path.exists() {
            let file = File::open(&self.spill_path)
                .map_err(|e| SpecLensError::io(self.spill_path.display().to_string(), e))?;
            for raw in BufReader::new(file).lines() {
                let raw =
                    raw.map_err(|e| SpecLensError::io(self.spill_path.display().to_string(), e))?;
                if raw.is_empty() {
                    continue;
                }
                let line: OutputLine = serde_json::from_str(&raw)?;
                if cmp(&line.text) {
                    out.push(line);
                }
            }
        }
        for line in &self.ring {
            if cmp(&line.text) {
                out.push(line.clone());
            }
        }
        Ok(out)
    }

    fn write_spill(&mut self, line: &OutputLine) -> SpecLensResult<()> {
        if self.spill_writer.is_none() {
            if let Some(parent) = self.spill_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| SpecLensError::io(parent.display().to_string(), e))?;
            }
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.spill_path)
                .map_err(|e| SpecLensError::io(self.spill_path.display().to_string(), e))?;
            self.spill_writer = Some(file);
        }
        let writer = self.spill_writer.as_mut().expect("writer created above");
        let mut json = serde_json::to_vec(line)?;
        json.push(b'\n');
        let len = json.len() as u64;
        // Enforce the disk cap: drop the line silently rather than growing
        // beyond the cap. The frontend is notified via a separate
        // `buffer_spilled` event (emitted by the caller).
        if self.spill_bytes + len > self.disk_cap_bytes {
            return Ok(());
        }
        writer
            .write_all(&json)
            .map_err(|e| SpecLensError::io(self.spill_path.display().to_string(), e))?;
        self.spill_bytes += len;
        Ok(())
    }

    fn read_spill_range(&self, from_seq: u64, to_seq: u64) -> SpecLensResult<Vec<OutputLine>> {
        if !self.spill_path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&self.spill_path)
            .map_err(|e| SpecLensError::io(self.spill_path.display().to_string(), e))?;
        let mut out = Vec::new();
        for raw in BufReader::new(file).lines() {
            let raw =
                raw.map_err(|e| SpecLensError::io(self.spill_path.display().to_string(), e))?;
            if raw.is_empty() {
                continue;
            }
            let line: OutputLine = serde_json::from_str(&raw)?;
            if line.seq >= from_seq && line.seq < to_seq {
                out.push(line);
            }
        }
        Ok(out)
    }
}

/// Validate `bufferMaxLines` per FR-048. Returns the clamped value or an
/// error suitable for `terminal.setConfig`.
pub fn validate_buffer_max_lines(value: usize) -> SpecLensResult<usize> {
    if (MIN_BUFFER_LINES..=MAX_BUFFER_LINES).contains(&value) {
        Ok(value)
    } else {
        Err(SpecLensError::ConfigInvalid(format!(
            "bufferMaxLines must be between {MIN_BUFFER_LINES} and {MAX_BUFFER_LINES}; got {value}"
        )))
    }
}

/// Validate `diskCapMiB` per FR-048.
pub fn validate_disk_cap_mib(value: u64) -> SpecLensResult<u64> {
    if (MIN_DISK_CAP_MIB..=MAX_DISK_CAP_MIB).contains(&value) {
        Ok(value)
    } else {
        Err(SpecLensError::ConfigInvalid(format!(
            "diskCapMiB must be between {MIN_DISK_CAP_MIB} and {MAX_DISK_CAP_MIB}; got {value}"
        )))
    }
}

pub fn session_spill_path(logs_dir: &Path, session_id: &str) -> PathBuf {
    logs_dir.join(format!("{session_id}.ndjson"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::OutputStream;
    use tempfile::TempDir;

    fn line(text: &str) -> OutputLine {
        OutputLine {
            seq: 0,
            ts: 0,
            stream: OutputStream::Stdout,
            text: text.to_string(),
            ansi_spans: None,
            highlight: None,
            step_id: None,
        }
    }

    #[test]
    fn push_assigns_monotonic_seq() {
        let tmp = TempDir::new().unwrap();
        let mut buf = OutputBuffer::new(10, 16, tmp.path().join("s.ndjson"));
        assert_eq!(buf.push(line("a")).unwrap().0, 0);
        assert_eq!(buf.push(line("b")).unwrap().0, 1);
        assert_eq!(buf.next_seq(), 2);
    }

    #[test]
    fn ring_evicts_oldest_and_spills_to_disk() {
        let tmp = TempDir::new().unwrap();
        let mut buf = OutputBuffer::new(3, 16, tmp.path().join("s.ndjson"));
        for i in 0..5 {
            let (_, spilled) = buf.push(line(&format!("line-{i}"))).unwrap();
            // First 3 pushes fit; pushes 4+5 evict.
            if i < 3 {
                assert!(!spilled);
            } else {
                assert!(spilled);
            }
        }
        assert_eq!(buf.memory_len(), 3);
        assert!(buf.spill_bytes() > 0);
        // Memory now holds lines [2, 3, 4]; disk holds [0, 1].
        let mem_slice = buf.slice(2, 5).unwrap();
        assert_eq!(mem_slice.len(), 3);
        assert_eq!(mem_slice[0].text, "line-2");
    }

    #[test]
    fn slice_transparently_merges_memory_and_spill() {
        let tmp = TempDir::new().unwrap();
        let mut buf = OutputBuffer::new(2, 16, tmp.path().join("s.ndjson"));
        for i in 0..5 {
            buf.push(line(&format!("line-{i}"))).unwrap();
        }
        // Buffer has 3/4 in memory, 0/1/2 on disk.
        let merged = buf.slice(0, 5).unwrap();
        assert_eq!(merged.len(), 5);
        for (i, l) in merged.iter().enumerate() {
            assert_eq!(l.seq, i as u64);
            assert_eq!(l.text, format!("line-{i}"));
        }
    }

    #[test]
    fn search_plain_case_insensitive_across_tiers() {
        let tmp = TempDir::new().unwrap();
        let mut buf = OutputBuffer::new(2, 16, tmp.path().join("s.ndjson"));
        buf.push(line("Hello world")).unwrap();
        buf.push(line("another line")).unwrap();
        buf.push(line("HELLO again")).unwrap();
        buf.push(line("tail")).unwrap();
        let hits = buf.search_plain("hello", false).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].text, "Hello world");
        assert_eq!(hits[1].text, "HELLO again");
    }

    #[test]
    fn validate_buffer_max_lines_bounds() {
        assert!(validate_buffer_max_lines(999).is_err());
        assert!(validate_buffer_max_lines(1_000).is_ok());
        assert!(validate_buffer_max_lines(200_000).is_ok());
        assert!(validate_buffer_max_lines(200_001).is_err());
    }

    #[test]
    fn validate_disk_cap_mib_bounds() {
        assert!(validate_disk_cap_mib(15).is_err());
        assert!(validate_disk_cap_mib(16).is_ok());
        assert!(validate_disk_cap_mib(8_192).is_ok());
        assert!(validate_disk_cap_mib(8_193).is_err());
    }
}
