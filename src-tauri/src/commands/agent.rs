//! `agent.*` detection commands (full impl in Phase 7 US5).

use crate::error::{IpcError, IpcResult};
use crate::models::{AgentId, AgentProfile};

#[tauri::command]
pub async fn agent_detect(_project_id: String) -> IpcResult<AgentProfile> {
    Err(IpcError::new(
        "E_INTERNAL",
        "agent.detect not yet implemented (Phase 7 US5)",
    ))
}

#[tauri::command]
pub async fn agent_list_known() -> IpcResult<Vec<AgentId>> {
    Ok(AgentId::all_known().to_vec())
}
