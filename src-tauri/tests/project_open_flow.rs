//! Phase 3 US1 acceptance: open a Spec-Kit project, persist it in recents,
//! emit a `Project` aggregate with the 7 placeholder steps. Exercised
//! through the service layer (the IPC wrapper is a thin adapter).

mod common;

use common::{fixture_project_empty, fixture_project_ok};

use speclens_lib::services::{AppDataStore, PhaseScanner, StateStore};
use tempfile::TempDir;

#[test]
fn detect_ok_fixture_reports_speckit_project() {
    let (_tmp, root) = fixture_project_ok();
    let det = PhaseScanner::detect(&root).expect("detect");
    assert!(det.is_speckit_project());
    assert!(det.specify_dir_exists);
    assert!(det.specs_dir_exists);
    assert!(det.feature_json_exists);
}

#[test]
fn detect_empty_fixture_still_counts_as_speckit_project() {
    let (_tmp, root) = fixture_project_empty();
    let det = PhaseScanner::detect(&root).expect("detect empty");
    assert!(det.is_speckit_project());
    assert!(det.specify_dir_exists);
    assert!(!det.feature_json_exists);
}

#[test]
fn state_store_persists_last_step_between_opens() {
    let (_tmp, root) = fixture_project_ok();
    let store = StateStore::new(root.clone());

    // First open: select step 2-specify.
    let mut state = store.load_state().expect("load empty");
    state.last_step_id = Some("2-specify".into());
    store.save_state(&state).expect("save");

    // Second open: the saved selection comes back.
    let reopened = StateStore::new(root);
    let loaded = reopened.load_state().expect("load");
    assert_eq!(loaded.last_step_id.as_deref(), Some("2-specify"));
}

#[test]
fn app_data_store_bumps_recent_project_on_open() {
    let tmp = TempDir::new().expect("appdata tempdir");
    let store = AppDataStore::new(tmp.path().to_path_buf());
    let (_proj_tmp, root) = fixture_project_ok();

    let cfg1 = store
        .mutate(|cfg| {
            cfg.touch_recent(speclens_lib::models::RecentProject {
                id: uuid::Uuid::new_v4(),
                name: "ok".into(),
                path: root.display().to_string(),
                last_opened_at: chrono::Utc::now(),
                pinned: false,
            });
            Ok(())
        })
        .expect("mutate 1");
    assert_eq!(cfg1.recent_projects.len(), 1);

    // Touching the same path must dedupe rather than add a new entry.
    let cfg2 = store
        .mutate(|cfg| {
            cfg.touch_recent(speclens_lib::models::RecentProject {
                id: uuid::Uuid::new_v4(),
                name: "ok".into(),
                path: root.display().to_string(),
                last_opened_at: chrono::Utc::now(),
                pinned: false,
            });
            Ok(())
        })
        .expect("mutate 2");
    assert_eq!(cfg2.recent_projects.len(), 1);
}

#[test]
fn placeholder_steps_cover_all_seven_spec_kit_phases() {
    let steps = PhaseScanner::placeholder_steps();
    let ids: Vec<&str> = steps.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "1-constitution",
            "2-specify",
            "3-clarify",
            "4-plan",
            "5-tasks",
            "6-analyze",
            "7-implement",
        ]
    );
}
