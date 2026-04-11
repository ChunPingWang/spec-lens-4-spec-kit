//! Environment / version checker for the Spec-Kit CLI.
//!
//! Phase 7 US5 backend — probes `spec-kit --version`, compares the result
//! against a bundled "latest known" manifest, and exposes the pure
//! functions the command layer and tests need. The real probe is wrapped
//! behind the [`VersionProbe`] trait so we can inject deterministic
//! fixtures into tests (no process spawning) and so a future background
//! task can poll a release API without touching the command layer.
//!
//! Offline fallback (FR-013): if the probe or the manifest lookup errors,
//! we still return an [`EnvironmentStatus`] containing the cached values
//! we do have and emit a warn-level issue describing the failure — never
//! a hard error.

use std::path::PathBuf;
use std::process::Command;

use chrono::Utc;

use crate::error::SpecLensResult;
use crate::models::{EnvIssue, EnvIssueKind, EnvIssueSeverity, EnvironmentStatus};

/// Bundled "latest known" Spec-Kit manifest. Kept deliberately small so
/// the v1 build has a deterministic offline baseline — a background task
/// can refresh this from a signed manifest post-v1 (see research.md
/// D-versioning).
pub const BUNDLED_LATEST: &str = "0.6.0";

/// Result of probing the Spec-Kit CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeOutcome {
    /// The version string parsed out of `spec-kit --version`, if any.
    pub version: Option<String>,
    /// Absolute path to the resolved binary (for display / diagnostics).
    pub path: Option<PathBuf>,
    /// Populated when the probe failed. Callers MUST still produce a
    /// non-fatal [`EnvironmentStatus`] (FR-013).
    pub error: Option<String>,
}

impl ProbeOutcome {
    pub fn not_found() -> Self {
        Self {
            version: None,
            path: None,
            error: Some("spec-kit binary not found on PATH".to_string()),
        }
    }

    pub fn installed(version: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self {
            version: Some(version.into()),
            path,
            error: None,
        }
    }
}

/// Injection point for probing the CLI. The default [`SystemProbe`]
/// shells out to `spec-kit --version`; tests use the in-memory
/// [`StaticProbe`] below.
pub trait VersionProbe {
    fn probe(&self) -> ProbeOutcome;
}

/// Production probe — runs `spec-kit --version` on the current PATH and
/// parses the first SemVer-shaped token out of stdout. Errors of any
/// kind (binary missing, non-zero exit, unparseable output) are folded
/// into [`ProbeOutcome::error`] so the caller can still ship an amber
/// status (FR-013).
pub struct SystemProbe;

impl VersionProbe for SystemProbe {
    fn probe(&self) -> ProbeOutcome {
        let Ok(output) = Command::new("spec-kit").arg("--version").output() else {
            return ProbeOutcome::not_found();
        };
        if !output.status.success() {
            return ProbeOutcome {
                version: None,
                path: None,
                error: Some(format!(
                    "spec-kit --version exited with status {:?}",
                    output.status.code()
                )),
            };
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let version = parse_semver(&stdout);
        let path = which_spec_kit();
        match version {
            Some(v) => ProbeOutcome::installed(v, path),
            None => ProbeOutcome {
                version: None,
                path,
                error: Some(format!("could not parse version from: {}", stdout.trim())),
            },
        }
    }
}

/// Deterministic probe for tests + integration harnesses.
#[derive(Debug, Clone)]
pub struct StaticProbe {
    pub outcome: ProbeOutcome,
}

impl StaticProbe {
    pub fn new(outcome: ProbeOutcome) -> Self {
        Self { outcome }
    }
}

impl VersionProbe for StaticProbe {
    fn probe(&self) -> ProbeOutcome {
        self.outcome.clone()
    }
}

/// Extract the first `X.Y.Z[-prerelease][+build]` token from a free-form
/// CLI version string (e.g. `"spec-kit 0.6.1 (build abc)"` → `"0.6.1"`).
pub fn parse_semver(text: &str) -> Option<String> {
    for token in text.split_whitespace() {
        // Strip common prefixes like `v` or `version=`.
        let cleaned = token.trim_start_matches(['v', 'V']).trim_matches(',');
        if looks_like_semver(cleaned) {
            return Some(cleaned.to_string());
        }
    }
    None
}

fn looks_like_semver(s: &str) -> bool {
    // Minimal validation: at least MAJOR.MINOR.PATCH with digits.
    let core = s.split(['-', '+']).next().unwrap_or(s);
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

/// Compare two SemVer strings. Returns an `Ordering` where `Greater`
/// means `a > b`. Falls back to string comparison if either side is not
/// well-formed so we never panic on user-provided data.
pub fn compare_semver(a: &str, b: &str) -> std::cmp::Ordering {
    fn to_tuple(v: &str) -> Option<(u32, u32, u32)> {
        let core = v.split(['-', '+']).next()?;
        let mut parts = core.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        Some((major, minor, patch))
    }
    match (to_tuple(a), to_tuple(b)) {
        (Some(x), Some(y)) => x.cmp(&y),
        _ => a.cmp(b),
    }
}

fn which_spec_kit() -> Option<PathBuf> {
    let path_env = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_env) {
        for candidate in ["spec-kit", "spec-kit.exe"] {
            let full = dir.join(candidate);
            if full.is_file() {
                return Some(full);
            }
        }
    }
    None
}

/// Build an [`EnvironmentStatus`] from a probe outcome and the bundled
/// "latest known" manifest. The optional `latest_override` lets callers
/// inject a newer value from a successful background fetch.
pub fn build_status(
    outcome: ProbeOutcome,
    latest_override: Option<&str>,
) -> SpecLensResult<EnvironmentStatus> {
    let latest_known = latest_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| BUNDLED_LATEST.to_string());

