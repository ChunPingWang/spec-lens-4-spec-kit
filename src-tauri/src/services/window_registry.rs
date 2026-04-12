//! Window registry (Phase 10 polish, T165 / FR-100, FR-101).
//!
//! SpecLens is multi-window: each top-level `tauri::Window` is bound to
//! at most one project and owns its own PTY session, terminal buffer, and
//! phase-tab state. The *registry* tracks which `(window_id, project_id)`
//! pairs exist so commands like `window_open_project` can implement the
//! FR-100 policy: opening a project that is already attached to a window
//! should focus that window instead of creating a second one — unless
//! the caller explicitly asks for a duplicate.
//!
//! The registry is intentionally a pure data structure: no Tauri runtime
//! is required. The command layer (`commands/window.rs`) holds the real
//! `WindowHandle` map behind a mutex and delegates all policy decisions
//! to this module.
//!
//! This keeps FR-101 (isolated per-window state) testable end-to-end
//! without spinning up a windowing system.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Snapshot of an open SpecLens window. Mirrors `WindowHandle` from the
/// `data-model.md §1` entity set but kept separate so this module has no
/// dependency on the on-disk config model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenWindow {
    /// Tauri-assigned window label / identifier. Unique per process.
    pub window_id: String,
    /// Project currently bound to the window, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<Uuid>,
}

impl OpenWindow {
    pub fn new(window_id: impl Into<String>, project_id: Option<Uuid>) -> Self {
        Self {
            window_id: window_id.into(),
            project_id,
        }
    }
}

/// Decision returned by [`WindowRegistry::plan_open`].
///
/// Commands inspect this enum to decide whether to bring an existing
/// window to front or to call `tauri::WebviewWindowBuilder::new`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenPlan {
    /// A window already owns this project — the command should focus the
    /// named window instead of creating a new one.
    FocusExisting { window_id: String },
    /// No window owns this project yet — the command should create a new
    /// top-level window and register it via [`WindowRegistry::register`].
    CreateNew,
}

/// In-memory window ↔ project map. Cheap to clone (a small `BTreeMap`).
#[derive(Debug, Default, Clone)]
pub struct WindowRegistry {
    by_id: BTreeMap<String, OpenWindow>,
}

impl WindowRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a deterministic snapshot of every open window ordered by
    /// `window_id`. Used by `window_list` and tests.
    pub fn list(&self) -> Vec<OpenWindow> {
        self.by_id.values().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Insert / replace a window record. Returns `true` when the record
    /// was new.
    pub fn register(&mut self, window: OpenWindow) -> bool {
        self.by_id
            .insert(window.window_id.clone(), window)
            .is_none()
    }

    /// Drop the window with the given id. Returns `true` if it existed.
    pub fn unregister(&mut self, window_id: &str) -> bool {
        self.by_id.remove(window_id).is_some()
    }

    /// Re-point an existing window at a different project (or detach it
    /// by passing `None`). Returns `false` if the id is unknown.
    pub fn attach_project(&mut self, window_id: &str, project_id: Option<Uuid>) -> bool {
        if let Some(entry) = self.by_id.get_mut(window_id) {
            entry.project_id = project_id;
            true
        } else {
            false
        }
    }

    /// Find the first window bound to `project_id`, if any.
    pub fn find_by_project(&self, project_id: Uuid) -> Option<&OpenWindow> {
        self.by_id
            .values()
            .find(|w| w.project_id == Some(project_id))
    }

    /// Implement the FR-100 focus-existing policy. When `allow_duplicate`
    /// is `true` the caller explicitly requested a second window onto the
    /// same project, so we bypass the check.
    pub fn plan_open(&self, project_id: Uuid, allow_duplicate: bool) -> OpenPlan {
        if !allow_duplicate {
            if let Some(existing) = self.find_by_project(project_id) {
                return OpenPlan::FocusExisting {
                    window_id: existing.window_id.clone(),
                };
            }
        }
        OpenPlan::CreateNew
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pid() -> Uuid {
        Uuid::new_v4()
    }

    #[test]
    fn empty_registry_always_plans_create() {
        let reg = WindowRegistry::new();
        assert_eq!(reg.plan_open(pid(), false), OpenPlan::CreateNew);
        assert!(reg.is_empty());
    }

    #[test]
    fn register_and_list_are_deterministic() {
        let mut reg = WindowRegistry::new();
        let p1 = pid();
        let p2 = pid();
        assert!(reg.register(OpenWindow::new("win-2", Some(p2))));
        assert!(reg.register(OpenWindow::new("win-1", Some(p1))));
        let list = reg.list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].window_id, "win-1");
        assert_eq!(list[1].window_id, "win-2");
    }

    #[test]
    fn register_overwrites_existing_returns_false() {
        let mut reg = WindowRegistry::new();
        let p = pid();
        assert!(reg.register(OpenWindow::new("win-1", None)));
        assert!(!reg.register(OpenWindow::new("win-1", Some(p))));
        assert_eq!(reg.list()[0].project_id, Some(p));
    }

    #[test]
    fn unregister_removes_entry() {
        let mut reg = WindowRegistry::new();
        reg.register(OpenWindow::new("win-1", Some(pid())));
        assert!(reg.unregister("win-1"));
        assert!(reg.is_empty());
        assert!(!reg.unregister("win-1"));
    }

    #[test]
    fn attach_project_updates_entry() {
        let mut reg = WindowRegistry::new();
        let p1 = pid();
        let p2 = pid();
        reg.register(OpenWindow::new("win-1", Some(p1)));
        assert!(reg.attach_project("win-1", Some(p2)));
        assert_eq!(reg.find_by_project(p2).unwrap().window_id, "win-1");
        assert!(reg.find_by_project(p1).is_none());
        assert!(!reg.attach_project("win-missing", None));
    }

    #[test]
    fn plan_open_returns_focus_for_existing_project() {
        let mut reg = WindowRegistry::new();
        let p = pid();
        reg.register(OpenWindow::new("win-1", Some(p)));
        assert_eq!(
            reg.plan_open(p, false),
            OpenPlan::FocusExisting {
                window_id: "win-1".into()
            }
        );
    }

    #[test]
    fn plan_open_allow_duplicate_bypasses_focus() {
        let mut reg = WindowRegistry::new();
        let p = pid();
        reg.register(OpenWindow::new("win-1", Some(p)));
        assert_eq!(reg.plan_open(p, true), OpenPlan::CreateNew);
    }

    #[test]
    fn plan_open_unique_project_creates_new() {
        let mut reg = WindowRegistry::new();
        reg.register(OpenWindow::new("win-1", Some(pid())));
        let other = pid();
        assert_eq!(reg.plan_open(other, false), OpenPlan::CreateNew);
    }

    #[test]
    fn detach_project_releases_focus_slot() {
        let mut reg = WindowRegistry::new();
        let p = pid();
        reg.register(OpenWindow::new("win-1", Some(p)));
        reg.attach_project("win-1", None);
        assert_eq!(reg.plan_open(p, false), OpenPlan::CreateNew);
    }
}
