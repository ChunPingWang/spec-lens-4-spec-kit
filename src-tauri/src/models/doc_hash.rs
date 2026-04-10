//! Document hash record stored in `.speclens/doc-hashes.json`. See
//! `data-model.md §9`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocHashRecord {
    pub path: String,
    pub sha256: String,
    pub generated_at: DateTime<Utc>,
    pub last_verified_at: DateTime<Utc>,
}

impl DocHashRecord {
    pub fn new(path: String, sha256: String) -> Self {
        let now = Utc::now();
        Self {
            path,
            sha256,
            generated_at: now,
            last_verified_at: now,
        }
    }

    pub fn is_valid_sha256(&self) -> bool {
        self.sha256.len() == 64
            && self
                .sha256
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    }
}
