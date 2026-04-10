//! Debounced filesystem watcher for `.specify/` + `specs/`.
//!
//! Wraps `notify` + `notify-debouncer-full` and emits high-level
//! [`FsWatcherEvent`]s the command layer forwards to the frontend via
//! Tauri events. See `research.md D5` and `contracts/ipc.md` events
//! `phase_documents_changed` / `task_files_changed`.

use std::any::Any;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult};

use crate::error::{SpecLensError, SpecLensResult};

/// Default debounce window — small enough to feel live, large enough to
/// coalesce editor save storms.
pub const DEFAULT_DEBOUNCE: Duration = Duration::from_millis(200);

/// High-level watcher event forwarded to IPC subscribers.
#[derive(Debug, Clone)]
pub enum FsWatcherEvent {
    /// One or more paths under `.specify/` or `specs/` changed.
    DocumentsChanged { paths: Vec<PathBuf> },
    /// The watcher reported an error (e.g. backend restart).
    Error { message: String },
}

/// Owns the debouncer handle and a receiver the command layer polls.
///
/// The concrete debouncer type varies between `notify-debouncer-full`
/// minor releases, so we erase it behind `Box<dyn Any>` — `Drop` still
/// runs through the trait object, tearing down the watcher thread.
pub struct FsWatcher {
    _debouncer: Box<dyn Any + Send>,
    rx: mpsc::Receiver<FsWatcherEvent>,
    root: PathBuf,
}

impl FsWatcher {
    /// Start watching `<project_root>/.specify` and `<project_root>/specs`
    /// recursively. Missing directories are skipped silently; the caller
    /// can recompute the subscription list after a project rescan.
    pub fn start(project_root: impl Into<PathBuf>) -> SpecLensResult<Self> {
        Self::start_with(project_root, DEFAULT_DEBOUNCE)
    }

    pub fn start_with(
        project_root: impl Into<PathBuf>,
        debounce: Duration,
    ) -> SpecLensResult<Self> {
        let root = project_root.into();
        let (tx, rx) = mpsc::channel::<FsWatcherEvent>();

        let forward_tx = tx.clone();
        let mut debouncer =
            new_debouncer(debounce, None, move |res: DebounceEventResult| match res {
                Ok(events) => {
                    let mut paths: Vec<PathBuf> = Vec::new();
                    for ev in events {
                        for p in ev.event.paths {
                            if !paths.iter().any(|existing| existing == &p) {
                                paths.push(p);
                            }
                        }
                    }
                    if !paths.is_empty() {
                        let _ = forward_tx.send(FsWatcherEvent::DocumentsChanged { paths });
                    }
                }
                Err(errs) => {
                    let message = errs
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join("; ");
                    let _ = forward_tx.send(FsWatcherEvent::Error { message });
                }
            })
            .map_err(|e| SpecLensError::Internal(format!("init notify debouncer: {e}")))?;

        for sub in [".specify", "specs"] {
            let dir = root.join(sub);
            if dir.exists() {
                debouncer
                    .watcher()
                    .watch(&dir, RecursiveMode::Recursive)
                    .map_err(|e| {
                        SpecLensError::Internal(format!("watch {}: {e}", dir.display()))
                    })?;
                debouncer.cache().add_root(&dir, RecursiveMode::Recursive);
            }
        }

        Ok(Self {
            _debouncer: Box::new(debouncer),
            rx,
            root,
        })
    }

    /// Non-blocking drain of pending events. Callers normally invoke this
    /// from a tokio task driven by a timer.
    pub fn drain(&self) -> Vec<FsWatcherEvent> {
        let mut out = Vec::new();
        while let Ok(ev) = self.rx.try_recv() {
            out.push(ev);
        }
        out
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread::sleep;
    use tempfile::TempDir;

    #[test]
    fn start_on_project_without_specify_dir_is_ok() {
        let tmp = TempDir::new().expect("tempdir");
        let watcher = FsWatcher::start(tmp.path().to_path_buf());
        assert!(watcher.is_ok(), "watcher should accept empty roots");
    }

    #[test]
    fn detects_new_file_under_specify_dir() {
        let tmp = TempDir::new().expect("tempdir");
        let specify = tmp.path().join(".specify");
        fs::create_dir_all(&specify).expect("create .specify");
        let watcher = FsWatcher::start_with(tmp.path().to_path_buf(), Duration::from_millis(50))
            .expect("start");

        let file = specify.join("feature.json");
        fs::write(&file, "{}").expect("write");
        // Give the debouncer a chance to flush.
        sleep(Duration::from_millis(400));
        let events = watcher.drain();
        let has_docs_change = events
            .iter()
            .any(|e| matches!(e, FsWatcherEvent::DocumentsChanged { paths } if paths.iter().any(|p| p.ends_with("feature.json"))));
        assert!(
            has_docs_change,
            "expected DocumentsChanged for feature.json, got {events:?}"
        );
    }
}
