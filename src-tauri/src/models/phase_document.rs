//! Phase documents and document-status enum. See `data-model.md §8`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::DocHashRecord;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DocKind {
    Spec,
    Plan,
    Research,
    DataModel,
    Contract,
    Quickstart,
    Tasks,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocStatus {
    Generated,
    Modified,
    Missing,
    Unverified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PhaseDocument {
    pub relative_path: String,
    pub display_name: String,
    pub kind: DocKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<DocHashRecord>,
    pub status: DocStatus,
    pub size: u64,
    pub modified_at: DateTime<Utc>,
}
