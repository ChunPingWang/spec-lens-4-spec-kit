//! Shared Tauri managed state.
//!
//! Lives for the lifetime of the application. Holds the cross-project
//! [`AppDataStore`] plus a map of currently-open projects (one window ==
//! one project, but keying by UUID keeps the multi-window model explicit
//! for Phase 8).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

use uuid::Uuid;

use crate::models::Project;
use crate::services::{AppDataStore, BridgeEvent, StateStore, TerminalBridge};

/// The single value registered via `Builder::manage`.
pub struct AppState {
    pub app_data: AppDataStore,
    pub open_projects: Mutex<HashMap<Uuid, OpenProject>>,
    pub terminal: TerminalBridge,
    pub terminal_event_tx: Sender<BridgeEvent>,
    /// Receiver is pulled out at startup by the Tauri layer to spawn an
    /// event forwarder. `Mutex<Option<_>>` so the take is `&self`-safe.
    pub terminal_event_rx: Mutex<Option<Receiver<BridgeEvent>>>,
}

/// One entry per open project in the current Tauri instance.
pub struct OpenProject {
    pub project: Project,
    pub state_store: StateStore,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let (tx, rx) = channel::<BridgeEvent>();
        // Terminal log spill files live alongside the app-data dir so they
        // share the OS cleanup policy. Per-project logs would be nicer but
        // require knowing the project root, which isn't available until
        // `project.open` — keep it simple for now (US3 covers observe-only).
        let logs_root = app_data_dir.join("logs");
        Self {
            app_data: AppDataStore::new(app_data_dir),
            open_projects: Mutex::new(HashMap::new()),
            terminal: TerminalBridge::new(logs_root),
            terminal_event_tx: tx,
            terminal_event_rx: Mutex::new(Some(rx)),
        }
    }
}
