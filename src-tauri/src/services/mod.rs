//! Service layer: persistence, filesystem watching, and Spec-Kit scanners.
//!
//! Per `plan.md` the service layer is the only place allowed to touch disk
//! outside of tests. IPC command handlers call into these services rather
//! than performing IO directly.

pub mod agent_detector;
pub mod app_data_store;
pub mod doc_hash_store;
pub mod fs_watcher;
pub mod highlight_rules;
pub mod output_buffer;
pub mod phase_scanner;
pub mod state_store;
pub mod task_file_scanner;
pub mod tasks_parser;
pub mod terminal_bridge;
pub mod version_checker;

pub use app_data_store::AppDataStore;
pub use fs_watcher::{FsWatcher, FsWatcherEvent};
pub use phase_scanner::PhaseScanner;
pub use state_store::StateStore;
pub use task_file_scanner::{scan as scan_task_files, TaskFileCandidate};
pub use terminal_bridge::{BridgeEvent, TerminalBridge, TerminalConfig, TerminalSession};
