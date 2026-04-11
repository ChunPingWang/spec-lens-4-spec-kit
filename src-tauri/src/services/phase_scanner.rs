//! Spec-Kit project scanner — detects `.specify/` / `specs/` layout and
//! produces a list of [`Step`]s plus [`PhaseDocument`]s.
//!
//! The full scan/parse pipeline is implemented in Phase 3 (US1). This
//! module currently exposes the shared API and a minimal "detect" path
//! used by the `project.open` command to confirm a directory looks like
//! a Spec-Kit project before walking it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{SpecLensError, SpecLensResult};
#[cfg(test)]
use crate::models::StepStatus;
use crate::models::{DocHashRecord, DocKind, DocStatus, PhaseDocument, Step};
use crate::services::doc_hash_store::compute_sha256_hex;

/// Minimal project descriptor produced by [`PhaseScanner::detect`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectDetection {
    pub root: PathBuf,
    pub specify_dir_exists: bool,
    pub specs_dir_exists: bool,
    pub feature_json_exists: bool,
}

impl ProjectDetection {
    pub fn is_speckit_project(&self) -> bool {
        self.specify_dir_exists || self.specs_dir_exists
    }
}

/// Shape of `.specify/feature.json` — the pointer to the active feature
/// directory within `specs/`. Extra fields are ignored.
#[derive(Debug, Clone, Deserialize, Default)]
struct FeatureConfig {
    feature_directory: Option<String>,
}

/// Per-step document candidate: where the file _should_ live on disk plus
/// its typed classification. Missing files become `DocStatus::Missing`.
struct DocCandidate {
    relative_path: String,
    display_name: String,
    kind: DocKind,
}

/// Entry point for Spec-Kit scanning. The scanner is stateless — all
/// mutation goes through the service layer.
#[derive(Debug, Default)]
pub struct PhaseScanner;

impl PhaseScanner {
    /// Verify that a path exists, is a directory, and is readable.
    /// Returns a `ProjectDetection` summarising which Spec-Kit markers
    /// were found.
    pub fn detect(root: impl AsRef<Path>) -> SpecLensResult<ProjectDetection> {
        let root = root.as_ref();
        if !root.exists() {
            return Err(SpecLensError::PathNotFound(root.display().to_string()));
        }
        let meta = std::fs::metadata(root)
            .map_err(|e| SpecLensError::io(root.display().to_string(), e))?;
        if !meta.is_dir() {
            return Err(SpecLensError::PathNotDir(root.display().to_string()));
        }
        // Attempt a read_dir to confirm the directory is readable.
        std::fs::read_dir(root).map_err(|e| SpecLensError::io(root.display().to_string(), e))?;

        let specify = root.join(".specify");
        let specs = root.join("specs");
        let feature_json = specify.join("feature.json");
        Ok(ProjectDetection {
            root: root.to_path_buf(),
            specify_dir_exists: specify.is_dir(),
            specs_dir_exists: specs.is_dir(),
            feature_json_exists: feature_json.is_file(),
        })
    }

    /// Read `.specify/feature.json` and return the active feature directory
    /// relative to the project root, if present.
    pub fn active_feature_dir(root: &Path) -> Option<PathBuf> {
        let config_path = root.join(".specify").join("feature.json");
        let bytes = std::fs::read(&config_path).ok()?;
        let cfg: FeatureConfig = serde_json::from_slice(&bytes).ok()?;
        cfg.feature_directory.map(PathBuf::from)
    }

    /// Return the list of document candidates expected for a given step id,
    /// relative to the project root. The scanner uses this to produce
    /// `PhaseDocument[]` (including entries that are currently missing).
    fn document_candidates(root: &Path, step_id: &str) -> Vec<DocCandidate> {
        let feature_dir = Self::active_feature_dir(root)
            .unwrap_or_else(|| PathBuf::from("specs").join("000-unknown"));
        let feature_str = feature_dir.to_string_lossy().to_string();

        let one = |rel: String, name: &str, kind: DocKind| -> DocCandidate {
            DocCandidate {
                relative_path: rel,
                display_name: name.to_string(),
                kind,
            }
        };

        match step_id {
            "1-constitution" => vec![one(
                ".specify/memory/constitution.md".into(),
                "constitution.md",
                DocKind::Other,
            )],
            "2-specify" | "3-clarify" => {
                vec![one(
                    format!("{feature_str}/spec.md"),
                    "spec.md",
                    DocKind::Spec,
                )]
            }
            "4-plan" => {
                let base = &feature_str;
                vec![
                    one(format!("{base}/plan.md"), "plan.md", DocKind::Plan),
                    one(
                        format!("{base}/research.md"),
                        "research.md",
                        DocKind::Research,
                    ),
                    one(
                        format!("{base}/data-model.md"),
                        "data-model.md",
                        DocKind::DataModel,
                    ),
                    one(
                        format!("{base}/quickstart.md"),
                        "quickstart.md",
                        DocKind::Quickstart,
                    ),
                ]
            }
            "5-tasks" => vec![one(
                format!("{feature_str}/tasks.md"),
                "tasks.md",
                DocKind::Tasks,
            )],
            "6-analyze" | "7-implement" => Vec::new(),
            _ => Vec::new(),
        }
    }

