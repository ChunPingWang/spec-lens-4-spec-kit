//! Serde-serializable domain entities. Every type in this module is part of
//! the IPC contract defined in `specs/001-speclens-desktop/data-model.md`.

pub mod agent;
pub mod app_config;
pub mod doc_hash;
pub mod env;
pub mod phase_document;
pub mod project;
pub mod pty;
pub mod step;
pub mod task_entry;
pub mod task_file;

pub use agent::*;
pub use app_config::*;
pub use doc_hash::*;
pub use env::*;
pub use phase_document::*;
pub use project::*;
pub use pty::*;
pub use step::*;
pub use task_entry::*;
pub use task_file::*;
