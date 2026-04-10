//! Step entity with the six-state status enum. See `data-model.md §7`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::PhaseDocument;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    #[default]
    NotStarted,
    InProgress,
    Done,
    Modified,
    Missing,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub id: String,
    pub order: u32,
    pub display_name: String,
    pub status: StepStatus,
    #[serde(default)]
    pub phase_documents: Vec<PhaseDocument>,
    #[serde(default)]
    pub task_file_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_observed_at: Option<DateTime<Utc>>,
}

impl Step {
    /// Build a step with default status `NotStarted` in the given order slot.
    pub fn new(id: impl Into<String>, order: u32, display_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            order,
            display_name: display_name.into(),
            status: StepStatus::NotStarted,
            phase_documents: Vec::new(),
            task_file_paths: Vec::new(),
            last_observed_at: None,
        }
    }
}
