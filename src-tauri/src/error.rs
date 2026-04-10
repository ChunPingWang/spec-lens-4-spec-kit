//! IPC error catalog for SpecLens.
//!
//! All command handlers return `IpcResult<T>`. Errors are serialized to the
//! frontend as `{ code, message, hint? }` matching `contracts/ipc.md §Error
//! Catalog`. Messages MUST follow the Constitution IV shape
//! "what happened / why / what to do next".

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Wire format for errors returned to the frontend over Tauri IPC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpcError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl IpcError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// Strongly-typed error variants produced inside the backend. Converted to
/// [`IpcError`] at the IPC boundary so the frontend only ever sees the wire
/// shape above.
#[derive(Debug, Error)]
pub enum SpecLensError {
    #[error("project path does not exist: {0}")]
    PathNotFound(String),

    #[error("project path is not a directory: {0}")]
    PathNotDir(String),

    #[error("project path is not readable: {0}")]
    PathNotReadable(String),

    #[error("Spec-Kit step not found: {0}")]
    StepNotFound(String),

    #[error("document not found: {0}")]
    DocNotFound(String),

    #[error("document exceeds max bytes")]
    DocTooLarge,

    #[error("task file not found: {0}")]
    TaskFileNotFound(String),

    #[error("failed to parse task file: {0}")]
    TaskParseFailed(String),

    #[error("failed to spawn PTY: {0}")]
    PtySpawnFailed(String),

    #[error("PTY session not found: {0}")]
    PtySessionNotFound(String),

    #[error("PTY write failed: {0}")]
    PtyWriteFailed(String),

    #[error("invalid config: {0}")]
    ConfigInvalid(String),

    #[error("window not found: {0}")]
    WindowNotFound(String),

    #[error("io error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("serde error: {0}")]
    Serde(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl SpecLensError {
    /// Wrap an io::Error with the path that produced it.
    pub fn io<P: Into<String>>(path: P, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

impl From<serde_json::Error> for SpecLensError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e.to_string())
    }
}

impl From<SpecLensError> for IpcError {
    fn from(err: SpecLensError) -> Self {
        use SpecLensError::*;
        let (code, hint) = match &err {
            PathNotFound(_) => (
                "E_PATH_NOT_FOUND",
                Some("Pick a different folder that exists on disk."),
            ),
            PathNotDir(_) => (
                "E_PATH_NOT_DIR",
                Some("Choose a directory rather than a file."),
            ),
            PathNotReadable(_) => (
                "E_PATH_NOT_READABLE",
                Some("Check the directory permissions and try again."),
            ),
            StepNotFound(_) => (
                "E_STEP_NOT_FOUND",
                Some("Refresh the steps list; this id may be stale."),
            ),
            DocNotFound(_) => ("E_DOC_NOT_FOUND", None),
            DocTooLarge => (
                "E_DOC_TOO_LARGE",
                Some("Open the file in your editor for the full contents."),
            ),
            TaskFileNotFound(_) => ("E_TASK_FILE_NOT_FOUND", None),
            TaskParseFailed(_) => (
                "E_TASK_PARSE_FAILED",
                Some("Check the file syntax against the expected format."),
            ),
            PtySpawnFailed(_) => ("E_PTY_SPAWN_FAILED", Some("Verify your shell is on PATH.")),
            PtySessionNotFound(_) => ("E_PTY_SESSION_NOT_FOUND", None),
            PtyWriteFailed(_) => ("E_PTY_WRITE_FAILED", None),
            ConfigInvalid(_) => (
                "E_CONFIG_INVALID",
                Some("Adjust the value to fit the documented range."),
            ),
            WindowNotFound(_) => ("E_WINDOW_NOT_FOUND", None),
            Io { .. } | Serde(_) | Internal(_) => ("E_INTERNAL", None),
        };
        let mut out = IpcError::new(code, err.to_string());
        if let Some(h) = hint {
            out = out.with_hint(h);
        }
        out
    }
}

pub type IpcResult<T> = std::result::Result<T, IpcError>;
pub type SpecLensResult<T> = std::result::Result<T, SpecLensError>;
