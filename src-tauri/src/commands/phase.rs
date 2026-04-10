//! `phase.*` document commands (full impl in Phase 4 US2).

use crate::error::{IpcError, IpcResult};
use crate::models::PhaseDocument;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseOverviewResponse {
    pub summary: String,
    pub documents: Vec<PhaseDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseDocumentReadResponse {
    pub relative_path: String,
    pub content: String,
    pub truncated: bool,
}

#[tauri::command]
pub async fn phase_overview(
    _project_id: String,
    _step_id: String,
) -> IpcResult<PhaseOverviewResponse> {
    Err(IpcError::new(
        "E_INTERNAL",
        "phase.overview not yet implemented (Phase 4 US2)",
    ))
}

#[tauri::command]
pub async fn phase_documents_list(
    _project_id: String,
    _step_id: String,
) -> IpcResult<Vec<PhaseDocument>> {
    Err(IpcError::new(
        "E_INTERNAL",
        "phase.documents_list not yet implemented (Phase 4 US2)",
    ))
}

#[tauri::command]
pub async fn phase_document_read(
    _project_id: String,
    _relative_path: String,
) -> IpcResult<PhaseDocumentReadResponse> {
    Err(IpcError::new(
        "E_INTERNAL",
        "phase.document_read not yet implemented (Phase 4 US2)",
    ))
}
