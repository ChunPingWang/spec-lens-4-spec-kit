//! SHA-256 helpers + doc-hash computation for Phase 4 US2.
//!
//! The backing `.speclens/hashes.json` file is managed by [`StateStore`];
//! this module keeps the hash algorithm + path resolution isolated so the
//! scanner and commands can call into a single place.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::{SpecLensError, SpecLensResult};
use crate::models::DocHashRecord;

/// Compute a lowercase hex SHA-256 digest of a file on disk.
pub fn compute_sha256_hex(path: impl AsRef<Path>) -> SpecLensResult<String> {
    let path = path.as_ref();
    let bytes =
        std::fs::read(path).map_err(|e| SpecLensError::io(path.display().to_string(), e))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

/// Build a [`DocHashRecord`] for a given absolute file path, using
/// `relative_path` as the record key (what the frontend sees).
pub fn record_for(
    absolute: impl AsRef<Path>,
    relative_path: impl Into<String>,
) -> SpecLensResult<DocHashRecord> {
    let sha = compute_sha256_hex(absolute.as_ref())?;
    Ok(DocHashRecord::new(relative_path.into(), sha))
}

/// Resolve a relative path (as sent from the frontend) against the project
/// root, rejecting anything that escapes the root via `..` or absolute
/// paths. See Constitution "Path safety" and FR-085.
pub fn resolve_inside_root(root: &Path, relative: &str) -> SpecLensResult<PathBuf> {
    let rel = Path::new(relative);
    if rel.is_absolute() {
        return Err(SpecLensError::PathNotReadable(format!(
            "absolute path not allowed: {relative}"
        )));
    }
    let mut resolved = root.to_path_buf();
    for component in rel.components() {
        use std::path::Component::*;
        match component {
            Normal(seg) => resolved.push(seg),
            CurDir => {}
            ParentDir | RootDir | Prefix(_) => {
                return Err(SpecLensError::PathNotReadable(format!(
                    "path escapes project root: {relative}"
                )));
            }
        }
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn sha256_matches_known_vector() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("a.txt");
        fs::write(&path, b"abc").unwrap();
        let sha = compute_sha256_hex(&path).unwrap();
        // echo -n "abc" | shasum -a 256
        assert_eq!(
            sha,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn record_for_sets_path_and_hex_length_64() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("doc.md");
        fs::write(&path, b"hello\n").unwrap();
        let rec = record_for(&path, "doc.md".to_string()).unwrap();
        assert_eq!(rec.path, "doc.md");
        assert_eq!(rec.sha256.len(), 64);
        assert!(rec.is_valid_sha256());
    }

    #[test]
    fn resolve_inside_root_accepts_sub_path() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let resolved = resolve_inside_root(root, "specs/001/spec.md").unwrap();
        assert!(resolved.starts_with(root));
        assert!(resolved.ends_with("spec.md"));
    }

    #[test]
    fn resolve_inside_root_rejects_parent_escape() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let err = resolve_inside_root(root, "../secrets.txt").unwrap_err();
        assert!(matches!(err, SpecLensError::PathNotReadable(_)));
    }

    #[test]
    fn resolve_inside_root_rejects_absolute_path() {
        let tmp = TempDir::new().unwrap();
        let err = resolve_inside_root(tmp.path(), "/etc/passwd").unwrap_err();
        assert!(matches!(err, SpecLensError::PathNotReadable(_)));
    }
}
