//! Phase 7 US5 integration tests — exercise `version_checker` at the
//! service level the same way `tasks_flow.rs` exercises `tasks_parser`.
//! Covers T126 (three env scenarios) and T127 (install guide payload).
//!
//! The Tauri command wrapper is a thin async shim over
//! [`build_status`] + [`install_guide`], so testing the pure functions
//! provides equivalent coverage without spinning up a Tauri context.

use speclens_lib::models::{EnvBadgeColor, EnvIssueKind, EnvIssueSeverity};
use speclens_lib::services::version_checker::{
    build_status, install_guide, parse_semver, ProbeOutcome,
};

// ----- T126: three env_check scenarios ----------------------------------

#[test]
fn env_check_reports_missing_binary_as_red() {
    let status = build_status(ProbeOutcome::not_found(), Some("0.6.0")).expect("status");
    assert!(!status.speckit_installed);
    assert!(!status.update_available);
    assert_eq!(status.badge_color(), EnvBadgeColor::Red);
    assert_eq!(status.issues.len(), 1);
    assert_eq!(status.issues[0].kind, EnvIssueKind::MissingBinary);
    assert_eq!(status.issues[0].severity, EnvIssueSeverity::Error);
}

#[test]
fn env_check_reports_outdated_installation_as_amber() {
    let status =
        build_status(ProbeOutcome::installed("0.5.0", None), Some("0.6.0")).expect("status");
    assert!(status.speckit_installed);
    assert!(status.update_available);
    assert_eq!(status.badge_color(), EnvBadgeColor::Amber);
    assert!(status
        .issues
        .iter()
        .any(|i| i.kind == EnvIssueKind::VersionMismatch));
}

#[test]
fn env_check_reports_current_installation_as_green() {
    let status =
        build_status(ProbeOutcome::installed("0.6.0", None), Some("0.6.0")).expect("status");
    assert!(status.speckit_installed);
    assert!(!status.update_available);
    assert_eq!(status.badge_color(), EnvBadgeColor::Green);
    assert!(status.issues.is_empty());
    assert_eq!(status.speckit_version.as_deref(), Some("0.6.0"));
    assert_eq!(status.latest_known_version.as_deref(), Some("0.6.0"));
}

// FR-013: offline / partial-failure path must not lock the user out.
#[test]
fn env_check_offline_fallback_downgrades_to_amber_not_red() {
    let outcome = ProbeOutcome {
        version: Some("0.6.0".to_string()),
        path: None,
        error: Some("update registry unreachable".to_string()),
    };
    let status = build_status(outcome, Some("0.6.0")).expect("status");
    assert!(status.speckit_installed);
    assert_eq!(status.badge_color(), EnvBadgeColor::Amber);
    assert!(status
        .issues
        .iter()
        .any(|i| i.severity == EnvIssueSeverity::Warn));
}

// ----- T127: install guide contract --------------------------------------

#[test]
fn install_guide_macos_has_brew_step_and_release_notes_url() {
    let guide = install_guide("macos");
    assert_eq!(guide.platform, "macos");
    assert!(guide
        .steps
        .iter()
        .any(|s| s.command.as_deref().unwrap_or("").contains("brew")));
    assert!(guide.release_notes_url.starts_with("https://"));
}

#[test]
fn install_guide_windows_uses_winget() {
    let guide = install_guide("windows");
    assert!(guide
        .steps
        .iter()
        .any(|s| s.command.as_deref().unwrap_or("").contains("winget")));
}

#[test]
fn install_guide_linux_default_covers_supported_distros() {
    let guide = install_guide("linux");
    assert!(!guide.steps.is_empty());
    // Description should mention at least one supported distribution.
    let joined = guide
        .steps
        .iter()
        .map(|s| s.description.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("Ubuntu") || joined.contains("Debian") || joined.contains("Fedora"),
        "linux guide should reference a supported distro: {joined}"
    );
}

// ----- Parser sanity check — catches regressions in SemVer extraction.
#[test]
fn parse_semver_handles_common_cli_shapes() {
    assert_eq!(parse_semver("spec-kit 0.6.1"), Some("0.6.1".into()));
    assert_eq!(
        parse_semver("spec-kit version v1.2.3"),
        Some("1.2.3".into())
    );
    assert_eq!(parse_semver("garbage output"), None);
}
