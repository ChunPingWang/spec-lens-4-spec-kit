//! PTY session and output line entities. See `data-model.md §13-14`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnsiSpan {
    pub start: u32,
    pub end: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<u8>,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HighlightSeverity {
    Info,
    Success,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HighlightMatch {
    pub rule_id: String,
    pub severity: HighlightSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OutputLine {
    pub seq: u64,
    pub ts: u64,
    pub stream: OutputStream,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ansi_spans: Option<Vec<AnsiSpan>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight: Option<HighlightMatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PtySessionDescriptor {
    pub session_id: Uuid,
    pub window_id: String,
    pub project_id: Uuid,
    pub cwd: String,
    pub shell: String,
    pub started_at: DateTime<Utc>,
}
