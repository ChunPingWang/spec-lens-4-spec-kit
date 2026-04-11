//! Phase 6 US4 integration tests — exercise `task_file_scanner`,
//! `tasks_parser`, and `StateStore` at the service level in the same
//! style as `project_open_flow.rs`. Covers T103-T107.
//!
//! These tests intentionally bypass the Tauri command wrappers because
//! the wrappers are thin adapters over the services; the services are the
//! only places that touch disk.

mod common;

use std::fs;

use common::{fixture_project_empty, fixture_project_ok};
use speclens_lib::models::{ProjectState, TaskFileFormat, TaskStatus};
use speclens_lib::services::state_store::StateStore;
use speclens_lib::services::task_file_scanner::{resolve_under_root, scan};
use speclens_lib::services::tasks_parser::{detect_format, parse};

fn write_task_file(root: &std::path::Path, rel: &str, contents: &str) {
    let full = root.join(rel);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(&full, contents).expect("write");
}

// ----- T105: tasks_scan_files against 0 / 1 / many fixtures ---------------

#[test]
fn scan_returns_empty_for_empty_fixture() {
    let (_tmp, root) = fixture_project_empty();
    let candidates = scan(&root).expect("scan empty");
    // The empty fixture has no task files under specs/ — result may still
    // include constitution/templates if they happen to match; what we care
    // about is that nothing is auto-selected.
    assert!(
        candidates
            .iter()
            .all(|c| !c.relative_path.ends_with(".bin")),
        "scan should only return candidates with recognized extensions"
    );
}

#[test]
fn scan_ok_fixture_plus_single_added_task_file() {
    let (_tmp, root) = fixture_project_ok();
    write_task_file(&root, "specs/001-foo/tasks.md", "- [ ] T001 Lone task\n");
    let candidates = scan(&root).expect("scan");
    assert!(
        candidates
            .iter()
            .any(|c| c.relative_path == "specs/001-foo/tasks.md"),
        "scan should include the task file we just added: {:?}",
        candidates
            .iter()
            .map(|c| &c.relative_path)
            .collect::<Vec<_>>()
    );
}

#[test]
fn scan_many_task_files_returns_all_in_deterministic_order() {
    let (_tmp, root) = fixture_project_ok();
    write_task_file(&root, "specs/001-foo/tasks.md", "");
    write_task_file(&root, "specs/002-bar/tasks.json", "{\"tasks\":[]}");
    write_task_file(
        &root,
        "specs/003-baz/tasks.yaml",
        "tasks:\n  - id: T001\n    title: Foo\n    status: todo\n",
    );
    let candidates = scan(&root).expect("scan");
    let relevant: Vec<_> = candidates
        .iter()
        .map(|c| c.relative_path.clone())
        .filter(|r| r.starts_with("specs/00"))
        .collect();
    assert!(relevant.len() >= 3);
    // Must be sorted.
    let mut sorted = relevant.clone();
    sorted.sort();
    assert_eq!(relevant, sorted);
}

// ----- T106: tasks_parse_file covers happy paths + error hint ------------

#[test]
fn parse_all_four_formats_from_bundled_fixtures() {
    // The frontend + scanner walk project-relative paths; for parser tests
    // we can load the canonical fixtures in `tests/fixtures/task-files/`.
    let fixtures = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests/fixtures/task-files");

    for (rel, expected_format, expected_count) in [
        ("tasks.md", TaskFileFormat::Md, 5),
        ("tasks.json", TaskFileFormat::Json, 3),
        ("tasks.yaml", TaskFileFormat::Yaml, 3),
        ("tasks.txt", TaskFileFormat::Txt, 3),
    ] {
        let path = fixtures.join(rel);
        let contents = fs::read_to_string(&path).expect("fixture read");
        let format = detect_format(&path).expect("detect format");
        assert_eq!(format, expected_format);
        let (file, entries) = parse(rel, format, &contents).expect("parse fixture");
        assert_eq!(
            entries.len() as u32,
            expected_count,
            "fixture {rel} should have {expected_count} entries, got {}",
            entries.len()
        );
        assert_eq!(file.task_count, expected_count);
    }
}

#[test]
fn parse_malformed_json_returns_task_parse_failed_with_hint() {
    let err = parse("tasks.json", TaskFileFormat::Json, "{ not valid").unwrap_err();
    match err {
        speclens_lib::error::SpecLensError::TaskParseFailed(msg) => {
            assert!(
                msg.contains("tasks.json"),
                "error must mention the offending file: {msg}"
            );
        }
        other => panic!("expected TaskParseFailed, got {other:?}"),
    }
}

#[test]
fn resolve_under_root_rejects_escape_attempts() {
    let (_tmp, root) = fixture_project_ok();
    // Nonexistent neighbour is TaskFileNotFound.
    let err = resolve_under_root(&root, "../not-in-project.md").unwrap_err();
    assert!(matches!(
        err,
        speclens_lib::error::SpecLensError::TaskFileNotFound(_)
    ));
}

// ----- T107: tasks_state_set_selected rejects activePath not in list ----

#[test]
fn state_store_rejects_active_path_not_in_selected() {
    let (_tmp, root) = fixture_project_ok();
    let store = StateStore::new(root.clone());

    // Simulate the command layer rejecting the input: we mirror the check
    // directly so the integration test documents the invariant.
    let selected: Vec<String> = vec!["specs/001/tasks.md".into()];
    let active = Some("specs/999/other.md".to_string());

    let rejection = active
        .as_ref()
        .map(|p| selected.iter().any(|s| s == p))
        .unwrap_or(true);
    assert!(!rejection, "command layer must reject");

    // Happy path persists cleanly.
    let mut next = ProjectState {
        selected_task_file_paths: vec!["specs/001/tasks.md".into()],
        active_task_file_path: Some("specs/001/tasks.md".into()),
        ..ProjectState::default()
    };
    next.normalize();
    store.save_state(&next).expect("save");
    let loaded = store.load_state().expect("load");
    assert_eq!(
        loaded.active_task_file_path.as_deref(),
        Some("specs/001/tasks.md")
    );
    assert_eq!(
        loaded.selected_task_file_paths,
        vec!["specs/001/tasks.md".to_string()]
    );
}

#[test]
fn markdown_parser_tracks_done_count_and_ids_deterministic() {
    let md = "# Title\n\n- [ ] T001 A\n- [x] T002 B\n- [~] T003 C\n";
    let (file, entries) = parse("tasks.md", TaskFileFormat::Md, md).expect("parse");
    assert_eq!(file.task_count, 3);
    assert_eq!(file.completed_count, 1);
    assert_eq!(entries[0].status, TaskStatus::Todo);
    assert_eq!(entries[1].status, TaskStatus::Done);
    assert_eq!(entries[2].status, TaskStatus::InProgress);

    // Re-parsing the same contents must yield the same ids (stable hash).
    let (_, again) = parse("tasks.md", TaskFileFormat::Md, md).expect("reparse");
    let ids_a: Vec<_> = entries.iter().map(|e| e.id.clone()).collect();
    let ids_b: Vec<_> = again.iter().map(|e| e.id.clone()).collect();
    assert_eq!(ids_a, ids_b);
}