    let mut issues = Vec::new();

    let (installed, version, path) = match &outcome {
        ProbeOutcome {
            version: Some(v),
            path,
            error: None,
        } => (true, Some(v.clone()), path.clone()),
        ProbeOutcome {
            version: None,
            error: Some(err),
            ..
        } => {
            issues.push(EnvIssue {
                kind: EnvIssueKind::MissingBinary,
                severity: EnvIssueSeverity::Error,
                message: err.clone(),
                hint: Some(
                    "Install Spec-Kit CLI from the bundled guide and re-run the check.".to_string(),
                ),
            });
            (false, None, None)
        }
        ProbeOutcome {
            version: Some(v),
            path,
            error: Some(err),
        } => {
            issues.push(EnvIssue {
                kind: EnvIssueKind::Other,
                severity: EnvIssueSeverity::Warn,
                message: err.clone(),
                hint: None,
            });
            (true, Some(v.clone()), path.clone())
        }
        ProbeOutcome {
            version: None,
            path: _,
            error: None,
        } => {
            issues.push(EnvIssue {
                kind: EnvIssueKind::MissingBinary,
                severity: EnvIssueSeverity::Error,
                message: "spec-kit --version returned no output".to_string(),
                hint: None,
            });
            (false, None, None)
        }
    };

    let update_available = match &version {
        Some(v) => compare_semver(v, &latest_known) == std::cmp::Ordering::Less,
        None => false,
    };

    if update_available {
        issues.push(EnvIssue {
            kind: EnvIssueKind::VersionMismatch,
            severity: EnvIssueSeverity::Warn,
            message: format!(
                "Installed Spec-Kit {} is older than latest known {}",
                version.as_deref().unwrap_or("?"),
                latest_known
            ),
            hint: Some("Run the in-app updater from the environment banner.".to_string()),
        });
    }

    Ok(EnvironmentStatus {
        speckit_installed: installed,
        speckit_version: version,
        speckit_path: path.map(|p| p.display().to_string()),
        latest_known_version: Some(latest_known),
        update_available,
        checked_at: Utc::now(),
        issues,
    })
}

/// Bundled installation guide step.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstallGuideStep {
    pub title: String,
    pub description: String,
    pub command: Option<String>,
}

/// Full guide payload returned by `env_get_install_guide`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstallGuide {
    pub platform: String,
    pub steps: Vec<InstallGuideStep>,
    pub release_notes_url: String,
}

/// Return a platform-specific install guide. The content is intentionally
/// bundled in-code so the desktop app works fully offline (FR-086).
pub fn install_guide(platform: &str) -> InstallGuide {
    let release_notes_url = "https://github.com/github/spec-kit/releases".to_string();
    let steps = match platform {
        "macos" => vec![
            InstallGuideStep {
                title: "Install with Homebrew".into(),
                description: "Homebrew is the supported distribution on macOS 12+.".into(),
                command: Some("brew install github/tap/spec-kit".into()),
            },
            InstallGuideStep {
                title: "Verify the install".into(),
                description: "Confirm the binary is on PATH before re-running the check.".into(),
                command: Some("spec-kit --version".into()),
            },
        ],
        "windows" => vec![
            InstallGuideStep {
                title: "Install with Winget".into(),
                description: "Windows 10/11 x64 ships with Winget by default.".into(),
                command: Some("winget install GitHub.SpecKit".into()),
            },
            InstallGuideStep {
                title: "Verify the install".into(),
                description: "Open a new terminal so PATH refreshes, then run the check again."
                    .into(),
                command: Some("spec-kit --version".into()),
            },
        ],
        _ => vec![
            InstallGuideStep {
                title: "Install with the official script".into(),
                description: "Works on Ubuntu 22.04+, Debian 12+, and Fedora 38+.".into(),
                command: Some("curl -fsSL https://get.speckit.dev | sh".into()),
            },
            InstallGuideStep {
                title: "Verify the install".into(),
                description: "Confirm the binary resolves before re-running the check.".into(),
                command: Some("spec-kit --version".into()),
            },
        ],
    };
    InstallGuide {
        platform: platform.to_string(),
        steps,
        release_notes_url,
    }
}

