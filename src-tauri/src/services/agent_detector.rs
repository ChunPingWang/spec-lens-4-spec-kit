//! AI Agent detector — implements FR-061's three-step priority chain.
//!
//! Resolution order:
//!
//! 1. **Project config** (highest priority): read the `ai` field from
//!    `.specify/init-options.json`. Returns `AgentDetectionSource::Project`.
//! 2. **Filesystem fingerprint**: probe the project root for known
//!    per-agent marker files/directories (`.claude/`, `.github/copilot/`,
//!    `.cursor/`, etc.). Returns `AgentDetectionSource::Path`.
//! 3. **Generic fallback**: when neither of the above match, return
//!    [`AgentProfile::generic`] with `AgentDetectionSource::None`. The
//!    UI shows an "unrecognized agent" badge but the workspace stays
//!    fully functional (FR-060).
//!
//! The detector is pure (no async, no global state) so it can be unit
//! tested against tempfile fixtures and called from `project_open`
//! synchronously.

use std::path::Path;

use serde::Deserialize;

use crate::error::SpecLensResult;
use crate::models::{AgentDetectionSource, AgentId, AgentProfile};

/// Shape of `.specify/init-options.json` — only the `ai` slug matters
/// for agent detection. Extra fields are ignored.
#[derive(Debug, Default, Deserialize)]
struct InitOptions {
    #[serde(default)]
    ai: Option<String>,
}

/// Map a Spec-Kit `--ai` slug onto an [`AgentId`]. Returns `None` for
/// values we don't recognize so the caller can fall through to the
/// fingerprint pass.
pub fn ai_slug_to_id(slug: &str) -> Option<AgentId> {
    let s = slug.trim().to_ascii_lowercase();
    Some(match s.as_str() {
        "claude" | "claude-code" | "claudecode" => AgentId::ClaudeCode,
        "copilot" | "github-copilot" | "githubcopilot" => AgentId::Copilot,
        "gemini" | "gemini-cli" | "geminicli" => AgentId::GeminiCli,
        "cursor" => AgentId::Cursor,
        "windsurf" => AgentId::Windsurf,
        "q" | "amazon-q" | "amazonq" => AgentId::AmazonQ,
        "codex" | "codex-cli" | "codexcli" => AgentId::CodexCli,
        "qwen" | "qwen-code" | "qwencode" => AgentId::QwenCode,
        "opencode" => AgentId::Opencode,
        "kilo" | "kilo-code" | "kilocode" => AgentId::KiloCode,
        "auggie" | "auggie-cli" | "auggiecli" => AgentId::AuggieCli,
        "roo" | "roo-code" | "roocode" => AgentId::RooCode,
        "generic" => AgentId::Generic,
        _ => return None,
    })
}

/// Returns the localized i18n key + display label for an agent. The
/// display name doubles as the badge text and is intentionally never
/// translated (FR-087 — brand names stay in their canonical form).
fn display_for(id: AgentId) -> &'static str {
    match id {
        AgentId::ClaudeCode => "Claude Code",
        AgentId::Copilot => "GitHub Copilot",
        AgentId::GeminiCli => "Gemini CLI",
        AgentId::Cursor => "Cursor",
        AgentId::Windsurf => "Windsurf",
        AgentId::AmazonQ => "Amazon Q Developer",
        AgentId::CodexCli => "Codex CLI",
        AgentId::QwenCode => "Qwen Code",
        AgentId::Opencode => "opencode",
        AgentId::KiloCode => "Kilo Code",
        AgentId::AuggieCli => "Auggie CLI",
        AgentId::RooCode => "Roo Code",
        AgentId::Generic => "Generic",
    }
}

fn icon_key_for(id: AgentId) -> &'static str {
    match id {
        AgentId::ClaudeCode => "agent-claude-code",
        AgentId::Copilot => "agent-copilot",
        AgentId::GeminiCli => "agent-gemini-cli",
        AgentId::Cursor => "agent-cursor",
        AgentId::Windsurf => "agent-windsurf",
        AgentId::AmazonQ => "agent-amazon-q",
        AgentId::CodexCli => "agent-codex-cli",
        AgentId::QwenCode => "agent-qwen-code",
        AgentId::Opencode => "agent-opencode",
        AgentId::KiloCode => "agent-kilo-code",
        AgentId::AuggieCli => "agent-auggie-cli",
        AgentId::RooCode => "agent-roo-code",
        AgentId::Generic => "agent-generic",
    }
}

