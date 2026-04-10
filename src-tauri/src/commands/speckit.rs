//! `speckit.*` step commands. Phase 3 US1 reads steps from the open-projects
//! table. Real disk scans land in Phase 4.

use tauri::State;
use uuid::Uuid;

use crate::error::{IpcError, IpcResult};
use crate::models::Step;
use crate::state::AppState;

#[tauri::command]
pub async fn steps_list(project_id: Uuid, state: State<'_, AppState>) -> IpcResult<Vec<Step>> {
    let open = state
        .open_projects
        .lock()
        .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
    let entry = open.get(&project_id).ok_or_else(|| {
        IpcError::new("E_PATH_NOT_FOUND", "project is not currently open")
            .with_hint("Open the project first with project.open.")
    })?;
    Ok(entry.project.steps.clone())
}

#[tauri::command]
pub async fn steps_get(
    project_id: Uuid,
    step_id: String,
    state: State<'_, AppState>,
) -> IpcResult<Step> {
    let open = state
        .open_projects
        .lock()
        .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
    let entry = open.get(&project_id).ok_or_else(|| {
        IpcError::new("E_PATH_NOT_FOUND", "project is not currently open")
            .with_hint("Open the project first with project.open.")
    })?;
    entry
        .project
        .steps
        .iter()
        .find(|s| s.id == step_id)
        .cloned()
        .ok_or_else(|| IpcError::new("E_STEP_NOT_FOUND", format!("unknown step id: {step_id}")))
}
