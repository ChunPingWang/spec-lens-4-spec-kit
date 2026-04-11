//! Task file parser (`data-model.md §11`, `tasks.md T114`).
//!
//! Accepts four input formats and produces a common `(TaskFile, Vec<TaskEntry>)`
//! tuple:
//!
//! * **Markdown** (`.md`) — GFM checkbox lines (`- [ ] T001 Title`), with
//!   heading breadcrumbs captured into `section_path`.
//! * **JSON** (`.json`) — `{ "tasks": [{ "id", "title", "status", ... }] }`.
//! * **YAML** (`.yaml` / `.yml`) — same shape as JSON.
//! * **Plain text** (`.txt`) — one task per non-empty line, each line becomes
//!   a `Todo` entry.
//!
//! Unknown status characters default to `Todo` per `data-model.md §11`; the
//! entire file fails with `SpecLensError::TaskParseFailed { .. }` only when
//! the input is structurally malformed (e.g. invalid JSON), never for
//! individual ambiguous rows.

use std::path::Path;

use chrono::Utc;
use regex::Regex;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

use crate::error::{SpecLensError, SpecLensResult};
use crate::models::{TaskEntry, TaskFile, TaskFileFormat, TaskStatus};

/// Cached regex for markdown checkbox lines. Captures:
/// 1. Indentation / list marker (unused, but bounds the anchor).
/// 2. Checkbox character ( , `x`, `X`, `~`, `-`, `!`).
/// 3. Title text.
fn checkbox_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^\s*[-*+]\s+\[(?P<mark>.)\]\s+(?P<title>.+?)\s*$")
            .expect("checkbox regex must compile")
    })
}

/// Detect the format from a relative path's extension. Returns `None` when
/// the extension is unknown (callers should treat that as "not a task file").
pub fn detect_format(path: &Path) -> Option<TaskFileFormat> {
    let ext = path.extension().and_then(|e| e.to_str())?;
    match ext.to_ascii_lowercase().as_str() {
        "md" | "markdown" => Some(TaskFileFormat::Md),
        "json" => Some(TaskFileFormat::Json),
        "yaml" | "yml" => Some(TaskFileFormat::Yaml),
        "txt" => Some(TaskFileFormat::Txt),
        _ => None,
    }
}

