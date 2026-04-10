//! AgentProfile entity. See `data-model.md §6`.

use serde::{Deserialize, Serialize};

/// The 12 officially supported Spec-Kit AI Agents plus `Generic` fallback.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum AgentId {
    ClaudeCode,
    Copilot,
    GeminiCli,
    Cursor,
    Windsurf,
    AmazonQ,
    CodexCli,
    QwenCode,
    Opencode,
    KiloCode,
    AuggieCli,
    RooCode,
    Generic,
}

impl AgentId {
    pub fn display_key(self) -> &'static str {
        match self {
            Self::ClaudeCode => "agent.claudeCode",
            Self::Copilot => "agent.copilot",
            Self::GeminiCli => "agent.geminiCli",
            Self::Cursor => "agent.cursor",
            Self::Windsurf => "agent.windsurf",
            Self::AmazonQ => "agent.amazonQ",
            Self::CodexCli => "agent.codexCli",
            Self::QwenCode => "agent.qwenCode",
            Self::Opencode => "agent.opencode",
            Self::KiloCode => "agent.kiloCode",
            Self::AuggieCli => "agent.auggieCli",
            Self::RooCode => "agent.rooCode",
            Self::Generic => "agent.generic",
        }
    }

    pub fn all_known() -> &'static [AgentId] {
        &[
            Self::ClaudeCode,
            Self::Copilot,
            Self::GeminiCli,
            Self::Cursor,
            Self::Windsurf,
            Self::AmazonQ,
            Self::CodexCli,
            Self::QwenCode,
            Self::Opencode,
            Self::KiloCode,
            Self::AuggieCli,
            Self::RooCode,
            Self::Generic,
        ]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AgentDetectionSource {
    /// Read from Spec-Kit config `--ai` field.
    Project,
    /// Matched a filesystem or PATH fingerprint.
    Path,
    /// No match — using the generic fallback profile.
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentProfile {
    pub id: AgentId,
    pub display_name: String,
    pub icon_key: String,
    pub detected_from: AgentDetectionSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// v1 uses a single universal highlight rule set regardless of agent.
    pub highlight_rule_set: String,
}

impl AgentProfile {
    pub fn generic() -> Self {
        Self {
            id: AgentId::Generic,
            display_name: "Generic".to_string(),
            icon_key: "agent-generic".to_string(),
            detected_from: AgentDetectionSource::None,
            version: None,
            highlight_rule_set: "universal".to_string(),
        }
    }
}
