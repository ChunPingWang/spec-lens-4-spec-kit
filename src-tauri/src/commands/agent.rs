//! `agent.*` IPC commands. Phase 8 US6 implements the real detector
//! plus the static "known agents" catalog backing the settings UI.

use std::path::PathBuf;

use tauri::State;
use uuid::Uuid;

use crate::error::{IpcError, IpcResult};
use crate::models::{AgentId, AgentProfile};
use crate::services::agent_detector;
use crate::state::AppState;

#[tauri::command]
pub async fn agent_detect(project_id: Uuid, state: State<'_, AppState>) -> IpcResult<AgentProfile> {
    // Resolve the project's root from the open-projects table so this
    // command works without a separate `path` parameter — the frontend
    // already knows the project id.
    let root: PathBuf = {
        let open = state
            .open_projects
            .lock()
            .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
        let entry = open.get(&project_id).ok_or_else(|| {
            IpcError::new("E_PATH_NOT_FOUND", "project is not currently open")
                .with_hint("Open the project first with project.open.")
        })?;
        PathBuf::from(&entry.project.root_path)
    };

    let profile = agent_detector::detect(&root).map_err(IpcError::from)?;

    // Cache on the open project so the TopBar / banners always read the
    // freshest result without re-running the detector.
    {
        let mut open = state
            .open_projects
            .lock()
            .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
        if let Some(entry) = open.get_mut(&project_id) {
            entry.project.agent = profile.clone();
        }
    }

    Ok(profile)
}

#[tauri::command]
pub async fn agent_list_known() -> IpcResult<Vec<AgentProfile>> {
    Ok(AgentId::all_known()
        .iter()
        .map(|id| agent_detector::build_profile(*id, crate::models::AgentDetectionSource::None))
        .collect())
}