/// Build a fully-populated [`AgentProfile`] for the given id + source.
pub fn build_profile(id: AgentId, detected_from: AgentDetectionSource) -> AgentProfile {
    AgentProfile {
        id,
        display_name: display_for(id).to_string(),
        icon_key: icon_key_for(id).to_string(),
        detected_from,
        version: None,
        // v1 ships a single rule set across every agent (FR-061 note +
        // research D27). Per-agent overrides land post-v1.
        highlight_rule_set: "universal".to_string(),
    }
}

/// Per-agent filesystem fingerprint. The first entry whose path exists
/// (relative to the project root) wins; the fingerprints are ordered
/// from most-specific to least-specific.
struct Fingerprint {
    id: AgentId,
    paths: &'static [&'static str],
}

const FINGERPRINTS: &[Fingerprint] = &[
    Fingerprint {
        id: AgentId::ClaudeCode,
        paths: &[".claude/settings.json", ".claude/agents", ".claude"],
    },
    Fingerprint {
        id: AgentId::Copilot,
        paths: &[
            ".github/copilot-instructions.md",
            ".github/copilot",
            ".vscode/copilot",
        ],
    },
    Fingerprint {
        id: AgentId::GeminiCli,
        paths: &[".gemini/config.json", ".gemini", "GEMINI.md"],
    },
    Fingerprint {
        id: AgentId::Cursor,
        paths: &[".cursorrules", ".cursor/rules", ".cursor"],
    },
    Fingerprint {
        id: AgentId::Windsurf,
        paths: &[".windsurfrules", ".windsurf"],
    },
    Fingerprint {
        id: AgentId::AmazonQ,
        paths: &[".amazonq/config.json", ".amazonq", ".aws/amazonq"],
    },
    Fingerprint {
        id: AgentId::CodexCli,
        paths: &[".codex/config.json", ".codex"],
    },
    Fingerprint {
        id: AgentId::QwenCode,
        paths: &[".qwen/config.json", ".qwen"],
    },
    Fingerprint {
        id: AgentId::Opencode,
        paths: &[".opencode/config.json", ".opencode"],
    },
    Fingerprint {
        id: AgentId::KiloCode,
        paths: &[".kilocode/config.json", ".kilocode"],
    },
    Fingerprint {
        id: AgentId::AuggieCli,
        paths: &[".auggie/config.json", ".auggie"],
    },
    Fingerprint {
        id: AgentId::RooCode,
        paths: &[".roo/config.json", ".roo"],
    },
];

/// Read `.specify/init-options.json` and return the resolved
/// [`AgentId`] when the `ai` slug is recognized.
pub fn detect_from_config(root: &Path) -> Option<AgentId> {
    let path = root.join(".specify").join("init-options.json");
    let bytes = std::fs::read(&path).ok()?;
    let opts: InitOptions = serde_json::from_slice(&bytes).ok()?;
    let slug = opts.ai?;
    ai_slug_to_id(&slug)
}

/// Walk the per-agent fingerprints in declaration order and return the
/// first one whose marker exists under `root`.
pub fn detect_from_fingerprint(root: &Path) -> Option<AgentId> {
    for fp in FINGERPRINTS {
        for rel in fp.paths {
            if root.join(rel).exists() {
                return Some(fp.id);
            }
        }
    }
    None
}

/// Run the FR-061 three-step chain and return a populated profile.
/// Never errors — a missing or unreadable config simply falls through
/// to the next stage so the user can always open their project.
pub fn detect(root: &Path) -> SpecLensResult<AgentProfile> {
    if let Some(id) = detect_from_config(root) {
        return Ok(build_profile(id, AgentDetectionSource::Project));
    }
    if let Some(id) = detect_from_fingerprint(root) {
        return Ok(build_profile(id, AgentDetectionSource::Path));
    }
    Ok(build_profile(AgentId::Generic, AgentDetectionSource::None))
}

