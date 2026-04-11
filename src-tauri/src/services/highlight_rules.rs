//! Universal highlight rules for terminal output.
//!
//! Mirrors the frontend `src/lib/highlightRules.ts` so stream routing
//! decisions (FR-045 / US3) can be made inside the backend before emitting
//! `OutputLine` to the frontend. Rules are compiled once per process from a
//! JSON payload identical in shape to `src/lib/default-highlight-rules.json`.

use std::sync::OnceLock;

use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};

use crate::error::{SpecLensError, SpecLensResult};
use crate::models::{HighlightMatch, HighlightSeverity};

/// JSON-serialised form of a rule set. Matches the frontend shape exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleSetJson {
    pub version: u32,
    pub rule_set: String,
    pub rules: Vec<RuleJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleJson {
    pub id: String,
    pub severity: HighlightSeverity,
    pub pattern: String,
    #[serde(default)]
    pub flags: String,
}

/// Compiled rule ready for matching. Holds the severity + id + the compiled
/// regex. Built once via [`compile`] and cloned as needed.
#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub id: String,
    pub severity: HighlightSeverity,
    pub regex: Regex,
}

/// Compile a rule set, returning an ordered list of [`CompiledRule`]. Order
/// is preserved — the first matching rule wins (FR-045).
pub fn compile(set: &RuleSetJson) -> SpecLensResult<Vec<CompiledRule>> {
    let mut out = Vec::with_capacity(set.rules.len());
    for rule in &set.rules {
        let mut builder = RegexBuilder::new(&rule.pattern);
        if rule.flags.contains('i') {
            builder.case_insensitive(true);
        }
        if rule.flags.contains('m') {
            builder.multi_line(true);
        }
        if rule.flags.contains('s') {
            builder.dot_matches_new_line(true);
        }
        let regex = builder.build().map_err(|e| {
            SpecLensError::Internal(format!("invalid highlight rule '{}': {e}", rule.id))
        })?;
        out.push(CompiledRule {
            id: rule.id.clone(),
            severity: rule.severity,
            regex,
        });
    }
    Ok(out)
}

/// First rule that matches `line`, or `None`. See `highlightRules.ts`.
pub fn first_match(line: &str, rules: &[CompiledRule]) -> Option<HighlightMatch> {
    rules
        .iter()
        .find(|r| r.regex.is_match(line))
        .map(|r| HighlightMatch {
            rule_id: r.id.clone(),
            severity: r.severity,
        })
}

/// Bundled default rule set matching FR-045 categories: success, error,
/// running, warning, JSON, file-path. The frontend ships its own JSON file
/// at `src/lib/default-highlight-rules.json`; we keep a Rust copy so the
/// backend can annotate lines before emitting.
const DEFAULT_RULES_JSON: &str = r#"{
  "version": 1,
  "ruleSet": "universal",
  "rules": [
    { "id": "error-prefix",    "severity": "error",   "pattern": "^(?:ERROR|FATAL|Error):", "flags": "i" },
    { "id": "warn-prefix",     "severity": "warn",    "pattern": "^(?:WARN|WARNING):",       "flags": "i" },
    { "id": "running-marker",  "severity": "info",    "pattern": "^(?:RUN|RUNNING|>>>)",     "flags": "" },
    { "id": "success-marker",  "severity": "success", "pattern": "\\b(?:PASS|OK|SUCCESS|DONE|\u2714|\u2713)\\b", "flags": "" },
    { "id": "json-line",       "severity": "info",    "pattern": "^\\s*\\{.*\\}\\s*$",       "flags": "" },
    { "id": "file-path",       "severity": "info",    "pattern": "(?:[A-Za-z]:)?[\\w./\\-]+:\\d+(?::\\d+)?", "flags": "" }
  ]
}"#;

/// Load the compiled default rule set, cached behind a `OnceLock` so we pay
/// the compile cost once per process.
pub fn default_compiled() -> &'static [CompiledRule] {
    static CELL: OnceLock<Vec<CompiledRule>> = OnceLock::new();
    CELL.get_or_init(|| {
        let set: RuleSetJson = serde_json::from_str(DEFAULT_RULES_JSON)
            .expect("bundled default highlight rule set is valid json");
        compile(&set).expect("bundled default highlight rules compile")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rules_compile() {
        let rules = default_compiled();
        assert_eq!(rules.len(), 6);
    }

    #[test]
    fn error_prefix_matches_error_severity() {
        let hit = first_match("ERROR: boom", default_compiled()).expect("matched");
        assert_eq!(hit.severity, HighlightSeverity::Error);
        assert_eq!(hit.rule_id, "error-prefix");
    }

    #[test]
    fn warn_prefix_matches_warn_severity() {
        let hit = first_match("WARNING: low disk", default_compiled()).expect("matched");
        assert_eq!(hit.severity, HighlightSeverity::Warn);
    }

    #[test]
    fn pass_token_matches_success() {
        let hit = first_match("  PASS   src/a.test.ts", default_compiled()).expect("matched");
        assert_eq!(hit.severity, HighlightSeverity::Success);
    }

    #[test]
    fn running_marker_matches_info() {
        let hit = first_match(">>> cargo build", default_compiled()).expect("matched");
        assert_eq!(hit.severity, HighlightSeverity::Info);
        assert_eq!(hit.rule_id, "running-marker");
    }

    #[test]
    fn json_line_matches_info() {
        let hit = first_match(r#"  { "level": "info" }  "#, default_compiled()).expect("matched");
        assert_eq!(hit.rule_id, "json-line");
    }

    #[test]
    fn file_path_matches_info() {
        let hit = first_match("src/lib.rs:42:3 — todo", default_compiled()).expect("matched");
        assert_eq!(hit.rule_id, "file-path");
    }

    #[test]
    fn plain_line_returns_none() {
        assert!(first_match("nothing to see here", default_compiled()).is_none());
    }

    #[test]
    fn first_match_wins() {
        // An ERROR line also contains "error" and a file path, but the
        // error-prefix rule runs first so that severity wins.
        let hit = first_match("ERROR: src/lib.rs:1:1 oops", default_compiled()).unwrap();
        assert_eq!(hit.rule_id, "error-prefix");
    }
}
