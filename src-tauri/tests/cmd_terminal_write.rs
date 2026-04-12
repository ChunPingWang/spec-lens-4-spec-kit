//! T085 — contract tests for `terminal_write` echo round-trip (Phase 5 US3).
//!
//! The Tauri command `terminal_write` sends bytes to the PTY master file
//! descriptor. In production, the PTY reader thread captures the echoed
//! output and pushes it back through the `OutputBuffer` → `pty_output`
//! event pipeline.
//!
//! Since spawning a real PTY in CI is fragile, these tests exercise the
//! *service-level contract* through the headless session + `ingest_chunk`
//! pipeline: write text in, read it back from the buffer, and verify the
//! round-trip preserves content, ordering, and highlight classification.

use speclens_lib::error::SpecLensError;
use speclens_lib::models::{HighlightSeverity, OutputStream};
use speclens_lib::services::terminal_bridge::{TerminalBridge, TerminalConfig, TerminalSession};
use tempfile::TempDir;
use uuid::Uuid;

// ----- basic round-trip ---------------------------------------------------

#[test]
fn ingest_chunk_returns_lines_with_correct_text() {
    let tmp = TempDir::new().unwrap();
    let bridge = TerminalBridge::new(tmp.path().to_path_buf());

    let session = TerminalSession::headless(
        Uuid::new_v4(),
        "w1",
        tmp.path().join("spill.ndjson"),
        TerminalConfig::default(),
    );
    let session_id = session.descriptor.session_id;
    bridge.insert(session).unwrap();

    let lines = bridge
        .ingest_chunk(&session_id, "hello world\n", OutputStream::Stdout)
        .unwrap();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].text, "hello world");
    assert_eq!(lines[0].stream, OutputStream::Stdout);
}

#[test]
fn ingest_chunk_splits_multi_line_input() {
    let tmp = TempDir::new().unwrap();
    let bridge = TerminalBridge::new(tmp.path().to_path_buf());

    let session = TerminalSession::headless(
        Uuid::new_v4(),
        "w1",
        tmp.path().join("spill.ndjson"),
        TerminalConfig::default(),
    );
    let session_id = session.descriptor.session_id;
    bridge.insert(session).unwrap();

    let lines = bridge
        .ingest_chunk(
            &session_id,
            "line one\nline two\nline three\n",
            OutputStream::Stdout,
        )
        .unwrap();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].text, "line one");
    assert_eq!(lines[1].text, "line two");
    assert_eq!(lines[2].text, "line three");
}

#[test]
fn ingest_chunk_assigns_monotonic_sequence_numbers() {
    let tmp = TempDir::new().unwrap();
    let bridge = TerminalBridge::new(tmp.path().to_path_buf());

    let session = TerminalSession::headless(
        Uuid::new_v4(),
        "w1",
        tmp.path().join("spill.ndjson"),
        TerminalConfig::default(),
    );
    let session_id = session.descriptor.session_id;
    bridge.insert(session).unwrap();

    let batch1 = bridge
        .ingest_chunk(&session_id, "first\n", OutputStream::Stdout)
        .unwrap();
    let batch2 = bridge
        .ingest_chunk(&session_id, "second\n", OutputStream::Stdout)
        .unwrap();

    assert!(
        batch2[0].seq > batch1[0].seq,
        "seq must be monotonically increasing: {} > {}",
        batch2[0].seq,
        batch1[0].seq,
    );
}

// ----- highlight classification -------------------------------------------

#[test]
fn ingest_chunk_applies_error_highlight_to_error_lines() {
    let tmp = TempDir::new().unwrap();
    let bridge = TerminalBridge::new(tmp.path().to_path_buf());

    let session = TerminalSession::headless(
        Uuid::new_v4(),
        "w1",
        tmp.path().join("spill.ndjson"),
        TerminalConfig::default(),
    );
    let session_id = session.descriptor.session_id;
    bridge.insert(session).unwrap();

    let lines = bridge
        .ingest_chunk(&session_id, "ERROR: something broke\n", OutputStream::Stderr)
        .unwrap();

    let highlight = lines[0]
        .highlight
        .as_ref()
        .expect("error line should be highlighted");
    assert_eq!(
        highlight.severity,
        HighlightSeverity::Error,
        "severity should be Error"
    );
}

// ----- stderr stream preserved -------------------------------------------

#[test]
fn ingest_chunk_preserves_stderr_stream() {
    let tmp = TempDir::new().unwrap();
    let bridge = TerminalBridge::new(tmp.path().to_path_buf());

    let session = TerminalSession::headless(
        Uuid::new_v4(),
        "w1",
        tmp.path().join("spill.ndjson"),
        TerminalConfig::default(),
    );
    let session_id = session.descriptor.session_id;
    bridge.insert(session).unwrap();

    let lines = bridge
        .ingest_chunk(&session_id, "stderr output\n", OutputStream::Stderr)
        .unwrap();

    assert_eq!(lines[0].stream, OutputStream::Stderr);
}

// ----- write to unknown session -------------------------------------------

#[test]
fn ingest_chunk_on_unknown_session_returns_error() {
    let tmp = TempDir::new().unwrap();
    let bridge = TerminalBridge::new(tmp.path().to_path_buf());
    let unknown = Uuid::new_v4();

    let err = bridge
        .ingest_chunk(&unknown, "data\n", OutputStream::Stdout)
        .unwrap_err();
    assert!(
        matches!(err, SpecLensError::PtySessionNotFound(_)),
        "expected PtySessionNotFound, got {err:?}"
    );
}

// ----- buffer read-back (slice) ------------------------------------------

#[test]
fn buffer_slice_returns_ingested_lines() {
    let tmp = TempDir::new().unwrap();
    let bridge = TerminalBridge::new(tmp.path().to_path_buf());

    let session = TerminalSession::headless(
        Uuid::new_v4(),
        "w1",
        tmp.path().join("spill.ndjson"),
        TerminalConfig::default(),
    );
    let session_id = session.descriptor.session_id;
    bridge.insert(session).unwrap();

    bridge
        .ingest_chunk(&session_id, "alpha\nbeta\n", OutputStream::Stdout)
        .unwrap();

    let sess = bridge.get(&session_id).unwrap();
    let buf = sess.buffer.lock().unwrap();
    let slice = buf.slice(0, 100).unwrap();

    assert_eq!(slice.len(), 2);
    assert_eq!(slice[0].text, "alpha");
    assert_eq!(slice[1].text, "beta");
}