// ---------------------------------------------------------------------
// Tests — T135
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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

    #[test]
    fn config_takes_priority_over_fingerprint() {
        let tmp = TempDir::new().unwrap();
        // Filesystem looks like Cursor, but config says Claude → config wins.
        fs::create_dir_all(tmp.path().join(".cursor")).unwrap();
        write_init(tmp.path(), "claude");
        let profile = detect(tmp.path()).expect("detect");
        assert_eq!(profile.id, AgentId::ClaudeCode);
        assert_eq!(profile.detected_from, AgentDetectionSource::Project);
    }

    #[test]
    fn fingerprint_used_when_config_missing() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".cursor")).unwrap();
        let profile = detect(tmp.path()).expect("detect");
        assert_eq!(profile.id, AgentId::Cursor);
        assert_eq!(profile.detected_from, AgentDetectionSource::Path);
    }

    #[test]
    fn falls_back_to_generic_when_nothing_matches() {
        let tmp = TempDir::new().unwrap();
        let profile = detect(tmp.path()).expect("detect");
        assert_eq!(profile.id, AgentId::Generic);
        assert_eq!(profile.detected_from, AgentDetectionSource::None);
        assert_eq!(profile.highlight_rule_set, "universal");
    }

    #[test]
    fn unknown_ai_slug_falls_through_to_fingerprint() {
        let tmp = TempDir::new().unwrap();
        write_init(tmp.path(), "made-up-agent");
        fs::create_dir_all(tmp.path().join(".windsurf")).unwrap();
        let profile = detect(tmp.path()).expect("detect");
        assert_eq!(profile.id, AgentId::Windsurf);
        assert_eq!(profile.detected_from, AgentDetectionSource::Path);
    }

    #[test]
    fn malformed_init_options_falls_through_safely() {
        let tmp = TempDir::new().unwrap();
        write_file(tmp.path(), ".specify/init-options.json", "{ not json");
        let profile = detect(tmp.path()).expect("detect");
        assert_eq!(profile.id, AgentId::Generic);
    }

    #[test]
    fn ai_slug_recognises_all_known_agents() {
        let cases = [
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
            ("generic", AgentId::Generic),
        ];
        for (slug, expected) in cases {
            assert_eq!(ai_slug_to_id(slug), Some(expected), "slug={slug}");
        }
    }

    #[test]
    fn fingerprint_chain_covers_each_known_agent() {
        // Build one tempdir per agent and assert the fingerprint resolves.
        let cases: &[(AgentId, &str)] = &[
            (AgentId::ClaudeCode, ".claude/settings.json"),
            (AgentId::Copilot, ".github/copilot-instructions.md"),
            (AgentId::GeminiCli, ".gemini/config.json"),
            (AgentId::Cursor, ".cursorrules"),
            (AgentId::Windsurf, ".windsurfrules"),
            (AgentId::AmazonQ, ".amazonq/config.json"),
            (AgentId::CodexCli, ".codex/config.json"),
            (AgentId::QwenCode, ".qwen/config.json"),
            (AgentId::Opencode, ".opencode/config.json"),
            (AgentId::KiloCode, ".kilocode/config.json"),
            (AgentId::AuggieCli, ".auggie/config.json"),
            (AgentId::RooCode, ".roo/config.json"),
        ];
        for (id, marker) in cases {
            let tmp = TempDir::new().unwrap();
            write_file(tmp.path(), marker, "{}");
            let profile = detect(tmp.path()).expect("detect");
            assert_eq!(profile.id, *id, "marker={marker}");
            assert_eq!(profile.detected_from, AgentDetectionSource::Path);
        }
    }

    #[test]
    fn build_profile_uses_canonical_brand_names() {
        let p = build_profile(AgentId::ClaudeCode, AgentDetectionSource::Project);
        assert_eq!(p.display_name, "Claude Code");
        assert_eq!(p.icon_key, "agent-claude-code");
        assert_eq!(p.highlight_rule_set, "universal");
    }
}