    /// Walk the disk for a single step and produce [`PhaseDocument`] entries
    /// with status derived from the on-disk state + the persisted hashes.
    ///
    /// - `Generated` : file exists and the stored hash matches.
    /// - `Modified`  : file exists but its hash differs from the record.
    /// - `Unverified`: file exists but we have no hash on record yet.
    /// - `Missing`   : expected file is absent.
    pub fn scan_step_documents(
        root: &Path,
        step_id: &str,
        hashes: &HashMap<String, DocHashRecord>,
    ) -> SpecLensResult<Vec<PhaseDocument>> {
        let mut out = Vec::new();
        for candidate in Self::document_candidates(root, step_id) {
            let abs = root.join(&candidate.relative_path);
            if !abs.exists() {
                out.push(PhaseDocument {
                    relative_path: candidate.relative_path,
                    display_name: candidate.display_name,
                    kind: candidate.kind,
                    hash: None,
                    status: DocStatus::Missing,
                    size: 0,
                    modified_at: Utc::now(),
                });
                continue;
            }

            let meta = std::fs::metadata(&abs)
                .map_err(|e| SpecLensError::io(abs.display().to_string(), e))?;
            let modified_at: DateTime<Utc> = meta
                .modified()
                .ok()
                .and_then(|t| {
                    let d = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                    chrono::DateTime::<Utc>::from_timestamp(d.as_secs() as i64, d.subsec_nanos())
                })
                .unwrap_or_else(Utc::now);

            let sha = compute_sha256_hex(&abs)?;
            let (status, hash_record) = match hashes.get(&candidate.relative_path) {
                None => (DocStatus::Unverified, None),
                Some(rec) => {
                    if rec.sha256 == sha {
                        (DocStatus::Generated, Some(rec.clone()))
                    } else {
                        (DocStatus::Modified, Some(rec.clone()))
                    }
                }
            };
            out.push(PhaseDocument {
                relative_path: candidate.relative_path,
                display_name: candidate.display_name,
                kind: candidate.kind,
                hash: hash_record,
                status,
                size: meta.len(),
                modified_at,
            });
        }
        Ok(out)
    }

