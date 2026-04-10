//! Spec-Kit project scanner — detects `.specify/` / `specs/` layout and
//! produces a list of [`Step`]s plus [`PhaseDocument`]s.
//!
//! The full scan/parse pipeline is implemented in Phase 3 (US1). This
//! module currently exposes the shared API and a minimal "detect" path
//! used by the `project.open` command to confirm a directory looks like
//! a Spec-Kit project before walking it.

use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::{SpecLensError, SpecLensResult};
use crate::models::Step;
#[cfg(test)]
use crate::models::StepStatus;

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
