//! Cross-project application config persisted to the OS app-data directory.
//! Mirrors `data-model.md §1-3`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{SpecLensError, SpecLensResult};

pub const SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_TERMINAL_BUFFER_MAX_LINES: u32 = 10_000;
pub const DEFAULT_TERMINAL_DISK_CAP_MIB: u32 = 512;
pub const BUFFER_MAX_LINES_RANGE: (u32, u32) = (1_000, 200_000);
pub const DISK_CAP_MIB_RANGE: (u32, u32) = (16, 8_192);
pub const RECENT_PROJECTS_LIMIT: usize = 20;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LanguagePreference {
    #[serde(rename = "en")]
    En,
    #[serde(rename = "zh-TW")]
    ZhTw,
    #[default]
    #[serde(rename = "system")]
    System,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    Light,
    Dark,
    #[default]
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecentProject {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub last_opened_at: DateTime<Utc>,
    #[serde(default)]
    pub pinned: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WindowHandle {
    pub window_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<Uuid>,
    pub bounds: WindowBounds,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub schema_version: u32,
    pub language: LanguagePreference,
    pub theme: ThemePreference,
    #[serde(default)]
    pub recent_projects: Vec<RecentProject>,
    pub terminal_buffer_max_lines: u32,
    pub terminal_disk_cap_mib: u32,
    #[serde(default)]
    pub open_windows: Vec<WindowHandle>,
    #[serde(default)]
    pub notifications_enabled: bool,
    pub last_updated_at: DateTime<Utc>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            language: LanguagePreference::default(),
            theme: ThemePreference::default(),
            recent_projects: Vec::new(),
            terminal_buffer_max_lines: DEFAULT_TERMINAL_BUFFER_MAX_LINES,
            terminal_disk_cap_mib: DEFAULT_TERMINAL_DISK_CAP_MIB,
            open_windows: Vec::new(),
            notifications_enabled: true,
            last_updated_at: Utc::now(),
        }
    }
}

impl AppConfig {
    pub fn validate(&self) -> SpecLensResult<()> {
        let (min_lines, max_lines) = BUFFER_MAX_LINES_RANGE;
        if !(min_lines..=max_lines).contains(&self.terminal_buffer_max_lines) {
            return Err(SpecLensError::ConfigInvalid(format!(
                "terminalBufferMaxLines must be in [{min_lines},{max_lines}]"
            )));
        }
        let (min_cap, max_cap) = DISK_CAP_MIB_RANGE;
        if !(min_cap..=max_cap).contains(&self.terminal_disk_cap_mib) {
            return Err(SpecLensError::ConfigInvalid(format!(
                "terminalDiskCapMiB must be in [{min_cap},{max_cap}]"
            )));
        }
        Ok(())
    }

    /// Insert or bump a recent-projects entry using LRU + pinned-first ordering.
    /// The entry's `last_opened_at` is stamped with the current time so repeat
    /// touches always bubble the project to the top of the list.
    pub fn touch_recent(&mut self, mut entry: RecentProject) {
        entry.last_opened_at = Utc::now();
        self.recent_projects
            .retain(|r| r.id != entry.id && r.path != entry.path);
        self.recent_projects.insert(0, entry);
        self.enforce_recent_limit();
    }

    pub fn remove_recent(&mut self, id: Uuid) {
        self.recent_projects.retain(|r| r.id != id);
    }

    pub fn set_pinned(&mut self, id: Uuid, pinned: bool) {
        if let Some(entry) = self.recent_projects.iter_mut().find(|r| r.id == id) {
            entry.pinned = pinned;
        }
        self.sort_recent();
    }

    fn sort_recent(&mut self) {
        self.recent_projects.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then_with(|| b.last_opened_at.cmp(&a.last_opened_at))
        });
    }

    fn enforce_recent_limit(&mut self) {
        self.sort_recent();
        let pinned = self.recent_projects.iter().filter(|r| r.pinned).count();
        let allowed = RECENT_PROJECTS_LIMIT.max(pinned);
        if self.recent_projects.len() > allowed {
            self.recent_projects.truncate(allowed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn sample_project(name: &str, pinned: bool) -> RecentProject {
        RecentProject {
            id: Uuid::new_v4(),
            name: name.to_string(),
            path: format!("/tmp/{name}"),
            last_opened_at: Utc::now(),
            pinned,
        }
    }

    #[test]
    fn default_config_validates() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn buffer_max_lines_out_of_range_rejected() {
        let config = AppConfig {
            terminal_buffer_max_lines: 10,
            ..AppConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn disk_cap_out_of_range_rejected() {
        let config = AppConfig {
            terminal_disk_cap_mib: 1,
            ..AppConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn touch_recent_moves_to_front_and_dedupes() {
        let mut config = AppConfig::default();
        let a = sample_project("a", false);
        let b = sample_project("b", false);
        config.touch_recent(a.clone());
        config.touch_recent(b.clone());
        config.touch_recent(a.clone());
        assert_eq!(config.recent_projects[0].id, a.id);
        assert_eq!(config.recent_projects.len(), 2);
    }

    #[test]
    fn pinned_projects_sort_first() {
        let mut config = AppConfig::default();
        let a = sample_project("a", false);
        let b = sample_project("b", true);
        config.touch_recent(a);
        config.touch_recent(b.clone());
        config.set_pinned(b.id, true);
        assert_eq!(config.recent_projects[0].id, b.id);
    }

    #[test]
    fn recent_projects_cap_enforced() {
        let mut config = AppConfig::default();
        for i in 0..30 {
            config.touch_recent(sample_project(&format!("p{i}"), false));
        }
        assert_eq!(config.recent_projects.len(), RECENT_PROJECTS_LIMIT);
    }
}
