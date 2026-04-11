//! `phase.*` document commands (Phase 4 US2).
//!
//! These commands are read-only: they walk the project root, compute
//! document statuses using `.speclens/hashes.json`, and expose slices
//! for the Overview sub-page.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::error::{IpcError, IpcResult, SpecLensError};
use crate::models::{DocHashRecord, PhaseDocument};
use crate::services::doc_hash_store::{compute_sha256_hex, resolve_inside_root};
use crate::services::PhaseScanner;
use crate::state::AppState;

/// One lazy slice of a document rendered on the Overview sub-page.
/// Corresponds to `PhaseOverviewSlice` in contracts/ipc.md.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseOverviewSlice {
    pub title: String,
    pub relative_path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseOverviewResponse {
    pub step_id: String,
    pub slices: Vec<PhaseOverviewSlice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseDocumentReadResponse {
    pub relative_path: String,
    pub content: String,
    pub truncated: bool,
    pub hash: Option<DocHashRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseRecomputeResponse {
    pub updated: usize,
}

/// Soft cap to stop us from buffering gigantic files into memory. Callers
/// can override via `max_bytes` on `phase_document_read`.
const DEFAULT_READ_MAX_BYTES: u64 = 512 * 1024;
const HARD_READ_MAX_BYTES: u64 = 4 * 1024 * 1024;

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
pub async fn phase_documents_list(
    project_id: Uuid,
    step_id: String,
    state: State<'_, AppState>,
) -> IpcResult<Vec<PhaseDocument>> {
    let root = project_root(project_id, &state)?;
    // Pull up the saved hashes for this project so scan statuses reflect
    // the persisted baseline.
    let hashes = {
        let open = state
            .open_projects
            .lock()
            .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
        let entry = open
            .get(&project_id)
            .ok_or_else(|| IpcError::new("E_PATH_NOT_FOUND", "project is not currently open"))?;
        entry.state_store.load_hashes().unwrap_or_default()
    };
    PhaseScanner::scan_step_documents(&root, &step_id, &hashes).map_err(IpcError::from)
}

#[tauri::command]
pub async fn phase_overview(
    project_id: Uuid,
    step_id: String,
    state: State<'_, AppState>,
) -> IpcResult<PhaseOverviewResponse> {
    let root = project_root(project_id, &state)?;
    let hashes = {
        let open = state
            .open_projects
            .lock()
            .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
        let entry = open
            .get(&project_id)
            .ok_or_else(|| IpcError::new("E_PATH_NOT_FOUND", "project is not currently open"))?;
        entry.state_store.load_hashes().unwrap_or_default()
    };
    let docs =
        PhaseScanner::scan_step_documents(&root, &step_id, &hashes).map_err(IpcError::from)?;

    let mut slices = Vec::new();
    for doc in docs.iter().take(4) {
        let abs = root.join(&doc.relative_path);
        if !abs.exists() {
            continue;
        }
        let bytes = std::fs::read(&abs)
            .map_err(|e| SpecLensError::io(abs.display().to_string(), e))
            .map_err(IpcError::from)?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        // Extract the first H1/H2 block (up to 40 lines) as a cheap summary.
        let summary: String = text.lines().take(40).collect::<Vec<_>>().join("\n");
        slices.push(PhaseOverviewSlice {
            title: doc.display_name.clone(),
            relative_path: doc.relative_path.clone(),
            content: summary,
        });
    }
    Ok(PhaseOverviewResponse { step_id, slices })
}

#[tauri::command]
pub async fn phase_document_read(
    project_id: Uuid,
    relative_path: String,
    max_bytes: Option<u64>,
    state: State<'_, AppState>,
) -> IpcResult<PhaseDocumentReadResponse> {
    let root = project_root(project_id, &state)?;
    let cap = max_bytes
        .unwrap_or(DEFAULT_READ_MAX_BYTES)
        .min(HARD_READ_MAX_BYTES);
    let abs = resolve_inside_root(&root, &relative_path).map_err(IpcError::from)?;
    if !abs.exists() {
        return Err(IpcError::from(SpecLensError::DocNotFound(relative_path)));
    }
    let meta = std::fs::metadata(&abs)
        .map_err(|e| SpecLensError::io(abs.display().to_string(), e))
        .map_err(IpcError::from)?;
    let (bytes, truncated) = if meta.len() > cap {
        use std::io::Read;
        let mut f = std::fs::File::open(&abs)
            .map_err(|e| SpecLensError::io(abs.display().to_string(), e))
            .map_err(IpcError::from)?;
        let mut buf = vec![0u8; cap as usize];
        f.read_exact(&mut buf)
            .map_err(|e| SpecLensError::io(abs.display().to_string(), e))
            .map_err(IpcError::from)?;
        (buf, true)
    } else {
        (
            std::fs::read(&abs)
                .map_err(|e| SpecLensError::io(abs.display().to_string(), e))
                .map_err(IpcError::from)?,
            false,
        )
    };
    let content = String::from_utf8_lossy(&bytes).into_owned();

    // Look up the persisted hash (if any) so the UI can show verify state.
    let hash = {
        let open = state
            .open_projects
            .lock()
            .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
        let entry = open
            .get(&project_id)
            .ok_or_else(|| IpcError::new("E_PATH_NOT_FOUND", "project is not currently open"))?;
        entry
            .state_store
            .load_hashes()
            .ok()
            .and_then(|h| h.get(&relative_path).cloned())
    };

    Ok(PhaseDocumentReadResponse {
        relative_path,
        content,
        truncated,
        hash,
    })
}

#[tauri::command]
pub async fn phase_recompute_hashes(
    project_id: Uuid,
    step_id: Option<String>,
    state: State<'_, AppState>,
) -> IpcResult<PhaseRecomputeResponse> {
    let root = project_root(project_id, &state)?;
    let open = state
        .open_projects
        .lock()
        .map_err(|_| IpcError::new("E_INTERNAL", "open projects mutex poisoned"))?;
    let entry = open
        .get(&project_id)
        .ok_or_else(|| IpcError::new("E_PATH_NOT_FOUND", "project is not currently open"))?;

    // Walk the steps we care about (single step or all steps on the project).
    let step_ids: Vec<String> = match step_id {
        Some(id) => vec![id],
        None => entry.project.steps.iter().map(|s| s.id.clone()).collect(),
    };

    let mut hashes = entry.state_store.load_hashes().unwrap_or_default();
    let mut updated = 0usize;
    for sid in &step_ids {
        let docs =
            PhaseScanner::scan_step_documents(&root, sid, &hashes).map_err(IpcError::from)?;
        for doc in docs {
            let abs = root.join(&doc.relative_path);
            if !abs.exists() {
                continue;
            }
            let sha = compute_sha256_hex(&abs).map_err(IpcError::from)?;
            let previous = hashes.get(&doc.relative_path).map(|r| r.sha256.clone());
            if previous.as_deref() != Some(sha.as_str()) {
                updated += 1;
            }
            hashes.insert(
                doc.relative_path.clone(),
                DocHashRecord::new(doc.relative_path.clone(), sha),
            );
        }
    }
    entry
        .state_store
        .save_hashes(&hashes)
        .map_err(IpcError::from)?;
    Ok(PhaseRecomputeResponse { updated })
}
