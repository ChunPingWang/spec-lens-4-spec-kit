//! Per-project state persistence under `<projectRoot>/.speclens/`.
//!
//! See `data-model.md §12` (ProjectState) and `§9` (DocHashRecord).
//! This store is responsible for the `state.json` and `hashes.json` files
//! and ensures writes stay inside the declared project root (Constitution
//! safety: writes locked to project directory).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{from_slice, to_vec_pretty};

use crate::error::{SpecLensError, SpecLensResult};
use crate::models::{DocHashRecord, ProjectState};

const STATE_DIR: &str = ".speclens";
const STATE_FILE: &str = "state.json";
const HASHES_FILE: &str = "hashes.json";

/// Persists per-project state inside `<root>/.speclens/`.
#[derive(Debug)]
pub struct StateStore {
    root: PathBuf,
    lock: Mutex<()>,
}

impl StateStore {
    pub fn new<P: Into<PathBuf>>(project_root: P) -> Self {
        Self {
            root: project_root.into(),
            lock: Mutex::new(()),
        }
    }

    pub fn project_root(&self) -> &Path {
        &self.root
    }

    pub fn state_dir(&self) -> PathBuf {
        self.root.join(STATE_DIR)
    }

    pub fn state_path(&self) -> PathBuf {
        self.state_dir().join(STATE_FILE)
    }

    pub fn hashes_path(&self) -> PathBuf {
        self.state_dir().join(HASHES_FILE)
    }

    /// Load persisted `ProjectState`. Missing file returns a normalized default.
    pub fn load_state(&self) -> SpecLensResult<ProjectState> {
        let _guard = self.lock.lock().expect("state store mutex poisoned");
        let path = self.state_path();
        if !path.exists() {
            return Ok(ProjectState::default());
        }
        let bytes =
            fs::read(&path).map_err(|e| SpecLensError::io(path.display().to_string(), e))?;
        let mut state: ProjectState = from_slice(&bytes)
            .map_err(|e| SpecLensError::ConfigInvalid(format!("parse state.json: {e}")))?;
        state.normalize();
        Ok(state)
    }

    /// Persist `ProjectState` atomically.
    pub fn save_state(&self, state: &ProjectState) -> SpecLensResult<()> {
        let _guard = self.lock.lock().expect("state store mutex poisoned");
        let dir = self.state_dir();
        fs::create_dir_all(&dir).map_err(|e| SpecLensError::io(dir.display().to_string(), e))?;
        let path = self.state_path();
        let tmp = path.with_extension("json.tmp");
        let bytes = to_vec_pretty(state)
            .map_err(|e| SpecLensError::ConfigInvalid(format!("serialize state: {e}")))?;
        fs::write(&tmp, &bytes).map_err(|e| SpecLensError::io(tmp.display().to_string(), e))?;
        fs::rename(&tmp, &path).map_err(|e| SpecLensError::io(path.display().to_string(), e))?;
        Ok(())
    }

    /// Load the doc-hash map keyed by relative path.
    pub fn load_hashes(&self) -> SpecLensResult<HashMap<String, DocHashRecord>> {
        let _guard = self.lock.lock().expect("state store mutex poisoned");
        let path = self.hashes_path();
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let bytes =
            fs::read(&path).map_err(|e| SpecLensError::io(path.display().to_string(), e))?;
        let records: HashMap<String, DocHashRecord> = from_slice(&bytes)
            .map_err(|e| SpecLensError::ConfigInvalid(format!("parse hashes.json: {e}")))?;
        Ok(records)
    }

    pub fn save_hashes(&self, records: &HashMap<String, DocHashRecord>) -> SpecLensResult<()> {
        let _guard = self.lock.lock().expect("state store mutex poisoned");
        let dir = self.state_dir();
        fs::create_dir_all(&dir).map_err(|e| SpecLensError::io(dir.display().to_string(), e))?;
        let path = self.hashes_path();
        let tmp = path.with_extension("json.tmp");
        let bytes = to_vec_pretty(records)
            .map_err(|e| SpecLensError::ConfigInvalid(format!("serialize hashes: {e}")))?;
        fs::write(&tmp, &bytes).map_err(|e| SpecLensError::io(tmp.display().to_string(), e))?;
        fs::rename(&tmp, &path).map_err(|e| SpecLensError::io(path.display().to_string(), e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PhaseTab;
    use tempfile::TempDir;

    fn fresh_store() -> (TempDir, StateStore) {
        let tmp = TempDir::new().expect("tempdir");
        let store = StateStore::new(tmp.path().to_path_buf());
        (tmp, store)
    }

    #[test]
    fn load_state_returns_default_when_missing() {
        let (_tmp, store) = fresh_store();
        let state = store.load_state().expect("load default");
        assert!(state.last_phase_tab.is_none());
        assert!(state.last_step_id.is_none());
    }

    #[test]
    fn state_roundtrip_preserves_fields() {
        let (_tmp, store) = fresh_store();
        let state = ProjectState {
            last_step_id: Some("2-specify".into()),
            last_phase_tab: Some(PhaseTab::Tasks),
            terminal_collapsed: true,
            ..ProjectState::default()
        };
        store.save_state(&state).expect("save");

        let loaded = store.load_state().expect("load");
        assert_eq!(loaded.last_step_id.as_deref(), Some("2-specify"));
        assert_eq!(loaded.last_phase_tab, Some(PhaseTab::Tasks));
        assert!(loaded.terminal_collapsed);
    }

    #[test]
    fn hashes_roundtrip_persists_records() {
        let (_tmp, store) = fresh_store();
        let mut map = HashMap::new();
        map.insert(
            "specs/001/spec.md".to_string(),
            DocHashRecord::new("specs/001/spec.md".into(), "0".repeat(64)),
        );
        store.save_hashes(&map).expect("save");

        let loaded = store.load_hashes().expect("load");
        assert_eq!(loaded.len(), 1);
        assert!(loaded.contains_key("specs/001/spec.md"));
    }

    #[test]
    fn state_file_created_under_speclens_dir() {
        let (tmp, store) = fresh_store();
        store.save_state(&ProjectState::default()).expect("save");
        let expected = tmp.path().join(".speclens").join("state.json");
        assert!(
            expected.exists(),
            "state.json should exist at {}",
            expected.display()
        );
    }
}
