//! Integration tests for the terminal bridge: PTY session registry,
//! buffer transparency across the in-memory/spill tiers, and highlight
//! routing. Full end-to-end PTY spawn/echo is covered by T091 (Playwright).
//!
//! These tests exercise `TerminalBridge` + `OutputBuffer` as a unit via the
//! headless `TerminalSession::headless` constructor so they run in CI
//! without needing a real shell.

use speclens_lib::models::OutputStream;
use speclens_lib::services::output_buffer::session_spill_path;
use speclens_lib::services::{TerminalBridge, TerminalConfig, TerminalSession};
use tempfile::TempDir;
use uuid::Uuid;

fn setup(buffer_max_lines: usize) -> (TempDir, TerminalBridge, Uuid) {
    let tmp = TempDir::new().unwrap();
    let bridge = TerminalBridge::new(tmp.path().to_path_buf());
    let cfg = TerminalConfig {
        buffer_max_lines,
        disk_cap_mib: 16,
    };
    let session_id_holder = Uuid::new_v4();
    let session = TerminalSession::headless(
        session_id_holder,
        "window-1",
        session_spill_path(tmp.path(), &session_id_holder.to_string()),
        cfg,
    );
    let session_id = session.descriptor.session_id;
    bridge.insert(session).unwrap();
    (tmp, bridge, session_id)
}

#[test]
fn get_slice_spans_memory_and_disk_tiers() {
    let (_tmp, bridge, session_id) = setup(1_000);
    let bulk: String = (0..1_500).map(|i| format!("line-{i}\n")).collect();
    bridge
        .ingest_chunk(&session_id, &bulk, OutputStream::Stdout)
        .unwrap();

    let session = bridge.get(&session_id).unwrap();
    let buffer = session.buffer.lock().unwrap();
    // 1_000 in memory, 500 on disk.
    assert_eq!(buffer.memory_len(), 1_000);
    assert!(buffer.spill_bytes() > 0);
    // Pull the first 10 lines (on disk) and the last 10 lines (in memory).
    let head = buffer.slice(0, 10).unwrap();
    let tail = buffer.slice(1_490, 1_500).unwrap();
    assert_eq!(head.len(), 10);
    assert_eq!(tail.len(), 10);
    assert_eq!(head.first().unwrap().text, "line-0");
    assert_eq!(tail.last().unwrap().text, "line-1499");
    // Straddling slice covers the disk/memory boundary.
    let straddle = buffer.slice(495, 510).unwrap();
    assert_eq!(straddle.len(), 15);
    assert_eq!(straddle.first().unwrap().text, "line-495");
    assert_eq!(straddle.last().unwrap().text, "line-509");
}

#[test]
fn search_plain_finds_across_tiers() {
    let (_tmp, bridge, session_id) = setup(1_000);
    // Seed 1_200 lines, sprinkle NEEDLE tokens on both sides of the boundary.
    let mut chunk = String::new();
    for i in 0..1_200 {
        if i == 50 || i == 1_100 {
            chunk.push_str(&format!("line-{i} NEEDLE here\n"));
        } else {
            chunk.push_str(&format!("line-{i}\n"));
        }
    }
    bridge
        .ingest_chunk(&session_id, &chunk, OutputStream::Stdout)
        .unwrap();

    let session = bridge.get(&session_id).unwrap();
    let buffer = session.buffer.lock().unwrap();
    let hits = buffer.search_plain("NEEDLE", true).unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].seq, 50);
    assert_eq!(hits[1].seq, 1_100);
}

#[test]
fn ingest_chunk_routes_highlights_via_default_rules() {
    let (_tmp, bridge, session_id) = setup(1_000);
    let lines = bridge
        .ingest_chunk(
            &session_id,
            "boot sequence\nERROR: something\nPASS it worked\n",
            OutputStream::Stdout,
        )
        .unwrap();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].highlight.is_none());
    assert_eq!(lines[1].highlight.as_ref().unwrap().rule_id, "error-prefix");
    assert_eq!(
        lines[2].highlight.as_ref().unwrap().rule_id,
        "success-marker"
    );
}
