//! `terminal.*` PTY commands. See `contracts/ipc.md §terminal` and US3.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::error::{IpcError, IpcResult};
use crate::models::{OutputLine, PtySessionDescriptor};
use crate::services::output_buffer::{validate_buffer_max_lines, validate_disk_cap_mib};
use crate::services::TerminalConfig;
use crate::state::AppState;

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
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalConfigResponse {
    pub buffer_max_lines: usize,
    pub disk_cap_mi_b: u64,
}

impl From<TerminalConfig> for TerminalConfigResponse {
    fn from(cfg: TerminalConfig) -> Self {
        Self {
            buffer_max_lines: cfg.buffer_max_lines,
            disk_cap_mi_b: cfg.disk_cap_mib,
        }
    }
}

fn parse_uuid(raw: &str, hint: &str) -> IpcResult<Uuid> {
    Uuid::parse_str(raw).map_err(|_| {
        IpcError::new("E_INTERNAL", format!("invalid uuid: {raw}")).with_hint(hint.to_string())
    })
}

fn project_cwd(state: &AppState, project_id: &Uuid) -> IpcResult<PathBuf> {
    let map = state
        .open_projects
        .lock()
        .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
    let entry = map.get(project_id).ok_or_else(|| {
        IpcError::new("E_PATH_NOT_FOUND", "project is not currently open")
            .with_hint("Open the project first with project.open.")
    })?;
    Ok(PathBuf::from(&entry.project.root_path))
}

#[tauri::command]
pub async fn terminal_attach(
    project_id: String,
    window_id: Option<String>,
    state: State<'_, AppState>,
) -> IpcResult<PtySessionDescriptor> {
    let project_id = parse_uuid(
        &project_id,
        "Provide the project UUID returned by project.open.",
    )?;
    // FR-085 / NFR-012: PTY cwd is locked to the project root; we never
    // honour a caller-supplied cwd override.
    let cwd = project_cwd(&state, &project_id)?;
    let window_id = window_id.unwrap_or_else(|| "default".to_string());
    let tx = state.terminal_event_tx.clone();
    let session = state
        .terminal
        .attach(
            project_id,
            window_id,
            cwd,
            None,
            TerminalConfig::default(),
            tx,
        )
        .map_err(IpcError::from)?;
    Ok(session.descriptor.clone())
}

#[tauri::command]
pub async fn terminal_detach(session_id: String, state: State<'_, AppState>) -> IpcResult<()> {
    let session_id = parse_uuid(
        &session_id,
        "Use the sessionId returned by terminal.attach.",
    )?;
    state.terminal.remove(&session_id).map_err(IpcError::from)?;
    Ok(())
}

#[tauri::command]
pub async fn terminal_write(
    session_id: String,
    data: String,
    state: State<'_, AppState>,
) -> IpcResult<()> {
    let session_id = parse_uuid(
        &session_id,
        "Use the sessionId returned by terminal.attach.",
    )?;
    let session = state.terminal.get(&session_id).map_err(IpcError::from)?;
    session.write(data.as_bytes()).map_err(IpcError::from)
}

#[tauri::command]
pub async fn terminal_resize(
    session_id: String,
    cols: u16,
    rows: u16,
    state: State<'_, AppState>,
) -> IpcResult<()> {
    let session_id = parse_uuid(
        &session_id,
        "Use the sessionId returned by terminal.attach.",
    )?;
    let session = state.terminal.get(&session_id).map_err(IpcError::from)?;
    session.resize(cols, rows).map_err(IpcError::from)
}

#[tauri::command]
pub async fn terminal_get_slice(
    session_id: String,
    from_seq: u64,
    limit: u32,
    state: State<'_, AppState>,
) -> IpcResult<TerminalSliceResponse> {
    let session_id = parse_uuid(
        &session_id,
        "Use the sessionId returned by terminal.attach.",
    )?;
    let session = state.terminal.get(&session_id).map_err(IpcError::from)?;
    let buffer = session
        .buffer
        .lock()
        .map_err(|_| IpcError::new("E_INTERNAL", "output buffer mutex poisoned"))?;
    let to_seq = from_seq.saturating_add(limit as u64);
    let lines = buffer.slice(from_seq, to_seq).map_err(IpcError::from)?;
    let actual_to = lines.last().map(|l| l.seq + 1).unwrap_or(from_seq);
    Ok(TerminalSliceResponse {
        lines,
        from_seq,
        to_seq: actual_to,
    })
}

#[tauri::command]
pub async fn terminal_search(
    session_id: String,
    pattern: String,
    case_sensitive: bool,
    _is_regex: bool,
    state: State<'_, AppState>,
) -> IpcResult<TerminalSearchResponse> {
    let session_id = parse_uuid(
        &session_id,
        "Use the sessionId returned by terminal.attach.",
    )?;
    let session = state.terminal.get(&session_id).map_err(IpcError::from)?;
    let buffer = session
        .buffer
        .lock()
        .map_err(|_| IpcError::new("E_INTERNAL", "output buffer mutex poisoned"))?;
    // Plain substring search only for US3 MVP; regex toggle is accepted but
    // currently behaves as plain search. Full regex lands with T096 polish.
    let hits = buffer
        .search_plain(&pattern, case_sensitive)
        .map_err(IpcError::from)?;
    let matches = hits
        .into_iter()
        .map(|line| {
            let needle = if case_sensitive {
                line.text.find(&pattern).map(|i| (i, i + pattern.len()))
            } else {
                line.text
                    .to_lowercase()
                    .find(&pattern.to_lowercase())
                    .map(|i| (i, i + pattern.len()))
            };
            let (start, end) = needle.unwrap_or((0, 0));
            TerminalSearchHit {
                seq: line.seq,
                start: start as u32,
                end: end as u32,
                text: line.text,
            }
        })
        .collect();
    Ok(TerminalSearchResponse { matches })
}

#[tauri::command]
pub async fn terminal_get_config(
    session_id: String,
    state: State<'_, AppState>,
) -> IpcResult<TerminalConfigResponse> {
    let session_id = parse_uuid(
        &session_id,
        "Use the sessionId returned by terminal.attach.",
    )?;
    let session = state.terminal.get(&session_id).map_err(IpcError::from)?;
    Ok(session.config.into())
}

#[tauri::command]
pub async fn terminal_set_config(
    _session_id: String,
    buffer_max_lines: usize,
    disk_cap_mi_b: u64,
) -> IpcResult<TerminalConfigResponse> {
    // Validate against FR-048 bounds. Mutation requires recreating the
    // session buffer, which is out of scope for the US3 MVP; for now we
    // validate + echo so callers get the same config-invalid feedback.
    validate_buffer_max_lines(buffer_max_lines).map_err(IpcError::from)?;
    validate_disk_cap_mib(disk_cap_mi_b).map_err(IpcError::from)?;
    Ok(TerminalConfigResponse {
        buffer_max_lines,
        disk_cap_mi_b,
    })
}
