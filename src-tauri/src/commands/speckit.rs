//! `speckit.*` step commands plus the US5 environment-check commands.
//! Phase 3 US1 reads steps from the open-projects table; Phase 7 US5 adds
//! `env_check` and `env_get_install_guide` backed by
//! [`crate::services::version_checker`].

use tauri::State;
use uuid::Uuid;

use crate::error::{IpcError, IpcResult};
use crate::models::{EnvironmentStatus, Step};
use crate::services::version_checker::{
    build_status, install_guide as build_install_guide, InstallGuide, SystemProbe, VersionProbe,
};
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

#[tauri::command]
pub async fn env_check(
    project_id: Uuid,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> IpcResult<EnvironmentStatus> {
    // Short-circuit the in-memory cache unless `force` is set — repeated
    // opens of the same project should not re-spawn the probe.
    if !force.unwrap_or(false) {
        let open = state
            .open_projects
            .lock()
            .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
        if let Some(entry) = open.get(&project_id) {
            if entry.project.environment.speckit_installed
                || !entry.project.environment.issues.is_empty()
            {
                return Ok(entry.project.environment.clone());
            }
        }
    }

    // Run the probe on the calling (async) thread — it's a short-lived
    // process invocation and staying off the main thread keeps the UI
    // responsive.
    let probe = SystemProbe;
    let outcome = probe.probe();
    let status = build_status(outcome, None).map_err(IpcError::from)?;

    // Write-through into the open project so subsequent calls hit the
    // cache and so TopBar / banners can read the latest snapshot.
    {
        let mut open = state
            .open_projects
            .lock()
            .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
        if let Some(entry) = open.get_mut(&project_id) {
            entry.project.environment = status.clone();
        }
    }

    Ok(status)
}

#[tauri::command]
pub async fn env_get_install_guide(platform: Option<String>) -> IpcResult<InstallGuide> {
    let platform = platform.unwrap_or_else(|| {
        if cfg!(target_os = "macos") {
            "macos".to_string()
        } else if cfg!(target_os = "windows") {
            "windows".to_string()
        } else {
            "linux".to_string()
        }
    });
    Ok(build_install_guide(&platform))
}
