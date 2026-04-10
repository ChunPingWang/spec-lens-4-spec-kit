//! Task file descriptor. See `data-model.md §10`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskFileFormat {
    Md,
    Json,
    Yaml,
    Txt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskFile {
    pub relative_path: String,
    pub format: TaskFileFormat,
    pub display_name: String,
    pub task_count: u32,
    pub completed_count: u32,
    pub parsed_at: DateTime<Utc>,
}
