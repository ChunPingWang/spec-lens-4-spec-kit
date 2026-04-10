//! Individual task entry parsed out of a task file. See `data-model.md §11`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Skipped,
}

impl TaskStatus {
    /// Normalize an unknown source status (markdown checkbox character, JSON
    /// string, etc.) to one of the four canonical values. Unknown inputs
    /// default to `Todo` per `data-model.md §11 Validation`.
    pub fn normalize(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "x" | "done" | "complete" | "completed" => Self::Done,
            "~" | "-" | "in_progress" | "in-progress" | "progress" => Self::InProgress,
            "!" | "skip" | "skipped" => Self::Skipped,
            _ => Self::Todo,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskEntry {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    #[serde(default)]
    pub section_path: Vec<String>,
    pub file_path: String,
    pub line: u32,
    pub raw: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_known_values() {
        assert_eq!(TaskStatus::normalize("x"), TaskStatus::Done);
        assert_eq!(TaskStatus::normalize("done"), TaskStatus::Done);
        assert_eq!(TaskStatus::normalize("~"), TaskStatus::InProgress);
        assert_eq!(TaskStatus::normalize("in_progress"), TaskStatus::InProgress);
        assert_eq!(TaskStatus::normalize("skipped"), TaskStatus::Skipped);
        assert_eq!(TaskStatus::normalize(" "), TaskStatus::Todo);
        assert_eq!(TaskStatus::normalize("???"), TaskStatus::Todo);
    }
}
