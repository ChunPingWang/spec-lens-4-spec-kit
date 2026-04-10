//! `terminal.*` PTY commands (full impl in Phase 6 US4).

use crate::error::{IpcError, IpcResult};
use crate::models::{OutputLine, PtySessionDescriptor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSliceResponse {
    pub lines: Vec<OutputLine>,
    pub from_seq: u64,
    pub to_seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSearchResponse {
    pub matches: Vec<TerminalSearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSearchHit {
    pub seq: u64,
    pub start: u32,
    pub end: u32,
}

#[tauri::command]
pub async fn terminal_attach(_project_id: String) -> IpcResult<PtySessionDescriptor> {
    Err(IpcError::new(
        "E_INTERNAL",
        "terminal.attach not yet implemented (Phase 6 US4)",
    ))
}

#[tauri::command]
pub async fn terminal_detach(_session_id: String) -> IpcResult<()> {
    Err(IpcError::new(
        "E_INTERNAL",
        "terminal.detach not yet implemented (Phase 6 US4)",
    ))
}

#[tauri::command]
pub async fn terminal_write(_session_id: String, _data: String) -> IpcResult<()> {
    Err(IpcError::new(
        "E_INTERNAL",
        "terminal.write not yet implemented (Phase 6 US4)",
    ))
}

#[tauri::command]
pub async fn terminal_resize(_session_id: String, _cols: u16, _rows: u16) -> IpcResult<()> {
    Err(IpcError::new(
        "E_INTERNAL",
        "terminal.resize not yet implemented (Phase 6 US4)",
    ))
}

#[tauri::command]
pub async fn terminal_get_slice(
    _session_id: String,
    _from_seq: u64,
    _limit: u32,
) -> IpcResult<TerminalSliceResponse> {
    Err(IpcError::new(
        "E_INTERNAL",
        "terminal.get_slice not yet implemented (Phase 6 US4)",
    ))
}

#[tauri::command]
pub async fn terminal_search(
    _session_id: String,
    _pattern: String,
    _case_sensitive: bool,
    _is_regex: bool,
) -> IpcResult<TerminalSearchResponse> {
    Err(IpcError::new(
        "E_INTERNAL",
        "terminal.search not yet implemented (Phase 6 US4)",
    ))
}
