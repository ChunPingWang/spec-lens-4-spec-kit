//! Phase 8 US6 integration tests — exercise the `agent_detector` service
//! the same way `env_check_flow.rs` exercises `version_checker`. Covers
//! T136 (config + fingerprint paths for the 12 known agents) and T137
//! (`agent_list_known` returns 12 + 1 profiles).
//!
//! The Tauri command wrapper is a thin shim over the pure detector, so
//! testing the service functions provides equivalent coverage without
//! standing up a Tauri runtime.

use std::fs;
use std::path::Path;

use speclens_lib::models::{AgentDetectionSource, AgentId};
use speclens_lib::services::agent_detector::{
    ai_slug_to_id, build_profile, detect, detect_from_config, detect_from_fingerprint,
};
use tempfile::TempDir;

fn write_file(root: &Path, rel: &str, contents: &str) {
    let full = root.join(rel);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).expect("mkdir parent");
    }
    fs::write(&full, contents).expect("write file");
}

fn write_init(root: &Path, ai: &str) {
    write_file(
        root,
        ".specify/init-options.json",
        &format!("{{\"ai\":\"{ai}\"}}"),
    );
}

// ----- T136: 13 fixture projects (12 agents + generic) -------------------

#[test]
fn agent_detect_resolves_each_known_agent_via_config() {
    let cases: &[(&str, AgentId)] = &[
        ("claude", AgentId::ClaudeCode),
        ("copilot", AgentId::Copilot),
        ("gemini", AgentId::GeminiCli),
        ("cursor", AgentId::Cursor),
        ("windsurf", AgentId::Windsurf),
        ("q", AgentId::AmazonQ),
        ("codex", AgentId::CodexCli),
        ("qwen", AgentId::QwenCode),
        ("opencode", AgentId::Opencode),
        ("kilocode", AgentId::KiloCode),
        ("auggie", AgentId::AuggieCli),
        ("roo", AgentId::RooCode),
    ];

    for (slug, expected) in cases {
        let tmp = TempDir::new().unwrap();
        write_init(tmp.path(), slug);
        let profile = detect(tmp.path()).expect("detect");
        assert_eq!(profile.id, *expected, "config slug={slug}");
        assert_eq!(profile.detected_from, AgentDetectionSource::Project);
        // Brand names stay canonical regardless of locale (FR-087).
        assert!(
            !profile.display_name.is_empty(),
            "display name must be set for {expected:?}"
        );
        assert_eq!(profile.highlight_rule_set, "universal");
    }
}

#[test]
fn agent_detect_resolves_each_known_agent_via_fingerprint() {
    let cases: &[(&str, AgentId)] = &[
        (".claude/settings.json", AgentId::ClaudeCode),
        (".github/copilot-instructions.md", AgentId::Copilot),
        (".gemini/config.json", AgentId::GeminiCli),
        (".cursorrules", AgentId::Cursor),
        (".windsurfrules", AgentId::Windsurf),
        (".amazonq/config.json", AgentId::AmazonQ),
        (".codex/config.json", AgentId::CodexCli),
        (".qwen/config.json", AgentId::QwenCode),
        (".opencode/config.json", AgentId::Opencode),
        (".kilocode/config.json", AgentId::KiloCode),
        (".auggie/config.json", AgentId::AuggieCli),
        (".roo/config.json", AgentId::RooCode),
    ];

    for (marker, expected) in cases {
        let tmp = TempDir::new().unwrap();
        write_file(tmp.path(), marker, "{}");
        let profile = detect(tmp.path()).expect("detect");
        assert_eq!(profile.id, *expected, "marker={marker}");
        assert_eq!(profile.detected_from, AgentDetectionSource::Path);
    }
}

#[test]
fn agent_detect_thirteenth_fixture_falls_back_to_generic() {
    // The thirteenth project has no config and no fingerprint — must
    // resolve to the generic profile (FR-060: every feature available).
    let tmp = TempDir::new().unwrap();
    let profile = detect(tmp.path()).expect("detect");
    assert_eq!(profile.id, AgentId::Generic);
    assert_eq!(profile.detected_from, AgentDetectionSource::None);
    assert_eq!(profile.highlight_rule_set, "universal");
}

#[test]
fn agent_detect_config_takes_priority_over_fingerprint() {
    let tmp = TempDir::new().unwrap();
    // Filesystem says Cursor, config says Roo → config wins per FR-061.
    write_file(tmp.path(), ".cursorrules", "");
    write_init(tmp.path(), "roo");
    let profile = detect(tmp.path()).expect("detect");
    assert_eq!(profile.id, AgentId::RooCode);
    assert_eq!(profile.detected_from, AgentDetectionSource::Project);
}

// ----- T137: agent_list_known contract ----------------------------------

#[test]
fn agent_list_known_returns_twelve_plus_generic_profiles() {
    let profiles: Vec<_> = AgentId::all_known()
        .iter()
        .map(|id| build_profile(*id, AgentDetectionSource::None))
        .collect();
    assert_eq!(profiles.len(), 13);
    let generic_count = profiles.iter().filter(|p| p.id == AgentId::Generic).count();
    assert_eq!(generic_count, 1, "exactly one generic profile expected");
    // Every profile has a non-empty display name and the universal rule
    // set v1 ships.
    for p in &profiles {
        assert!(
            !p.display_name.is_empty(),
            "{:?} missing display name",
            p.id
        );
        assert!(!p.icon_key.is_empty(), "{:?} missing icon key", p.id);
        assert_eq!(p.highlight_rule_set, "universal");
    }
}

// ----- Detector helpers -------------------------------------------------

#[test]
fn detect_from_config_handles_unknown_slug_safely() {
    let tmp = TempDir::new().unwrap();
    write_init(tmp.path(), "totally-made-up");
    assert_eq!(detect_from_config(tmp.path()), None);
}

#[test]
fn detect_from_fingerprint_returns_none_for_empty_dir() {
    let tmp = TempDir::new().unwrap();
    assert_eq!(detect_from_fingerprint(tmp.path()), None);
}

#[test]
fn ai_slug_to_id_is_case_insensitive() {
    assert_eq!(ai_slug_to_id("Claude"), Some(AgentId::ClaudeCode));
    assert_eq!(ai_slug_to_id("COPILOT"), Some(AgentId::Copilot));
    assert_eq!(ai_slug_to_id("  Gemini-CLI  "), Some(AgentId::GeminiCli));
}
