//! `config.*` commands (full impl in Phase 8 US6).

use crate::error::{IpcError, IpcResult};
use crate::models::{AppConfig, LanguagePreference, ThemePreference};

#[tauri::command]
pub async fn config_get() -> IpcResult<AppConfig> {
    Err(IpcError::new(
        "E_INTERNAL",
        "config.get not yet implemented (Phase 8 US6)",
    ))
}

#[tauri::command]
pub async fn config_set_language(_language: LanguagePreference) -> IpcResult<AppConfig> {
    Err(IpcError::new(
        "E_INTERNAL",
        "config.set_language not yet implemented (Phase 8 US6)",
    ))
}

#[tauri::command]
pub async fn config_set_theme(_theme: ThemePreference) -> IpcResult<AppConfig> {
    Err(IpcError::new(
        "E_INTERNAL",
        "config.set_theme not yet implemented (Phase 8 US6)",
    ))
}

#[tauri::command]
pub async fn config_set_terminal_limits(
    _buffer_max_lines: u32,
    _disk_cap_mib: u32,
) -> IpcResult<AppConfig> {
    Err(IpcError::new(
        "E_INTERNAL",
        "config.set_terminal_limits not yet implemented (Phase 8 US6)",
    ))
}
