//! Environment status + issue models. See `data-model.md §5, §16`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EnvBadgeColor {
    Green,
    Amber,
    Red,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnvIssueKind {
    MissingBinary,
    VersionMismatch,
    PathNotWritable,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EnvIssueSeverity {
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EnvIssue {
    pub kind: EnvIssueKind,
    pub severity: EnvIssueSeverity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentStatus {
    pub speckit_installed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speckit_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speckit_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_known_version: Option<String>,
    #[serde(default)]
    pub update_available: bool,
    pub checked_at: DateTime<Utc>,
    #[serde(default)]
    pub issues: Vec<EnvIssue>,
}

impl EnvironmentStatus {
    pub fn placeholder() -> Self {
        Self {
            speckit_installed: false,
            speckit_version: None,
            speckit_path: None,
            latest_known_version: None,
            update_available: false,
            checked_at: Utc::now(),
            issues: Vec::new(),
        }
    }

    pub fn badge_color(&self) -> EnvBadgeColor {
        let has_error = self
            .issues
            .iter()
            .any(|i| i.severity == EnvIssueSeverity::Error);
        let has_warn = self
            .issues
            .iter()
            .any(|i| i.severity == EnvIssueSeverity::Warn);
        if !self.speckit_installed || has_error {
            EnvBadgeColor::Red
        } else if self.update_available || has_warn {
            EnvBadgeColor::Amber
        } else {
            EnvBadgeColor::Green
        }
    }
}
