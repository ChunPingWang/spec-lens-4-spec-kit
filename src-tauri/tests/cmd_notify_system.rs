//! T145 — contract tests for the `notify_system` command (Phase 9 US7).
//!
//! Following the same pattern as `env_check_flow.rs`: the Tauri command is
//! a thin wrapper over `notification_dispatcher::prepare` +
//! `tauri-plugin-notification`. Pure functions go through here so we can
//! cover info / warn / error and the toggle gating without spinning up the
//! Tauri runtime.
//!
//! Pairs with the `AppDataStore` round-trip to validate that
//! `config_set_notifications_enabled` is the only state surface
//! `notify_system` reads from when deciding whether to dispatch.

use speclens_lib::models::AppConfig;
use speclens_lib::services::app_data_store::AppDataStore;
use speclens_lib::services::notification_dispatcher::{
    prepare, should_dispatch, NotifyLevel, NotifyRequest, BODY_MAX_LEN, TITLE_MAX_LEN,
};
use tempfile::TempDir;

fn req(title: &str, body: &str, level: NotifyLevel) -> NotifyRequest {
    NotifyRequest {
        title: title.to_string(),
        body: body.to_string(),
        level,
    }
}

// ----- T145 a/b/c: info / warn / error levels round-trip cleanly --------

#[test]
fn notify_system_prepares_info_payload() {
    let prepared = prepare(req(
        "Spec-Kit step done",
        "T101 finished successfully.",
        NotifyLevel::Info,
    ))
    .expect("prepare info");
    assert_eq!(prepared.level, NotifyLevel::Info);
    assert_eq!(prepared.title, "Spec-Kit step done");
    assert_eq!(prepared.body, "T101 finished successfully.");
    assert_eq!(prepared.icon_key(), "notify-info");
}

#[test]
fn notify_system_prepares_warn_payload() {
    let prepared = prepare(req(
        "Spec-Kit warning",
        "Step T102 finished with warnings.",
        NotifyLevel::Warn,
    ))
    .expect("prepare warn");
    assert_eq!(prepared.level, NotifyLevel::Warn);
    assert_eq!(prepared.icon_key(), "notify-warn");
}

#[test]
fn notify_system_prepares_error_payload() {
    let prepared = prepare(req(
        "Spec-Kit step failed",
        "Step T103 failed: see terminal output for details.",
        NotifyLevel::Error,
    ))
    .expect("prepare error");
    assert_eq!(prepared.level, NotifyLevel::Error);
    assert_eq!(prepared.icon_key(), "notify-error");
}

// ----- Validation surface: bad payloads must NOT reach the OS plugin ----

#[test]
fn notify_system_rejects_blank_title() {
    let err = prepare(req("   ", "body", NotifyLevel::Info)).unwrap_err();
    assert_eq!(err.code, "E_VALIDATION");
}

#[test]
fn notify_system_rejects_blank_body() {
    let err = prepare(req("title", "\n\t  ", NotifyLevel::Info)).unwrap_err();
    assert_eq!(err.code, "E_VALIDATION");
}

#[test]
fn notify_system_truncates_oversized_payload() {
    let big = "x".repeat(TITLE_MAX_LEN * 3);
    let bigger = "y".repeat(BODY_MAX_LEN * 3);
    let prepared = prepare(req(&big, &bigger, NotifyLevel::Warn)).unwrap();
    assert_eq!(prepared.title.chars().count(), TITLE_MAX_LEN);
    assert_eq!(prepared.body.chars().count(), BODY_MAX_LEN);
}

// ----- Default level falls through as info ------------------------------

#[test]
fn notify_system_request_level_defaults_to_info_when_omitted() {
    let parsed: NotifyRequest =
        serde_json::from_str(r#"{"title":"hello","body":"world"}"#).expect("parse");
    let prepared = prepare(parsed).expect("prepare default");
    assert_eq!(prepared.level, NotifyLevel::Info);
}

// ----- Toggle gating: should_dispatch follows AppConfig ------------------

#[test]
fn notify_system_dispatch_gating_follows_app_config_toggle() {
    let tmp = TempDir::new().expect("tempdir");
    let store = AppDataStore::new(tmp.path().to_path_buf());

    // Default config has notifications_enabled = true.
    let initial = store.load().expect("load default");
    assert!(initial.notifications_enabled);
    assert!(should_dispatch(initial.notifications_enabled));

    // Flip the toggle off and persist; reloaded config must reflect it
    // and `should_dispatch` must mirror the same value.
    let updated = store
        .mutate(|cfg: &mut AppConfig| {
            cfg.notifications_enabled = false;
            Ok(())
        })
        .expect("mutate");
    assert!(!updated.notifications_enabled);
    assert!(!should_dispatch(updated.notifications_enabled));

    let reloaded = store.load().expect("reload");
    assert!(!reloaded.notifications_enabled);
}