/// Parse `contents` as the given format, producing the descriptor + entries.
pub fn parse(
    relative_path: &str,
    format: TaskFileFormat,
    contents: &str,
) -> SpecLensResult<(TaskFile, Vec<TaskEntry>)> {
    let entries = match format {
        TaskFileFormat::Md => parse_markdown(relative_path, contents)?,
        TaskFileFormat::Json => parse_json(relative_path, contents)?,
        TaskFileFormat::Yaml => parse_yaml(relative_path, contents)?,
        TaskFileFormat::Txt => parse_txt(relative_path, contents),
    };

    let completed = entries
        .iter()
        .filter(|e| e.status == TaskStatus::Done)
        .count() as u32;
    let display_name = Path::new(relative_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("tasks")
        .to_string();

    let file = TaskFile {
        relative_path: relative_path.to_string(),
        format,
        display_name,
        task_count: entries.len() as u32,
        completed_count: completed,
        parsed_at: Utc::now(),
    };
    Ok((file, entries))
}

// -------- markdown --------------------------------------------------------

fn parse_markdown(relative_path: &str, contents: &str) -> SpecLensResult<Vec<TaskEntry>> {
    let re = checkbox_regex();
    let mut entries: Vec<TaskEntry> = Vec::new();
    let mut section: Vec<String> = Vec::new();
    let file_hash = short_hash(relative_path, contents);

    for (idx, raw_line) in contents.lines().enumerate() {
        let line_no = (idx + 1) as u32;
        let trimmed = raw_line.trim_start();
        if let Some(stripped) = trimmed.strip_prefix('#') {
            // Count the heading level, then push into section breadcrumb.
            let level = 1 + stripped.chars().take_while(|c| *c == '#').count();
            let title = stripped.trim_start_matches('#').trim().to_string();
            if !title.is_empty() {
                if section.len() >= level {
                    section.truncate(level - 1);
                }
                while section.len() < level - 1 {
                    section.push(String::new());
                }
                section.push(title);
            }
            continue;
        }

        let Some(caps) = re.captures(raw_line) else {
            continue;
        };
        let mark = caps.name("mark").map(|m| m.as_str()).unwrap_or(" ");
        let title = caps
            .name("title")
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let status = TaskStatus::normalize(mark);
        entries.push(TaskEntry {
            id: format!("{file_hash}:{line_no}"),
            title,
            status,
            section_path: section.clone(),
            file_path: relative_path.to_string(),
            line: line_no,
            raw: raw_line.to_string(),
        });
    }
    Ok(entries)
}

// -------- json / yaml shared shape ---------------------------------------

#[derive(Debug, Deserialize)]
struct TaskDoc {
    tasks: Vec<TaskRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskRow {
    id: Option<String>,
    title: Option<String>,
    status: Option<String>,
    #[serde(default)]
    line: Option<u32>,
    #[serde(default)]
    section_path: Vec<String>,
}

fn parse_json(relative_path: &str, contents: &str) -> SpecLensResult<Vec<TaskEntry>> {
    let doc: TaskDoc = serde_json::from_str(contents).map_err(|e| {
        SpecLensError::TaskParseFailed(format!("{relative_path}: line {}: {e}", e.line().max(1)))
    })?;
    Ok(rows_to_entries(relative_path, contents, doc.tasks))
}

fn parse_yaml(relative_path: &str, contents: &str) -> SpecLensResult<Vec<TaskEntry>> {
    let doc: TaskDoc = serde_yaml::from_str(contents)
        .map_err(|e| SpecLensError::TaskParseFailed(format!("{relative_path}: {e}")))?;
    Ok(rows_to_entries(relative_path, contents, doc.tasks))
}

fn rows_to_entries(relative_path: &str, contents: &str, rows: Vec<TaskRow>) -> Vec<TaskEntry> {
    let file_hash = short_hash(relative_path, contents);
    rows.into_iter()
        .enumerate()
        .map(|(idx, row)| {
            let line = row.line.unwrap_or((idx + 1) as u32);
            let title = row.title.unwrap_or_default();
            let status = row
                .status
                .as_deref()
                .map(TaskStatus::normalize)
                .unwrap_or(TaskStatus::Todo);
            let id = row
                .id
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("{file_hash}:{line}"));
            let raw = serde_json::to_string(&serde_json::json!({
                "id": id,
                "title": title,
                "status": status_to_str(status),
            }))
            .unwrap_or_default();
            TaskEntry {
                id,
                title,
                status,
                section_path: row.section_path,
                file_path: relative_path.to_string(),
                line,
                raw,
            }
        })
        .collect()
}

fn status_to_str(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "todo",
        TaskStatus::InProgress => "in_progress",
        TaskStatus::Done => "done",
        TaskStatus::Skipped => "skipped",
    }
}

// -------- plain text ------------------------------------------------------

fn parse_txt(relative_path: &str, contents: &str) -> Vec<TaskEntry> {
    let file_hash = short_hash(relative_path, contents);
    contents
        .lines()
        .enumerate()
        .filter_map(|(idx, raw)| {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return None;
            }
            let line_no = (idx + 1) as u32;
            Some(TaskEntry {
                id: format!("{file_hash}:{line_no}"),
                title: trimmed.to_string(),
                status: TaskStatus::Todo,
                section_path: Vec::new(),
                file_path: relative_path.to_string(),
                line: line_no,
                raw: raw.to_string(),
            })
        })
        .collect()
}

