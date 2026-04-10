//! Cross-project configuration persistence.
//!
//! Stores `AppConfig` under the OS-specific app data directory (see
//! `data-model.md §2`). The store is responsible for atomic writes and for
//! validating deserialized config before returning it.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{from_slice, to_vec_pretty};

use crate::error::{SpecLensError, SpecLensResult};
use crate::models::AppConfig;

const CONFIG_FILE_NAME: &str = "config.json";

/// Persists `AppConfig` to a user-provided directory. Uses an internal
/// mutex so concurrent IPC handlers cannot corrupt the JSON file.
#[derive(Debug)]
pub struct AppDataStore {
    dir: PathBuf,
    lock: Mutex<()>,
}

impl AppDataStore {
    /// Create a new store rooted at `dir`. The directory is created lazily
    /// when [`Self::save`] is called.
    pub fn new<P: Into<PathBuf>>(dir: P) -> Self {
        Self {
            dir: dir.into(),
            lock: Mutex::new(()),
        }
    }

    /// Absolute path of the config file backing the store.
    pub fn config_path(&self) -> PathBuf {
        self.dir.join(CONFIG_FILE_NAME)
    }

    /// Load the config from disk. Missing file returns the default config.
    pub fn load(&self) -> SpecLensResult<AppConfig> {
        let _guard = self.lock.lock().expect("app data store mutex poisoned");
        let path = self.config_path();
        if !path.exists() {
            return Ok(AppConfig::default());
        }
        let bytes =
            fs::read(&path).map_err(|e| SpecLensError::io(path.display().to_string(), e))?;
        let cfg: AppConfig = from_slice(&bytes)
            .map_err(|e| SpecLensError::ConfigInvalid(format!("parse config.json: {e}")))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Persist the config atomically (write to temp file + rename).
    pub fn save(&self, cfg: &AppConfig) -> SpecLensResult<()> {
        cfg.validate()?;
        let _guard = self.lock.lock().expect("app data store mutex poisoned");
        fs::create_dir_all(&self.dir)
            .map_err(|e| SpecLensError::io(self.dir.display().to_string(), e))?;
        let path = self.config_path();
        let tmp = path.with_extension("json.tmp");
        let bytes = to_vec_pretty(cfg)
            .map_err(|e| SpecLensError::ConfigInvalid(format!("serialize config: {e}")))?;
        fs::write(&tmp, &bytes).map_err(|e| SpecLensError::io(tmp.display().to_string(), e))?;
        fs::rename(&tmp, &path).map_err(|e| SpecLensError::io(path.display().to_string(), e))?;
        Ok(())
    }

    /// Convenience helper: load, mutate, save.
    pub fn mutate<F>(&self, f: F) -> SpecLensResult<AppConfig>
    where
        F: FnOnce(&mut AppConfig) -> SpecLensResult<()>,
    {
        let mut cfg = self.load()?;
        f(&mut cfg)?;
        self.save(&cfg)?;
        Ok(cfg)
    }

    /// Accessor used by tests.
    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{LanguagePreference, RecentProject, ThemePreference};
    use chrono::Utc;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn fresh_store() -> (TempDir, AppDataStore) {
        let tmp = TempDir::new().expect("tempdir");
        let store = AppDataStore::new(tmp.path().to_path_buf());
        (tmp, store)
    }

    fn sample_recent(name: &str) -> RecentProject {
        RecentProject {
            id: Uuid::new_v4(),
            name: name.to_string(),
            path: format!("/tmp/{name}"),
            last_opened_at: Utc::now(),
            pinned: false,
        }
    }

    #[test]
    fn load_returns_default_when_file_missing() {
        let (_tmp, store) = fresh_store();
        let cfg = store.load().expect("load default");
        assert_eq!(cfg.language, LanguagePreference::System);
        assert_eq!(cfg.theme, ThemePreference::System);
        assert!(cfg.recent_projects.is_empty());
    }

    #[test]
    fn save_then_load_roundtrips() {
        let (_tmp, store) = fresh_store();
        let mut cfg = AppConfig {
            language: LanguagePreference::ZhTw,
            ..AppConfig::default()
        };
        cfg.touch_recent(sample_recent("example"));
        store.save(&cfg).expect("save");

        let loaded = store.load().expect("load");
        assert_eq!(loaded.language, LanguagePreference::ZhTw);
        assert_eq!(loaded.recent_projects.len(), 1);
        assert_eq!(loaded.recent_projects[0].path, "/tmp/example");
    }

    #[test]
    fn save_rejects_invalid_config() {
        let (_tmp, store) = fresh_store();
        let cfg = AppConfig {
            terminal_buffer_max_lines: 10, // below minimum
            ..AppConfig::default()
        };
        let err = store.save(&cfg).unwrap_err();
        assert!(matches!(err, SpecLensError::ConfigInvalid(_)));
    }

    #[test]
    fn mutate_applies_and_persists_change() {
        let (_tmp, store) = fresh_store();
        let cfg = store
            .mutate(|c| {
                c.theme = ThemePreference::Dark;
                Ok(())
            })
            .expect("mutate");
        assert_eq!(cfg.theme, ThemePreference::Dark);

        let reloaded = store.load().expect("reload");
        assert_eq!(reloaded.theme, ThemePreference::Dark);
    }
}
