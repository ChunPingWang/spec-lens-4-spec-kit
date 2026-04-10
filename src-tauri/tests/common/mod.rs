//! Shared helpers for integration tests in `src-tauri/tests/*`.
//!
//! Tests that need a real Spec-Kit layout on disk can call
//! [`fixture_project_ok`] to copy the canonical fixture from
//! `<repo>/tests/fixtures/speckit-project-ok` into a freshly created
//! temporary directory. This keeps individual tests hermetic.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::{fs, io};

use tempfile::TempDir;

/// Repository-relative path to the bundled fixtures directory.
pub fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("tests")
        .join("fixtures")
}

/// Recursively copy `src` into `dst`, creating directories as needed.
pub fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else if ty.is_file() {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Copy the canonical "well-formed project" fixture into a fresh temp dir.
pub fn fixture_project_ok() -> (TempDir, PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().join("speckit-project-ok");
    copy_dir_all(&fixtures_root().join("speckit-project-ok"), &root).expect("copy fixture");
    (tmp, root)
}

/// Copy the empty-but-initialised fixture into a fresh temp dir.
pub fn fixture_project_empty() -> (TempDir, PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().join("speckit-project-empty");
    copy_dir_all(&fixtures_root().join("speckit-project-empty"), &root).expect("copy fixture");
    (tmp, root)
}
