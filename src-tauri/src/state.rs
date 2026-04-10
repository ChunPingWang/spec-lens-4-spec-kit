//! Shared Tauri managed state.
//!
//! Lives for the lifetime of the application. Holds the cross-project
//! [`AppDataStore`] plus a map of currently-open projects (one window ==
//! one project, but keying by UUID keeps the multi-window model explicit
//! for Phase 8).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use uuid::Uuid;

use crate::models::Project;
use crate::services::{AppDataStore, StateStore};

/// The single value registered via `Builder::manage`.
pub struct AppState {
    pub app_data: AppDataStore,
    pub open_projects: Mutex<HashMap<Uuid, OpenProject>>,
}

/// One entry per open project in the current Tauri instance.
pub struct OpenProject {
    pub project: Project,
    pub state_store: StateStore,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            app_data: AppDataStore::new(app_data_dir),
            open_projects: Mutex::new(HashMap::new()),
        }
    }
}
