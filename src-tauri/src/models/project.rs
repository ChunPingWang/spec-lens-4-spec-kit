//! In-memory Project aggregate. See `data-model.md §4` and `§12`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{AgentProfile, EnvironmentStatus, Step, TaskFile};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub root_path: String,
    pub speckit_detected: bool,
    pub opened_at: DateTime<Utc>,
    pub environment: EnvironmentStatus,
    pub agent: AgentProfile,
    pub steps: Vec<Step>,
    #[serde(default)]
    pub task_files: Vec<TaskFile>,
    pub state: ProjectState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PhaseTab {
    Overview,
    Documents,
    Tasks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectState {
    pub schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_step_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_phase_tab: Option<PhaseTab>,
    #[serde(default)]
    pub selected_task_file_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_task_file_path: Option<String>,
    #[serde(default)]
    pub terminal_collapsed: bool,
    pub last_opened_at: DateTime<Utc>,
}

impl Default for ProjectState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            last_step_id: None,
            last_phase_tab: None,
            selected_task_file_paths: Vec::new(),
            active_task_file_path: None,
            terminal_collapsed: false,
            last_opened_at: Utc::now(),
        }
    }
}

impl ProjectState {
    /// Ensures `active_task_file_path`, if set, is also present in the
    /// `selected_task_file_paths` list. See `data-model.md §12 Validation`.
    pub fn normalize(&mut self) {
        if let Some(active) = &self.active_task_file_path {
            if !self.selected_task_file_paths.iter().any(|p| p == active) {
                self.active_task_file_path = None;
            }
        }
    }
}
