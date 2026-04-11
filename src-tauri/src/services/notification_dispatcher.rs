//! Pure-logic helpers behind the `notify_system` command (Phase 9 US7 / FR-070).
//!
//! The Tauri command itself is a thin wrapper that calls
//! [`tauri_plugin_notification`] to surface an OS-native toast. Everything
//! that does NOT require an `AppHandle` lives here so it can be unit-tested
//! without spinning up the Tauri runtime — same pattern as
//! [`crate::services::version_checker`] for `env_check`.
//!
//! Responsibilities:
//!   - Parse / validate the inbound `{ title, body, level }` payload.
//!   - Trim whitespace, enforce non-empty title/body, cap their length so
//!     a runaway log line cannot blast a 64 KiB OS notification.
//!   - Map our `NotifyLevel` enum to a stable string id the frontend can
//!     use for icon/color routing in the in-app fallback.
//!   - Decide whether the dispatch should actually fire given the current
//!     `notifications_enabled` toggle.

use serde::{Deserialize, Serialize};

use crate::error::{IpcError, IpcResult};

/// Hard cap for the title field. Most OS notification centres truncate
/// well below this, but we keep our own limit so the frontend can rely on
/// a contract instead of OS-specific quirks.
pub const TITLE_MAX_LEN: usize = 120;

/// Hard cap for the body field. Linux notify-osd ignores anything past
/// ~150 chars, macOS keeps ~200, Windows toast keeps ~3 lines (~250). We
/// pick a single ceiling and document it in `contracts/ipc.md §I`.
pub const BODY_MAX_LEN: usize = 480;

/// Severity dial for [`NotifyRequest`]. Mirrors the wire enum in
/// `contracts/ipc.md §I` (`info | warn | error`). `info` is the default
/// when the field is omitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NotifyLevel {
    #[default]
    Info,
    Warn,
    Error,
}

impl NotifyLevel {
    /// Stable lowercase id used for icon mapping in the frontend and for
    /// log scopes in the backend.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

/// Wire payload for `notify_system`. Kept separate from
/// [`PreparedNotification`] so we can unit-test the parse → validate →
/// prepare pipeline without touching the plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotifyRequest {
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub level: NotifyLevel,
}

/// Result of [`prepare`]: a sanitized payload ready to be handed to
/// `tauri_plugin_notification` (or to the in-app fallback toast). All
/// strings here are guaranteed non-empty and under their max length.
///
/// Serialized back across IPC so the frontend in-app toast can use the
/// already-truncated values without recomputing them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedNotification {
    pub title: String,
    pub body: String,
    pub level: NotifyLevel,
}

impl PreparedNotification {
    /// Stable per-level identifier the frontend uses to pick an icon /
    /// colour for the in-app toast fallback.
    pub fn icon_key(&self) -> &'static str {
        match self.level {
            NotifyLevel::Info => "notify-info",
            NotifyLevel::Warn => "notify-warn",
            NotifyLevel::Error => "notify-error",
        }
    }
}

/// Validate + sanitize an incoming request. Returns a canonical
/// [`PreparedNotification`] or an [`IpcError`] with the
/// `E_VALIDATION` code so the frontend can react without parsing the
/// human-readable message.
pub fn prepare(req: NotifyRequest) -> IpcResult<PreparedNotification> {
    let title = req.title.trim();
    if title.is_empty() {
        return Err(
            IpcError::new("E_VALIDATION", "notify_system: title must not be empty")
                .with_hint("Pass a short, non-empty title."),
        );
    }
    let body = req.body.trim();
    if body.is_empty() {
        return Err(
            IpcError::new("E_VALIDATION", "notify_system: body must not be empty")
                .with_hint("Pass a non-empty body string."),
        );
    }
    let title = truncate(title, TITLE_MAX_LEN);
    let body = truncate(body, BODY_MAX_LEN);
    Ok(PreparedNotification {
        title,
        body,
        level: req.level,
    })
}

