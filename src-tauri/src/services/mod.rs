//! Service layer: persistence, filesystem watching, and Spec-Kit scanners.
//!
//! Per `plan.md` the service layer is the only place allowed to touch disk
//! outside of tests. IPC command handlers call into these services rather
//! than performing IO directly.

pub mod app_data_store;
pub mod fs_watcher;
pub mod phase_scanner;
pub mod state_store;

pub use app_data_store::AppDataStore;
pub use fs_watcher::{FsWatcher, FsWatcherEvent};
pub use phase_scanner::PhaseScanner;
pub use state_store::StateStore;
