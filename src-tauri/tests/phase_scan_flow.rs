//! Phase 4 US2 acceptance: walk a project directory and report document
//! status; recompute hashes to transition from `Unverified` to `Generated`.

mod common;

use common::fixture_project_ok;

use speclens_lib::models::{DocHashRecord, DocStatus};
use speclens_lib::services::doc_hash_store::compute_sha256_hex;
use speclens_lib::services::{PhaseScanner, StateStore};
use std::collections::HashMap;
use std::fs;

#[test]
fn scan_specify_step_reports_unverified_without_hashes() {
    let (_tmp, root) = fixture_project_ok();
    let hashes: HashMap<String, DocHashRecord> = HashMap::new();
    let docs = PhaseScanner::scan_step_documents(&root, "2-specify", &hashes).expect("scan");
    assert!(
        docs.iter()
            .any(|d| d.relative_path.ends_with("spec.md") && d.status == DocStatus::Unverified),
        "expected spec.md to be Unverified when no hashes are stored, got {docs:?}"
    );
}

#[test]
fn scan_reports_modified_after_edit() {
    let (_tmp, root) = fixture_project_ok();
    // Baseline: store the original plan.md hash.
    let plan_path = root.join("specs/001-sample/plan.md");
    let original_sha = compute_sha256_hex(&plan_path).expect("sha");
    let mut hashes = HashMap::new();
    hashes.insert(
        "specs/001-sample/plan.md".to_string(),
        DocHashRecord::new("specs/001-sample/plan.md".to_string(), original_sha),
    );

    // Mutate the file on disk.
    fs::write(&plan_path, b"# Plan (edited)\n").expect("overwrite plan.md");

    let docs = PhaseScanner::scan_step_documents(&root, "4-plan", &hashes).expect("scan");
    let plan = docs
        .iter()
        .find(|d| d.relative_path.ends_with("plan.md"))
        .expect("plan.md entry");
    assert_eq!(plan.status, DocStatus::Modified);
}

#[test]
fn state_store_persists_hash_map_between_sessions() {
    let (_tmp, root) = fixture_project_ok();
    let store_a = StateStore::new(root.clone());

    let plan_path = root.join("specs/001-sample/plan.md");
    let sha = compute_sha256_hex(&plan_path).unwrap();
    let mut hashes = HashMap::new();
    hashes.insert(
        "specs/001-sample/plan.md".to_string(),
        DocHashRecord::new("specs/001-sample/plan.md".to_string(), sha.clone()),
    );
    store_a.save_hashes(&hashes).expect("save hashes");

    let store_b = StateStore::new(root);
    let loaded = store_b.load_hashes().expect("load hashes");
    assert_eq!(loaded.len(), 1);
    assert_eq!(
        loaded
            .get("specs/001-sample/plan.md")
            .map(|r| r.sha256.as_str()),
        Some(sha.as_str())
    );
}
