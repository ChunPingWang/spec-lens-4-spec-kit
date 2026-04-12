//! T088 — contract tests for the `terminal_set_config` command (Phase 5
//! US3 / FR-048).
//!
//! The Tauri command is a thin wrapper that validates
//! `bufferMaxLines ∈ [1_000, 200_000]` and `diskCapMiB ∈ [16, 8_192]` via
//! `output_buffer::validate_buffer_max_lines` and
//! `output_buffer::validate_disk_cap_mib`. These integration tests pin
//! the accepted and rejected ranges against the pure validators so the
//! contract documented in `contracts/ipc.md` stays honoured without
//! spinning up the Tauri runtime.

use speclens_lib::services::output_buffer::{
    validate_buffer_max_lines, validate_disk_cap_mib, MAX_BUFFER_LINES, MAX_DISK_CAP_MIB,
    MIN_BUFFER_LINES, MIN_DISK_CAP_MIB,
};

// ----- buffer_max_lines ---------------------------------------------------

#[test]
fn buffer_max_lines_accepts_lower_bound() {
    assert_eq!(
        validate_buffer_max_lines(MIN_BUFFER_LINES).unwrap(),
        MIN_BUFFER_LINES
    );
}

#[test]
fn buffer_max_lines_accepts_upper_bound() {
    assert_eq!(
        validate_buffer_max_lines(MAX_BUFFER_LINES).unwrap(),
        MAX_BUFFER_LINES
    );
}

#[test]
fn buffer_max_lines_accepts_mid_range() {
    assert!(validate_buffer_max_lines(10_000).is_ok());
    assert!(validate_buffer_max_lines(50_000).is_ok());
    assert!(validate_buffer_max_lines(150_000).is_ok());
}

#[test]
fn buffer_max_lines_rejects_below_min() {
    assert!(validate_buffer_max_lines(0).is_err());
    assert!(validate_buffer_max_lines(MIN_BUFFER_LINES - 1).is_err());
}

#[test]
fn buffer_max_lines_rejects_above_max() {
    assert!(validate_buffer_max_lines(MAX_BUFFER_LINES + 1).is_err());
    assert!(validate_buffer_max_lines(1_000_000).is_err());
}

#[test]
fn buffer_max_lines_error_message_references_bounds() {
    let err = validate_buffer_max_lines(500).expect_err("below min");
    let msg = format!("{err}");
    assert!(msg.contains(&MIN_BUFFER_LINES.to_string()));
    assert!(msg.contains(&MAX_BUFFER_LINES.to_string()));
}

// ----- disk_cap_mib -------------------------------------------------------

#[test]
fn disk_cap_mib_accepts_lower_bound() {
    assert_eq!(
        validate_disk_cap_mib(MIN_DISK_CAP_MIB).unwrap(),
        MIN_DISK_CAP_MIB
    );
}

#[test]
fn disk_cap_mib_accepts_upper_bound() {
    assert_eq!(
        validate_disk_cap_mib(MAX_DISK_CAP_MIB).unwrap(),
        MAX_DISK_CAP_MIB
    );
}

#[test]
fn disk_cap_mib_accepts_mid_range() {
    assert!(validate_disk_cap_mib(64).is_ok());
    assert!(validate_disk_cap_mib(512).is_ok());
    assert!(validate_disk_cap_mib(4_096).is_ok());
}

#[test]
fn disk_cap_mib_rejects_below_min() {
    assert!(validate_disk_cap_mib(0).is_err());
    assert!(validate_disk_cap_mib(MIN_DISK_CAP_MIB - 1).is_err());
}

#[test]
fn disk_cap_mib_rejects_above_max() {
    assert!(validate_disk_cap_mib(MAX_DISK_CAP_MIB + 1).is_err());
    assert!(validate_disk_cap_mib(100_000).is_err());
}

#[test]
fn disk_cap_mib_error_message_references_bounds() {
    let err = validate_disk_cap_mib(5).expect_err("below min");
    let msg = format!("{err}");
    assert!(msg.contains(&MIN_DISK_CAP_MIB.to_string()));
    assert!(msg.contains(&MAX_DISK_CAP_MIB.to_string()));
}
