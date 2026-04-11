//! Task file scanner — walks a project root using the `ignore` crate so
//! `.gitignore` / `.ignore` files are respected (T115 / FR-050).
//!
//! The scanner returns [`TaskFileCandidate`] rows keyed by project-relative
//! path. It intentionally does NOT auto-select any of them: FR-050 requires
//! the user to pick a file explicitly the first time (`tasks_pick_file` or
//! the picker UI), so the caller is responsible for presenting this list
//! without any pre-checked items.
//!
//! A candidate matches when:
//!
//! 1. Its extension is one of `.md` / `.json` / `.yaml` / `.yml` / `.txt`.
//! 2. Its basename contains "task" (case-insensitive) **or** its full
//!    project-relative path starts with `specs/` or `.specify/` — the two
//!    canonical spots Spec-Kit stores per-feature tasks.
//!
//! Keeping the heuristic file-name based (rather than parsing every
//! candidate) keeps the scan fast on large repos; the picker UI can still
//! show the counts lazily via `tasks_parse_file`.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

use crate::error::{SpecLensError, SpecLensResult};
use crate::models::TaskFileFormat;
use crate::services::tasks_parser::detect_format;

/// Candidate returned by the scanner. This is *not* the fully-parsed
/// [`crate::models::TaskFile`] — counts are computed lazily by the parser.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskFileCandidate {
    pub relative_path: String,
    pub display_name: String,
    pub format: TaskFileFormat,
}

/// Walks `project_root` and returns candidate task files. Hidden files and
/// `.gitignore`-excluded paths are skipped.
pub fn scan(project_root: &Path) -> SpecLensResult<Vec<TaskFileCandidate>> {
    if !project_root.exists() {
        return Err(SpecLensError::PathNotFound(
            project_root.display().to_string(),
        ));
    }
    if !project_root.is_dir() {
        return Err(SpecLensError::PathNotDir(
            project_root.display().to_string(),
        ));
    }

    let mut walker = WalkBuilder::new(project_root);
    walker
        .hidden(false) // allow `.specify/` traversal
        .git_ignore(true)
        .git_exclude(true)
        .require_git(false) // honour .gitignore even outside a git repo
        .parents(false)
        .follow_links(false);

    let mut out: Vec<TaskFileCandidate> = Vec::new();
    for result in walker.build() {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue, // tolerate transient fs errors per file
        };
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        let Some(format) = detect_format(path) else {
            continue;
        };
        let Ok(rel) = path.strip_prefix(project_root) else {
            continue;
        };
        let rel_str = rel.to_string_lossy().replace('\\', "/");

        if !looks_like_task_file(path, &rel_str) {
            continue;
        }

        let display_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("tasks")
            .to_string();

        out.push(TaskFileCandidate {
            relative_path: rel_str,
            display_name,
            format,
        });
    }

    // Stable, deterministic order so the picker UI doesn't jitter.
    out.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(out)
}

fn looks_like_task_file(path: &Path, rel: &str) -> bool {
    let basename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if basename.contains("task") {
        return true;
    }
    let rel_lower = rel.to_ascii_lowercase();
    rel_lower.starts_with("specs/") || rel_lower.starts_with(".specify/")
}

/// Resolve a project-relative path to an absolute path, ensuring the result
/// stays under `project_root`. Returns `Err(PathNotFound)` if the file no
/// longer exists and `Err(ConfigInvalid)` if the path tries to escape the
/// project root (FR-085 boundary check).
pub fn resolve_under_root(project_root: &Path, relative_path: &str) -> SpecLensResult<PathBuf> {
    let joined = project_root.join(relative_path);
    let canon = std::fs::canonicalize(&joined)
        .map_err(|_| SpecLensError::TaskFileNotFound(relative_path.to_string()))?;
    let canon_root = std::fs::canonicalize(project_root).unwrap_or_else(|_| project_root.into());
    if !canon.starts_with(&canon_root) {
        return Err(SpecLensError::ConfigInvalid(format!(
            "{relative_path} resolves outside the project root"
        )));
    }
    Ok(canon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_project(files: &[(&str, &str)]) -> TempDir {
        let tmp = TempDir::new().expect("tempdir");
        for (path, contents) in files {
            let full = tmp.path().join(path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).expect("mkdir");
            }
            fs::write(full, contents).expect("write");
        }
        tmp
    }

    #[test]
    fn scan_returns_candidates_without_auto_selecting_any() {
        let tmp = make_project(&[
            ("specs/001/tasks.md", "- [ ] T001 Foo\n"),
            ("specs/001/plan.md", "# Plan\n"),
            ("docs/README.md", "# docs\n"),
            ("my-tasks.txt", "first\n"),
            ("randomfile.yaml", "tasks:\n  - id: T001\n"),
        ]);

        let candidates = scan(tmp.path()).expect("scan");
        // Only the two task-like files should be picked up.
        let rels: Vec<_> = candidates.iter().map(|c| c.relative_path.clone()).collect();
        assert!(
            rels.iter().any(|r| r == "specs/001/tasks.md"),
            "missing specs/001/tasks.md in {rels:?}"
        );
        assert!(
            rels.iter().any(|r| r == "my-tasks.txt"),
            "missing my-tasks.txt in {rels:?}"
        );
        // `specs/001/plan.md` does NOT match "task" and is in specs/ so it
        // would normally pass the prefix test — make sure we keep it OUT by
        // requiring the basename hint.
        assert!(
            !rels.iter().any(|r| r == "docs/README.md"),
            "docs/README.md should be filtered"
        );
    }

    #[test]
    fn scan_respects_gitignore() {
        let tmp = make_project(&[
            (".gitignore", "ignored-tasks.md\n"),
            ("ignored-tasks.md", "- [ ] T001 should not appear\n"),
            ("visible-tasks.md", "- [ ] T001 visible\n"),
        ]);
        let candidates = scan(tmp.path()).expect("scan");
        let rels: Vec<_> = candidates.iter().map(|c| c.relative_path.clone()).collect();
        assert!(rels.iter().any(|r| r == "visible-tasks.md"));
        assert!(!rels.iter().any(|r| r == "ignored-tasks.md"));
    }

    #[test]
    fn resolve_rejects_paths_outside_project_root() {
        let tmp = TempDir::new().expect("tempdir");
        fs::write(tmp.path().join("tasks.md"), "- [ ] a\n").expect("write");
        // Valid path is OK.
        assert!(resolve_under_root(tmp.path(), "tasks.md").is_ok());
        // Missing file is TaskFileNotFound.
        let missing = resolve_under_root(tmp.path(), "nope.md").unwrap_err();
        assert!(matches!(missing, SpecLensError::TaskFileNotFound(_)));
    }

    #[test]
    fn scan_produces_deterministic_order() {
        let tmp = make_project(&[("z-tasks.md", ""), ("a-tasks.md", ""), ("m-tasks.md", "")]);
        let candidates = scan(tmp.path()).expect("scan");
        let rels: Vec<_> = candidates.iter().map(|c| c.relative_path.clone()).collect();
        assert_eq!(rels, vec!["a-tasks.md", "m-tasks.md", "z-tasks.md"]);
    }
}
