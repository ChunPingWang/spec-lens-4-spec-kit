//! `project.*` IPC commands. Phase 3 US1 MVP implementation.

use std::path::PathBuf;

use chrono::Utc;
use tauri::State;
use tracing::info;
use uuid::Uuid;

use crate::error::{IpcError, IpcResult, SpecLensError};
use crate::models::{AgentProfile, EnvironmentStatus, Project, RecentProject};
use crate::services::{PhaseScanner, StateStore};
use crate::state::{AppState, OpenProject};

#[tauri::command]
pub async fn project_pick_directory() -> IpcResult<Option<String>> {
    // The actual folder picker runs on the frontend via plugin-dialog;
    // this backend stub exists so the IPC catalog stays symmetrical and
    // so tests can mock a typed return.
    Ok(None)
}

#[tauri::command]
pub async fn project_open(path: String, state: State<'_, AppState>) -> IpcResult<Project> {
    let root = PathBuf::from(&path);
    let detection = PhaseScanner::detect(&root).map_err(IpcError::from)?;
    if !detection.is_speckit_project() {
        return Err(IpcError::from(SpecLensError::PathNotReadable(path.clone()))
            .hint_or("No .specify/ or specs/ directory found in this folder."));
    }

    let id = Uuid::new_v4();
    let name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();

    // Load or bootstrap per-project state (best-effort: fall back to default
    // so a corrupt state.json cannot lock the user out of their project).
    let state_store = StateStore::new(root.clone());
    let mut project_state = state_store.load_state().unwrap_or_default();
    project_state.last_opened_at = Utc::now();

    // Placeholder env + agent for US1; real detection lands in US5/US7.
    let environment = EnvironmentStatus::placeholder();
    let agent = AgentProfile::generic();

    let project = Project {
        id,
        name: name.clone(),
        root_path: root.display().to_string(),
        speckit_detected: true,
        opened_at: Utc::now(),
        environment,
        agent,
        steps: PhaseScanner::placeholder_steps(),
        task_files: Vec::new(),
        state: project_state,
    };

    // Persist to recents (LRU dedupe by path).
    state
        .app_data
        .mutate(|cfg| {
            cfg.touch_recent(RecentProject {
                id,
                name: name.clone(),
                path: root.display().to_string(),
                last_opened_at: Utc::now(),
                pinned: false,
            });
            cfg.last_updated_at = Utc::now();
            Ok(())
        })
        .map_err(IpcError::from)?;

    // Save initial per-project state file.
    let _ = state_store.save_state(&project.state);

    // Register in open-projects table.
    {
        let mut open = state
            .open_projects
            .lock()
            .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
        open.insert(
            id,
            OpenProject {
                project: project.clone(),
                state_store,
            },
        );
    }

    info!(project = %name, path = %root.display(), "project opened");
    Ok(project)
}

#[tauri::command]
pub async fn project_list_recent(state: State<'_, AppState>) -> IpcResult<Vec<RecentProject>> {
    let cfg = state.app_data.load().map_err(IpcError::from)?;
    Ok(cfg.recent_projects)
}

#[tauri::command]
pub async fn project_remove_recent(id: Uuid, state: State<'_, AppState>) -> IpcResult<()> {
    state
        .app_data
        .mutate(|cfg| {
            cfg.remove_recent(id);
            Ok(())
        })
        .map_err(IpcError::from)?;
    Ok(())
}

#[tauri::command]
pub async fn project_pin_recent(
    id: Uuid,
    pinned: bool,
    state: State<'_, AppState>,
) -> IpcResult<()> {
    state
        .app_data
        .mutate(|cfg| {
            cfg.set_pinned(id, pinned);
            Ok(())
        })
        .map_err(IpcError::from)?;
    Ok(())
}

#[tauri::command]
pub async fn project_set_last_step(
    project_id: Uuid,
    step_id: Option<String>,
    state: State<'_, AppState>,
) -> IpcResult<()> {
    let mut open = state
        .open_projects
        .lock()
        .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
    let entry = open.get_mut(&project_id).ok_or_else(|| {
        IpcError::new("E_PATH_NOT_FOUND", "project is not currently open")
            .hint_or("Open the project first with project.open.")
    })?;

    // Validate step id exists on the project (or accept `None` to clear).
    if let Some(ref sid) = step_id {
        if !entry.project.steps.iter().any(|s| &s.id == sid) {
            return Err(
                IpcError::new("E_STEP_NOT_FOUND", format!("unknown step id: {sid}"))
                    .hint_or("Refresh the steps list; this id may be stale."),
            );
        }
    }

    entry.project.state.last_step_id = step_id;
    entry
        .state_store
        .save_state(&entry.project.state)
        .map_err(IpcError::from)?;
    Ok(())
}

#[tauri::command]
pub async fn project_close(id: Uuid, state: State<'_, AppState>) -> IpcResult<()> {
    let mut open = state
        .open_projects
        .lock()
        .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
    if let Some(entry) = open.remove(&id) {
        // Persist the last known project state on the way out.
        let _ = entry.state_store.save_state(&entry.project.state);
    }
    Ok(())
}

// ---- small helpers ----

trait IpcErrorExt {
    fn hint_or(self, hint: &str) -> Self;
}

impl IpcErrorExt for IpcError {
    fn hint_or(mut self, hint: &str) -> Self {
        if self.hint.is_none() {
            self.hint = Some(hint.to_string());
        }
        self
    }
}