fn short_hash(relative_path: &str, contents: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(relative_path.as_bytes());
    hasher.update(b"\n");
    hasher.update(contents.as_bytes());
    let digest = hasher.finalize();
    hex::encode(digest)[..12].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MD: &str = "# Sample Task File\n\n## Section A\n\n- [ ] T001 First todo item\n- [x] T002 Second done item\n- [ ] T003 Third todo item\n\n## Section B\n\n- [ ] T004 Fourth todo item\n- [x] T005 Fifth done item\n";

    const JSON: &str = "{\n  \"tasks\": [\n    { \"id\": \"T001\", \"title\": \"First item\", \"status\": \"todo\" },\n    { \"id\": \"T002\", \"title\": \"Second item\", \"status\": \"done\" },\n    { \"id\": \"T003\", \"title\": \"Third item\", \"status\": \"in_progress\" }\n  ]\n}\n";

    const YAML: &str = "tasks:\n  - id: T001\n    title: First item\n    status: todo\n  - id: T002\n    title: Second item\n    status: done\n  - id: T003\n    title: Third item\n    status: skipped\n";

    const TXT: &str = "First plain-text task\nSecond plain-text task\nThird plain-text task\n";

    #[test]
    fn detect_format_from_extension() {
        assert_eq!(
            detect_format(Path::new("tasks.md")),
            Some(TaskFileFormat::Md)
        );
        assert_eq!(
            detect_format(Path::new("a.JSON")),
            Some(TaskFileFormat::Json)
        );
        assert_eq!(
            detect_format(Path::new("a.yml")),
            Some(TaskFileFormat::Yaml)
        );
        assert_eq!(detect_format(Path::new("a.txt")), Some(TaskFileFormat::Txt));
        assert_eq!(detect_format(Path::new("README")), None);
    }

    #[test]
    fn markdown_parses_checkboxes_and_sections() {
        let (file, entries) = parse("tasks.md", TaskFileFormat::Md, MD).expect("parse");
        assert_eq!(file.format, TaskFileFormat::Md);
        assert_eq!(file.task_count, 5);
        assert_eq!(file.completed_count, 2);
        assert_eq!(entries.len(), 5);
        // First entry lives under Sample Task File > Section A.
        assert_eq!(entries[0].status, TaskStatus::Todo);
        assert_eq!(entries[0].title, "T001 First todo item");
        assert_eq!(
            entries[0].section_path,
            vec!["Sample Task File".to_string(), "Section A".to_string()]
        );
        assert_eq!(entries[1].status, TaskStatus::Done);
        // Section B tasks are isolated from Section A.
        assert_eq!(
            entries[3].section_path,
            vec!["Sample Task File".to_string(), "Section B".to_string()]
        );
    }

    #[test]
    fn markdown_ids_are_deterministic_for_same_contents() {
        let (_, a) = parse("tasks.md", TaskFileFormat::Md, MD).expect("a");
        let (_, b) = parse("tasks.md", TaskFileFormat::Md, MD).expect("b");
        let ids_a: Vec<_> = a.iter().map(|e| e.id.clone()).collect();
        let ids_b: Vec<_> = b.iter().map(|e| e.id.clone()).collect();
        assert_eq!(ids_a, ids_b);
        // All ids must be unique within a file.
        let mut sorted = ids_a.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), ids_a.len());
    }

    #[test]
    fn json_parses_shared_shape() {
        let (file, entries) = parse("tasks.json", TaskFileFormat::Json, JSON).expect("parse");
        assert_eq!(file.task_count, 3);
        assert_eq!(file.completed_count, 1);
        assert_eq!(entries[0].status, TaskStatus::Todo);
        assert_eq!(entries[1].status, TaskStatus::Done);
        assert_eq!(entries[2].status, TaskStatus::InProgress);
    }

    #[test]
    fn yaml_parses_shared_shape() {
        let (file, entries) = parse("tasks.yaml", TaskFileFormat::Yaml, YAML).expect("parse");
        assert_eq!(file.task_count, 3);
        assert_eq!(entries[2].status, TaskStatus::Skipped);
    }

    #[test]
    fn txt_parses_one_per_line_ignoring_blanks() {
        let (file, entries) = parse("tasks.txt", TaskFileFormat::Txt, TXT).expect("parse");
        assert_eq!(file.task_count, 3);
        assert_eq!(file.completed_count, 0);
        assert_eq!(entries[0].title, "First plain-text task");
        assert!(entries.iter().all(|e| e.status == TaskStatus::Todo));
    }

    #[test]
    fn json_malformed_returns_task_parse_failed_with_line_hint() {
        let err = parse("tasks.json", TaskFileFormat::Json, "{ not valid json").unwrap_err();
        match err {
            SpecLensError::TaskParseFailed(msg) => {
                assert!(msg.contains("tasks.json"), "msg must include path: {msg}");
            }
            other => panic!("expected TaskParseFailed, got {other:?}"),
        }
    }
}
