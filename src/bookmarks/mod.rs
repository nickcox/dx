pub mod storage;

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::common;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BookmarkStore {
    bookmarks: BTreeMap<String, PathBuf>,
}

#[derive(Debug, Error)]
pub enum BookmarkError {
    #[error("invalid bookmark name: {0}")]
    InvalidName(String),
    #[error("bookmark path does not exist: {0}")]
    PathNotFound(String),
    #[error("bookmark path is not a directory: {0}")]
    PathNotDirectory(String),
    #[error("failed to canonicalize bookmark path {path}: {source}")]
    CanonicalizePath { path: String, source: io::Error },
    #[error("bookmark not found: {0}")]
    NotFound(String),
}

impl BookmarkStore {
    pub fn from_paths(bookmarks: BTreeMap<String, PathBuf>) -> Self {
        Self { bookmarks }
    }

    pub fn is_empty(&self) -> bool {
        self.bookmarks.is_empty()
    }

    pub fn set(&mut self, name: &str, path: &Path) -> Result<PathBuf, BookmarkError> {
        validate_name(name)?;

        let canonical = match fs::canonicalize(path) {
            Ok(value) => value,
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                return Err(BookmarkError::PathNotFound(path.display().to_string()))
            }
            Err(source) => {
                return Err(BookmarkError::CanonicalizePath {
                    path: path.display().to_string(),
                    source,
                })
            }
        };

        if !canonical.is_dir() {
            return Err(BookmarkError::PathNotDirectory(
                canonical.display().to_string(),
            ));
        }

        self.bookmarks.insert(name.to_string(), canonical.clone());
        Ok(canonical)
    }

    pub fn remove(&mut self, name: &str) -> Result<PathBuf, BookmarkError> {
        validate_name(name)?;
        self.bookmarks
            .remove(name)
            .ok_or_else(|| BookmarkError::NotFound(name.to_string()))
    }

    pub fn get(&self, name: &str) -> Option<PathBuf> {
        if !is_valid_name(name) {
            return None;
        }

        let path = self.bookmarks.get(name)?;
        if path.is_dir() {
            Some(path.clone())
        } else {
            None
        }
    }

    pub fn list(&self) -> Vec<(String, PathBuf)> {
        self.bookmarks
            .iter()
            .map(|(name, path)| (name.clone(), path.clone()))
            .collect()
    }
}

pub fn validate_name(name: &str) -> Result<(), BookmarkError> {
    if is_valid_name(name) {
        return Ok(());
    }

    Err(BookmarkError::InvalidName(name.to_string()))
}

pub fn lookup(name: &str) -> Option<PathBuf> {
    let store = storage::read_store().ok()?;
    store.get(name)
}

fn is_valid_name(name: &str) -> bool {
    common::is_valid_identifier(name)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{validate_name, BookmarkError, BookmarkStore};

    fn make_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "dx-bookmarks-{label}-{nonce}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn accepts_valid_bookmark_names() {
        validate_name("my-project").expect("valid name");
        validate_name("docs_v2").expect("valid name");
        validate_name("A1").expect("valid name");
    }

    #[test]
    fn rejects_invalid_bookmark_names() {
        let invalid = ["../hack", "foo/bar", "~home", "has space", "", "."];
        for name in invalid {
            let err = validate_name(name).expect_err("invalid name should fail");
            assert!(matches!(err, BookmarkError::InvalidName(_)));
        }
    }

    #[test]
    fn set_with_explicit_path_succeeds() {
        let temp = make_temp_dir("set-explicit");
        let target = temp.join("project");
        fs::create_dir_all(&target).expect("create project dir");

        let mut store = BookmarkStore::default();
        let output = store.set("proj", &target).expect("set bookmark");

        assert_eq!(output, fs::canonicalize(&target).expect("canonical target"));
        assert_eq!(store.get("proj"), Some(output));

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn set_overwrites_existing_bookmark() {
        let temp = make_temp_dir("set-overwrite");
        let first = temp.join("first");
        let second = temp.join("second");
        fs::create_dir_all(&first).expect("create first dir");
        fs::create_dir_all(&second).expect("create second dir");

        let mut store = BookmarkStore::default();
        let _ = store.set("proj", &first).expect("set first");
        let output = store.set("proj", &second).expect("set second");

        assert_eq!(output, fs::canonicalize(&second).expect("canonical second"));
        assert_eq!(store.get("proj"), Some(output));

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn set_rejects_nonexistent_path() {
        let temp = make_temp_dir("set-missing");
        let missing = temp.join("missing");

        let mut store = BookmarkStore::default();
        let err = store
            .set("proj", &missing)
            .expect_err("missing path should fail");
        assert!(matches!(err, BookmarkError::PathNotFound(_)));

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn remove_existing_bookmark_succeeds() {
        let temp = make_temp_dir("remove-existing");
        let target = temp.join("target");
        fs::create_dir_all(&target).expect("create target");

        let mut store = BookmarkStore::default();
        let canonical = store.set("proj", &target).expect("set bookmark");
        let removed = store.remove("proj").expect("remove bookmark");

        assert_eq!(removed, canonical);
        assert!(store.get("proj").is_none());

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn remove_nonexistent_bookmark_fails() {
        let mut store = BookmarkStore::default();
        let err = store.remove("missing").expect_err("remove should fail");
        assert!(matches!(err, BookmarkError::NotFound(_)));
    }

    #[test]
    fn get_returns_none_for_stale_path() {
        let temp = make_temp_dir("stale");
        let target = temp.join("target");
        fs::create_dir_all(&target).expect("create target");

        let mut store = BookmarkStore::default();
        let _ = store.set("proj", &target).expect("set bookmark");
        fs::remove_dir_all(&target).expect("remove target");

        assert!(store.get("proj").is_none());
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn list_is_sorted_by_name() {
        let temp = make_temp_dir("list-sorted");
        let a = temp.join("a");
        let b = temp.join("b");
        fs::create_dir_all(&a).expect("create a");
        fs::create_dir_all(&b).expect("create b");

        let mut store = BookmarkStore::default();
        let _ = store.set("zeta", &b).expect("set zeta");
        let _ = store.set("alpha", &a).expect("set alpha");

        let entries = store.list();
        assert_eq!(entries[0].0, "alpha");
        assert_eq!(entries[1].0, "zeta");

        let _ = fs::remove_dir_all(temp);
    }
}
