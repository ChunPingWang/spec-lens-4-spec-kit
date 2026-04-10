//! `tasks.*` commands (full impl in Phase 5 US3).

use crate::error::{IpcError, IpcResult};
use crate::models::{TaskEntry, TaskFile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksParseResponse {
    pub file: TaskFile,
    pub entries: Vec<TaskEntry>,
}

#[tauri::command]
pub async fn tasks_scan(_project_id: String) -> IpcResult<Vec<TaskFile>> {
    Err(IpcError::new(
        "E_INTERNAL",
        "tasks.scan not yet implemented (Phase 5 US3)",
    ))
}

#[tauri::command]
pub async fn tasks_parse(
    _project_id: String,
    _relative_path: String,
) -> IpcResult<TasksParseResponse> {
    Err(IpcError::new(
        "E_INTERNAL",
        "tasks.parse not yet implemented (Phase 5 US3)",
    ))
}

#[tauri::command]
pub async fn tasks_pick_files(_project_id: String) -> IpcResult<Vec<String>> {
    Err(IpcError::new(
        "E_INTERNAL",
        "tasks.pick_files not yet implemented (Phase 5 US3)",
    ))
}

#[tauri::command]
pub async fn tasks_set_selected(
    _project_id: String,
    _selected_paths: Vec<String>,
    _active_path: Option<String>,
) -> IpcResult<()> {
    Err(IpcError::new(
        "E_INTERNAL",
        "tasks.set_selected not yet implemented (Phase 5 US3)",
    ))
}