    /// Placeholder step list used by the welcome screen until the full
    /// scanner lands in Phase 3. Returns the 7 Spec-Kit steps in the
    /// `not_started` state so the UI can render a skeleton.
    pub fn placeholder_steps() -> Vec<Step> {
        const STEPS: &[(&str, &str)] = &[
            ("1-constitution", "Constitution"),
            ("2-specify", "Specify"),
            ("3-clarify", "Clarify"),
            ("4-plan", "Plan"),
            ("5-tasks", "Tasks"),
            ("6-analyze", "Analyze"),
            ("7-implement", "Implement"),
        ];
        let now = Utc::now();
        STEPS
            .iter()
            .enumerate()
            .map(|(idx, (id, name))| {
                let mut step = Step::new(*id, idx as u32, *name);
                step.last_observed_at = Some(now);
                step
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn detect_returns_not_found_for_missing_path() {
        let err = PhaseScanner::detect("/definitely/does/not/exist/speclens").unwrap_err();
        assert!(matches!(err, SpecLensError::PathNotFound(_)));
    }

    #[test]
    fn detect_returns_not_dir_for_file() {
        let tmp = TempDir::new().expect("tempdir");
        let file = tmp.path().join("not-a-dir.txt");
        fs::write(&file, "x").expect("write");
        let err = PhaseScanner::detect(&file).unwrap_err();
        assert!(matches!(err, SpecLensError::PathNotDir(_)));
    }

    #[test]
    fn detect_flags_specify_and_specs_dirs() {
        let tmp = TempDir::new().expect("tempdir");
        fs::create_dir_all(tmp.path().join(".specify")).expect("mk .specify");
        fs::create_dir_all(tmp.path().join("specs/001-sample")).expect("mk specs");
        fs::write(tmp.path().join(".specify/feature.json"), "{}").expect("write feature.json");

        let det = PhaseScanner::detect(tmp.path()).expect("detect");
        assert!(det.specify_dir_exists);
        assert!(det.specs_dir_exists);
        assert!(det.feature_json_exists);
        assert!(det.is_speckit_project());
    }

    fn make_ok_project() -> TempDir {
        let tmp = TempDir::new().expect("tempdir");
        let root = tmp.path();
        fs::create_dir_all(root.join(".specify/memory")).unwrap();
        fs::write(
            root.join(".specify/feature.json"),
            r#"{"feature_directory":"specs/001-sample"}"#,
        )
        .unwrap();
        fs::write(
            root.join(".specify/memory/constitution.md"),
            "# Constitution\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("specs/001-sample")).unwrap();
        fs::write(root.join("specs/001-sample/spec.md"), "# Spec\n").unwrap();
        fs::write(root.join("specs/001-sample/plan.md"), "# Plan\n").unwrap();
        fs::write(root.join("specs/001-sample/tasks.md"), "# Tasks\n").unwrap();
        tmp
    }

    #[test]
    fn scan_step_documents_flags_plan_step_files() {
        let tmp = make_ok_project();
        let root = tmp.path();
        let hashes = HashMap::new();
        let docs = PhaseScanner::scan_step_documents(root, "4-plan", &hashes).expect("scan");
        let paths: Vec<&str> = docs.iter().map(|d| d.relative_path.as_str()).collect();
        assert!(paths.contains(&"specs/001-sample/plan.md"));
        assert!(paths.contains(&"specs/001-sample/research.md"));
        let plan = docs
            .iter()
            .find(|d| d.relative_path == "specs/001-sample/plan.md")
            .unwrap();
        assert_eq!(plan.status, DocStatus::Unverified);
        let research = docs
            .iter()
            .find(|d| d.relative_path == "specs/001-sample/research.md")
            .unwrap();
        assert_eq!(research.status, DocStatus::Missing);
    }

    #[test]
    fn scan_step_documents_reports_generated_when_hash_matches() {
        let tmp = make_ok_project();
        let root = tmp.path();
        let plan_abs = root.join("specs/001-sample/plan.md");
        let sha = compute_sha256_hex(&plan_abs).unwrap();
        let mut hashes = HashMap::new();
        hashes.insert(
            "specs/001-sample/plan.md".to_string(),
            DocHashRecord::new("specs/001-sample/plan.md".to_string(), sha),
        );
        let docs = PhaseScanner::scan_step_documents(root, "4-plan", &hashes).unwrap();
        let plan = docs
            .iter()
            .find(|d| d.relative_path == "specs/001-sample/plan.md")
            .unwrap();
        assert_eq!(plan.status, DocStatus::Generated);
        assert!(plan.hash.is_some());
    }

    #[test]
    fn scan_step_documents_flags_modified_when_content_diverges() {
        let tmp = make_ok_project();
        let root = tmp.path();
        let mut hashes = HashMap::new();
        hashes.insert(
            "specs/001-sample/plan.md".to_string(),
            DocHashRecord::new(
                "specs/001-sample/plan.md".to_string(),
                "0".repeat(64), // wrong, so it's "modified"
            ),
        );
        let docs = PhaseScanner::scan_step_documents(root, "4-plan", &hashes).unwrap();
        let plan = docs
            .iter()
            .find(|d| d.relative_path == "specs/001-sample/plan.md")
            .unwrap();
        assert_eq!(plan.status, DocStatus::Modified);
    }

    #[test]
    fn scan_step_documents_empty_for_analyze_and_implement() {
        let tmp = make_ok_project();
        let hashes = HashMap::new();
        assert!(
            PhaseScanner::scan_step_documents(tmp.path(), "6-analyze", &hashes)
                .unwrap()
                .is_empty()
        );
        assert!(
            PhaseScanner::scan_step_documents(tmp.path(), "7-implement", &hashes)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn placeholder_steps_has_seven_entries_in_order() {
        let steps = PhaseScanner::placeholder_steps();
        assert_eq!(steps.len(), 7);
        assert_eq!(steps[0].id, "1-constitution");
        assert_eq!(steps[6].id, "7-implement");
        for (idx, step) in steps.iter().enumerate() {
            assert_eq!(step.order as usize, idx);
            assert_eq!(step.status, StepStatus::NotStarted);
        }
    }
}
