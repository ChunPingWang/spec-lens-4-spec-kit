//! `.gitignore` hint helper for FR-091.
//!
//! SpecLens persists per-project state under `.speclens/` at the project
//! root. On the first write we surface a one-time recommendation that the
//! user add `.speclens/` to the project `.gitignore`. The decision is pure
//! string parsing over the repository `.gitignore` — no git invocation and
//! no filesystem writes — so it is trivially unit-tested and cheap to call
//! from the command layer.
//!
//! Call sites (state_store, doc_hash_store) pass the project root; this
//! module returns a [`GitignoreHint`] describing whether a recommendation
//! should be shown and, when requested, the exact line that would be
//! appended. The command layer owns display state — this helper is the
//! single source of truth for *detection*.
//!
//! The matcher is intentionally lenient: any of `.speclens`, `.speclens/`,
//! `/.speclens`, `/.speclens/`, `**/.speclens/`, or any of the above with a
//! trailing `*` counts as "already ignored". Negated entries (`!.speclens`)
//! are treated as *not* ignoring so we still surface the hint.

use std::path::{Path, PathBuf};

/// Outcome of inspecting a project's `.gitignore` for `.speclens/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitignoreHint {
    /// `true` when the hint should be displayed (the project is a git
    /// checkout, `.speclens/` is not yet ignored, and we have not already
    /// recorded the acknowledgement).
    pub should_prompt: bool,
    /// The line SpecLens would append to `.gitignore` if the user accepts.
    pub recommended_line: &'static str,
    /// Absolute path of the `.gitignore` we inspected (existing or target).
    pub gitignore_path: PathBuf,
    /// `true` when the project root is inside a git checkout (`.git` dir
    /// at the root or an ancestor). We only prompt inside git checkouts —
    /// a bare directory does not need `.gitignore` hygiene.
    pub in_git_checkout: bool,
}

/// Canonical line SpecLens recommends appending to `.gitignore`.
pub const RECOMMENDED_LINE: &str = ".speclens/";

/// Inspect `root/.gitignore` and return a [`GitignoreHint`]. The helper
/// never writes anything — the caller decides whether to show UI and the
/// user is the only party that actually edits `.gitignore`.
pub fn inspect(root: &Path) -> GitignoreHint {
    let gitignore_path = root.join(".gitignore");
    let in_git_checkout = has_git_checkout(root);

    // If we are not inside a git checkout, the hint never fires. We still
    // return the recommended path so callers can surface documentation.
    if !in_git_checkout {
        return GitignoreHint {
            should_prompt: false,
            recommended_line: RECOMMENDED_LINE,
            gitignore_path,
            in_git_checkout: false,
        };
    }

    let contents = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
    let already_ignored = is_speclens_ignored(&contents);

    GitignoreHint {
        should_prompt: !already_ignored,
        recommended_line: RECOMMENDED_LINE,
        gitignore_path,
        in_git_checkout: true,
    }
}

/// Walk up from `root` looking for a `.git` directory (or file, for
/// worktrees/submodules). Returns `true` on first hit, `false` if the walk
/// reaches the filesystem root without finding one.
fn has_git_checkout(root: &Path) -> bool {
    let mut cursor: &Path = root;
    loop {
        if cursor.join(".git").exists() {
            return true;
        }
        match cursor.parent() {
            Some(parent) => cursor = parent,
            None => return false,
        }
    }
}

/// Return `true` when `contents` already contains a line that would cause
/// `.speclens/` to be ignored. The check is lenient enough to accept the
/// common variants but refuses to accept negations (`!.speclens`).
pub fn is_speclens_ignored(contents: &str) -> bool {
    for raw in contents.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('!') {
            // Negation — explicitly *not* ignored.
            continue;
        }
        // Strip optional leading `/` or `**/`, trailing `/` or `/*`.
        let mut candidate = line;
        if let Some(stripped) = candidate.strip_prefix("**/") {
            candidate = stripped;
        }
        if let Some(stripped) = candidate.strip_prefix('/') {
            candidate = stripped;
        }
        if let Some(stripped) = candidate.strip_suffix("/*") {
            candidate = stripped;
        }
        if let Some(stripped) = candidate.strip_suffix('/') {
            candidate = stripped;
        }
        if candidate == ".speclens" {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    fn init_git(root: &Path) {
        std::fs::create_dir(root.join(".git")).expect("create .git");
    }

    #[test]
    fn recognises_canonical_entry() {
        assert!(is_speclens_ignored(".speclens/\n"));
        assert!(is_speclens_ignored(".speclens\n"));
        assert!(is_speclens_ignored("/.speclens/\n"));
        assert!(is_speclens_ignored("**/.speclens/\n"));
        assert!(is_speclens_ignored(".speclens/*\n"));
    }

    #[test]
    fn ignores_comments_and_blank_lines() {
        let contents = "# SpecLens state\n\n  \n.speclens/\n";
        assert!(is_speclens_ignored(contents));
    }

    #[test]
    fn negations_do_not_count_as_ignored() {
        assert!(!is_speclens_ignored("!.speclens/\n"));
        assert!(!is_speclens_ignored("target/\nnode_modules/\n"));
    }

    #[test]
    fn does_not_match_similar_names() {
        assert!(!is_speclens_ignored("speclens/\n"));
        assert!(!is_speclens_ignored(".speclens-cache/\n"));
    }

    #[test]
    fn inspect_outside_git_never_prompts() {
        let dir = tempdir().unwrap();
        let hint = inspect(dir.path());
        assert!(!hint.in_git_checkout);
        assert!(!hint.should_prompt);
        assert_eq!(hint.recommended_line, RECOMMENDED_LINE);
    }

    #[test]
    fn inspect_git_repo_without_entry_prompts() {
        let dir = tempdir().unwrap();
        init_git(dir.path());
        let hint = inspect(dir.path());
        assert!(hint.in_git_checkout);
        assert!(hint.should_prompt);
    }

    #[test]
    fn inspect_git_repo_with_missing_file_prompts() {
        let dir = tempdir().unwrap();
        init_git(dir.path());
        let hint = inspect(dir.path());
        assert!(hint.should_prompt);
        assert_eq!(hint.gitignore_path, dir.path().join(".gitignore"));
    }

    #[test]
    fn inspect_git_repo_with_entry_does_not_prompt() {
        let dir = tempdir().unwrap();
        init_git(dir.path());
        std::fs::write(
            dir.path().join(".gitignore"),
            "# SpecLens\n.speclens/\nnode_modules/\n",
        )
        .unwrap();
        let hint = inspect(dir.path());
        assert!(hint.in_git_checkout);
        assert!(!hint.should_prompt);
    }

    #[test]
    fn inspect_walks_up_to_find_git_dir() {
        let dir = tempdir().unwrap();
        init_git(dir.path());
        let sub = dir.path().join("packages").join("frontend");
        std::fs::create_dir_all(&sub).unwrap();
        let hint = inspect(&sub);
        assert!(hint.in_git_checkout);
        // No .gitignore at the sub directory → should prompt.
        assert!(hint.should_prompt);
    }
}
