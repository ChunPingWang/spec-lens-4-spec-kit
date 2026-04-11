//! `config.*` commands plus the Phase 9 US7 `notify_system` command.
//!
//! All handlers are thin: validation/sanitization lives in
//! [`crate::services::notification_dispatcher`] and disk persistence lives
//! in [`crate::services::AppDataStore`]. The Tauri layer here only stitches
//! state, surfaces typed errors, and (for `notify_system`) talks to
//! `tauri-plugin-notification`.

use tauri::{AppHandle, Runtime, State};
use tauri_plugin_notification::NotificationExt;
use tracing::{debug, warn};

use crate::error::{IpcError, IpcResult};
use crate::models::{
    AppConfig, LanguagePreference, ThemePreference, BUFFER_MAX_LINES_RANGE, DISK_CAP_MIB_RANGE,
};
use crate::services::notification_dispatcher::{
    prepare, should_dispatch, NotifyRequest, PreparedNotification,
};
use crate::state::AppState;

#[tauri::command]
pub async fn config_get(state: State<'_, AppState>) -> IpcResult<AppConfig> {
    state.app_data.load().map_err(IpcError::from)
}

#[tauri::command]
pub async fn config_set_language(
    language: LanguagePreference,
    state: State<'_, AppState>,
) -> IpcResult<AppConfig> {
    state
        .app_data
        .mutate(|cfg| {
            cfg.language = language;
            cfg.last_updated_at = chrono::Utc::now();
            Ok(())
        })
        .map_err(IpcError::from)
}

#[tauri::command]
pub async fn config_set_theme(
    theme: ThemePreference,
    state: State<'_, AppState>,
) -> IpcResult<AppConfig> {
    state
        .app_data
        .mutate(|cfg| {
            cfg.theme = theme;
            cfg.last_updated_at = chrono::Utc::now();
            Ok(())
        })
        .map_err(IpcError::from)
}

#[tauri::command]
pub async fn config_set_terminal_limits(
    buffer_max_lines: Option<u32>,
    disk_cap_mib: Option<u32>,
    state: State<'_, AppState>,
) -> IpcResult<AppConfig> {
    // Validate against the model-level ranges before mutating so the
    // store's own validation pass cannot leave the on-disk file in a
    // half-updated state.
    if let Some(lines) = buffer_max_lines {
        let (lo, hi) = BUFFER_MAX_LINES_RANGE;
        if !(lo..=hi).contains(&lines) {
            return Err(IpcError::new(
                "E_VALIDATION",
                format!("bufferMaxLines must be in [{lo},{hi}]"),
            ));
        }
    }
    if let Some(cap) = disk_cap_mib {
        let (lo, hi) = DISK_CAP_MIB_RANGE;
        if !(lo..=hi).contains(&cap) {
            return Err(IpcError::new(
                "E_VALIDATION",
                format!("diskCapMiB must be in [{lo},{hi}]"),
            ));
        }
    }
    state
        .app_data
        .mutate(|cfg| {
            if let Some(lines) = buffer_max_lines {
                cfg.terminal_buffer_max_lines = lines;
            }
            if let Some(cap) = disk_cap_mib {
                cfg.terminal_disk_cap_mib = cap;
            }
            cfg.last_updated_at = chrono::Utc::now();
            Ok(())
        })
        .map_err(IpcError::from)
}

#[tauri::command]
pub async fn config_set_notifications_enabled(
    enabled: bool,
    state: State<'_, AppState>,
) -> IpcResult<AppConfig> {
    state
        .app_data
        .mutate(|cfg| {
            cfg.notifications_enabled = enabled;
            cfg.last_updated_at = chrono::Utc::now();
            Ok(())
        })
        .map_err(IpcError::from)
}

/// Phase 9 US7 — surface a native OS notification (FR-070).
///
/// The frontend calls this when a Spec-Kit step transitions to `done` /
/// `failed` while the window is unfocused. Honors the user's
/// `notificationsEnabled` toggle: if disabled, returns success without
/// firing so the call site does not have to special-case the toggle.
#[tauri::command]
pub async fn notify_system<R: Runtime>(
    request: NotifyRequest,
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> IpcResult<PreparedNotification> {
    let prepared = prepare(request)?;

    let cfg = state.app_data.load().map_err(IpcError::from)?;
    if !should_dispatch(cfg.notifications_enabled) {
        debug!(
            level = prepared.level.as_str(),
            "notify_system: notifications disabled, skipping dispatch"
        );
        return Ok(prepared);
    }

    let res = app
        .notification()
        .builder()
        .title(prepared.title.clone())
        .body(prepared.body.clone())
        .show();

    if let Err(err) = res {
        // FR-013-style fallback: never lock the user out — log and let the
        // frontend in-app toast pick up the prepared payload from the
        // returned value.
        warn!(error = %err, "notify_system: OS notification failed; returning prepared payload");
    }

    Ok(prepared)
}
