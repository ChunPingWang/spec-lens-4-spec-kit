//! `tasks.*` commands (Phase 6 US4). See `contracts/ipc.md §D`.
//!
//! These handlers are thin wrappers around `task_file_scanner`,
//! `tasks_parser`, and the per-project `StateStore`. They enforce the
//! FR-085 "writes stay under project root" rule by resolving every
//! incoming `relativePath` through `task_file_scanner::resolve_under_root`.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::error::{IpcError, IpcResult, SpecLensError};
use crate::models::{ProjectState, TaskEntry, TaskFile};
use crate::services::task_file_scanner::{resolve_under_root, scan, TaskFileCandidate};
use crate::services::tasks_parser::{detect_format, parse};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksParseResponse {
    pub file: TaskFile,
    pub entries: Vec<TaskEntry>,
}

fn project_root(project_id: Uuid, state: &State<'_, AppState>) -> IpcResult<PathBuf> {
    let open = state
        .open_projects
        .lock()
        .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
    let entry = open
        .get(&project_id)
        .ok_or_else(|| IpcError::new("E_PATH_NOT_FOUND", "project is not currently open"))?;
    Ok(PathBuf::from(&entry.project.root_path))
}

#[tauri::command]
pub async fn tasks_scan_files(
    project_id: Uuid,
    state: State<'_, AppState>,
) -> IpcResult<Vec<TaskFileCandidate>> {
    let root = project_root(project_id, &state)?;
    scan(&root).map_err(IpcError::from)
}

#[tauri::command]
pub async fn tasks_parse_file(
    project_id: Uuid,
    relative_path: String,
    state: State<'_, AppState>,
) -> IpcResult<TasksParseResponse> {
    let root = project_root(project_id, &state)?;
    let absolute = resolve_under_root(&root, &relative_path).map_err(IpcError::from)?;
    let format = detect_format(&absolute).ok_or_else(|| {
        IpcError::from(SpecLensError::TaskParseFailed(format!(
            "{relative_path}: unsupported extension"
        )))
    })?;
    let contents = fs::read_to_string(&absolute)
        .map_err(|e| IpcError::from(SpecLensError::io(absolute.display().to_string(), e)))?;
    let (file, entries) = parse(&relative_path, format, &contents).map_err(IpcError::from)?;
    Ok(TasksParseResponse { file, entries })
}

#[tauri::command]
pub async fn tasks_state_set_selected(
    project_id: Uuid,
    selected_paths: Vec<String>,
    active_path: Option<String>,
    state: State<'_, AppState>,
) -> IpcResult<ProjectState> {
    // FR-050 validation: `activePath` MUST appear in `selectedPaths`.
    if let Some(active) = active_path.as_ref() {
        if !selected_paths.iter().any(|p| p == active) {
            return Err(IpcError::new(
                "E_CONFIG_INVALID",
                format!("activePath '{active}' not in selectedPaths"),
            )
            .with_hint("Select the path before marking it active."));
        }
    }

    let mut open = state
        .open_projects
        .lock()
        .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
    let entry = open
        .get_mut(&project_id)
        .ok_or_else(|| IpcError::new("E_PATH_NOT_FOUND", "project is not currently open"))?;

    let mut next_state = entry.state_store.load_state().map_err(IpcError::from)?;
    next_state.selected_task_file_paths = selected_paths;
    next_state.active_task_file_path = active_path;
    next_state.normalize();
    entry
        .state_store
        .save_state(&next_state)
        .map_err(IpcError::from)?;
    entry.project.state = next_state.clone();
    Ok(next_state)
}