/// Pure decision helper: should `notify_system` actually fire given the
/// current `notifications_enabled` toggle? Extracted so the dispatch
/// branch in [`crate::commands::config::notify_system`] stays trivial and
/// so the gating rule is unit-testable.
pub fn should_dispatch(notifications_enabled: bool) -> bool {
    notifications_enabled
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    // Char-boundary safe truncation — keep first `max - 1` chars then add
    // a single ellipsis so the user knows something was clipped.
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn req(title: &str, body: &str, level: NotifyLevel) -> NotifyRequest {
        NotifyRequest {
            title: title.to_string(),
            body: body.to_string(),
            level,
        }
    }

    #[test]
    fn level_default_is_info() {
        let r: NotifyRequest = serde_json::from_str(r#"{"title":"x","body":"y"}"#).unwrap();
        assert_eq!(r.level, NotifyLevel::Info);
    }

    #[test]
    fn level_serde_round_trip_uses_lowercase() {
        for (lvl, wire) in [
            (NotifyLevel::Info, "info"),
            (NotifyLevel::Warn, "warn"),
            (NotifyLevel::Error, "error"),
        ] {
            let json = serde_json::to_string(&lvl).unwrap();
            assert_eq!(json, format!("\"{wire}\""));
            assert_eq!(lvl.as_str(), wire);
        }
    }

    #[test]
    fn prepare_trims_and_passes_through_short_payload() {
        let prepared = prepare(req("  hello  ", "  world  ", NotifyLevel::Info)).unwrap();
        assert_eq!(prepared.title, "hello");
        assert_eq!(prepared.body, "world");
        assert_eq!(prepared.level, NotifyLevel::Info);
        assert_eq!(prepared.icon_key(), "notify-info");
    }

    #[test]
    fn prepare_rejects_empty_title() {
        let err = prepare(req("   ", "body", NotifyLevel::Warn)).unwrap_err();
        assert_eq!(err.code, "E_VALIDATION");
        assert!(err.message.contains("title"));
    }

    #[test]
    fn prepare_rejects_empty_body() {
        let err = prepare(req("title", "", NotifyLevel::Error)).unwrap_err();
        assert_eq!(err.code, "E_VALIDATION");
        assert!(err.message.contains("body"));
    }

    #[test]
    fn prepare_truncates_oversized_title_and_body_with_ellipsis() {
        let big_title = "T".repeat(TITLE_MAX_LEN + 50);
        let big_body = "B".repeat(BODY_MAX_LEN + 200);
        let prepared = prepare(req(&big_title, &big_body, NotifyLevel::Warn)).unwrap();
        assert_eq!(prepared.title.chars().count(), TITLE_MAX_LEN);
        assert_eq!(prepared.body.chars().count(), BODY_MAX_LEN);
        assert!(prepared.title.ends_with('…'));
        assert!(prepared.body.ends_with('…'));
    }

    #[test]
    fn prepare_keeps_unicode_boundaries_intact() {
        // 4-byte chars count as 1 char each — must not panic on slice.
        let title: String = "🎉".repeat(TITLE_MAX_LEN + 5);
        let prepared = prepare(req(&title, "body", NotifyLevel::Info)).unwrap();
        assert_eq!(prepared.title.chars().count(), TITLE_MAX_LEN);
        assert!(prepared.title.ends_with('…'));
    }

    #[test]
    fn icon_key_maps_each_level() {
        for (lvl, key) in [
            (NotifyLevel::Info, "notify-info"),
            (NotifyLevel::Warn, "notify-warn"),
            (NotifyLevel::Error, "notify-error"),
        ] {
            let p = PreparedNotification {
                title: "t".into(),
                body: "b".into(),
                level: lvl,
            };
            assert_eq!(p.icon_key(), key);
        }
    }

    #[test]
    fn should_dispatch_respects_toggle() {
        assert!(should_dispatch(true));
        assert!(!should_dispatch(false));
    }
}