// ---------------------------------------------------------------------
// Tests — T125
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_semver_extracts_version_from_free_form_output() {
        assert_eq!(parse_semver("spec-kit 0.6.1"), Some("0.6.1".into()));
        assert_eq!(
            parse_semver("spec-kit version v1.2.3"),
            Some("1.2.3".into())
        );
        assert_eq!(
            parse_semver("spec-kit 2.0.0-beta.1 (build abc)"),
            Some("2.0.0-beta.1".into())
        );
    }

    #[test]
    fn parse_semver_rejects_non_semver_output() {
        assert_eq!(parse_semver(""), None);
        assert_eq!(parse_semver("spec-kit build abc"), None);
        assert_eq!(parse_semver("spec-kit 1.2"), None);
    }

    #[test]
    fn compare_semver_orders_core_versions() {
        use std::cmp::Ordering::*;
        assert_eq!(compare_semver("1.0.0", "1.0.0"), Equal);
        assert_eq!(compare_semver("1.0.1", "1.0.0"), Greater);
        assert_eq!(compare_semver("0.6.0", "0.6.1"), Less);
        assert_eq!(compare_semver("2.0.0", "1.9.9"), Greater);
    }

    #[test]
    fn build_status_flags_update_when_installed_below_latest() {
        let outcome = ProbeOutcome::installed("0.5.0", None);
        let status = build_status(outcome, Some("0.6.0")).expect("status");
        assert!(status.speckit_installed);
        assert!(status.update_available);
        assert_eq!(status.speckit_version.as_deref(), Some("0.5.0"));
        assert_eq!(status.latest_known_version.as_deref(), Some("0.6.0"));
        assert!(status
            .issues
            .iter()
            .any(|i| i.kind == EnvIssueKind::VersionMismatch));
    }

    #[test]
    fn build_status_green_when_up_to_date() {
        let outcome = ProbeOutcome::installed("0.6.0", None);
        let status = build_status(outcome, Some("0.6.0")).expect("status");
        assert!(status.speckit_installed);
        assert!(!status.update_available);
        assert!(status.issues.is_empty());
        assert_eq!(status.badge_color(), crate::models::EnvBadgeColor::Green);
    }

    #[test]
    fn build_status_red_when_binary_missing() {
        let status = build_status(ProbeOutcome::not_found(), Some("0.6.0")).expect("status");
        assert!(!status.speckit_installed);
        assert!(!status.update_available);
        assert_eq!(status.issues.len(), 1);
        assert_eq!(status.issues[0].severity, EnvIssueSeverity::Error);
        assert_eq!(status.badge_color(), crate::models::EnvBadgeColor::Red);
    }

    #[test]
    fn build_status_offline_fallback_keeps_installed_version() {
        // Simulate an installed binary whose probe surfaced a secondary
        // diagnostic (e.g. stderr junk). FR-013 says the user MUST still
        // be allowed to proceed, so the status should be amber, not red.
        let outcome = ProbeOutcome {
            version: Some("0.6.0".to_string()),
            path: None,
            error: Some("could not reach update registry".to_string()),
        };
        let status = build_status(outcome, Some("0.6.0")).expect("status");
        assert!(status.speckit_installed);
        assert_eq!(status.badge_color(), crate::models::EnvBadgeColor::Amber);
    }

    #[test]
    fn static_probe_returns_injected_outcome() {
        let probe = StaticProbe::new(ProbeOutcome::installed("9.9.9", None));
        assert_eq!(probe.probe().version.as_deref(), Some("9.9.9"));
    }

    #[test]
    fn install_guide_covers_all_supported_platforms() {
        for platform in ["macos", "windows", "linux"] {
            let guide = install_guide(platform);
            assert!(
                !guide.steps.is_empty(),
                "{platform} guide must not be empty"
            );
            assert!(guide.release_notes_url.starts_with("https://"));
        }
    }
}
