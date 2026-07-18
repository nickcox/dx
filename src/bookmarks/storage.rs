use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{BookmarkStore, validate_name};
use crate::common;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("failed to read bookmark store {path}: {source}")]
    ReadStore { path: String, source: io::Error },
    #[error("failed to parse bookmark store {path}: {source}")]
    ParseStore {
        path: String,
        source: toml::de::Error,
    },
    #[error("failed to create bookmark store directory {path}: {source}")]
    CreateStoreDir { path: String, source: io::Error },
    #[error("failed to serialize bookmark store: {0}")]
    SerializeStore(toml::ser::Error),
    #[error("failed to write bookmark store {path}: {source}")]
    WriteStore { path: String, source: io::Error },
    #[error("failed to replace bookmark store {to} from {from}: {source}")]
    ReplaceStore {
        from: String,
        to: String,
        source: io::Error,
    },
    #[error("invalid bookmark {name:?} in store {path}: {reason}")]
    InvalidBookmark {
        path: String,
        name: String,
        reason: String,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookmarkFile {
    #[serde(default)]
    pub bookmarks: BTreeMap<String, String>,
}

pub fn bookmark_file_path() -> PathBuf {
    bookmark_override_path()
        .or_else(bookmark_xdg_data_home_path)
        .or_else(bookmark_data_dir_path)
        .unwrap_or_else(|| env::temp_dir().join("dx").join("bookmarks.toml"))
}

pub fn read_store() -> Result<BookmarkStore, StorageError> {
    let path = bookmark_file_path();
    let raw = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Ok(BookmarkStore::default());
        }
        Err(source) => {
            return Err(StorageError::ReadStore {
                path: path.display().to_string(),
                source,
            });
        }
    };

    let parsed =
        toml::from_str::<BookmarkFile>(&raw).map_err(|source| StorageError::ParseStore {
            path: path.display().to_string(),
            source,
        })?;

    let mut bookmarks = BTreeMap::new();
    for (name, value) in parsed.bookmarks {
        validate_name(&name).map_err(|error| StorageError::InvalidBookmark {
            path: path.display().to_string(),
            name: name.clone(),
            reason: error.to_string(),
        })?;
        let bookmark_path = PathBuf::from(value);
        if !bookmark_path.is_absolute() {
            return Err(StorageError::InvalidBookmark {
                path: path.display().to_string(),
                name,
                reason: "path must be absolute".to_string(),
            });
        }
        bookmarks.insert(name, bookmark_path);
    }

    Ok(BookmarkStore::from_paths(bookmarks))
}

pub fn write_store(store: &BookmarkStore) -> Result<(), StorageError> {
    let target = bookmark_file_path();

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|source| StorageError::CreateStoreDir {
            path: parent.display().to_string(),
            source,
        })?;
    }

    let payload = BookmarkFile {
        bookmarks: store.to_serializable_map(),
    };
    let raw = toml::to_string(&payload).map_err(StorageError::SerializeStore)?;

    common::write_atomic_replace(&target, raw.as_bytes()).map_err(|err| {
        common::map_atomic_write_error(
            err,
            |source| StorageError::WriteStore {
                path: target.display().to_string(),
                source,
            },
            |source| StorageError::ReplaceStore {
                from: target.display().to_string(),
                to: target.display().to_string(),
                source,
            },
        )
    })
}

fn non_empty_env_path(name: &str) -> Option<PathBuf> {
    let value = env::var(name).ok()?;
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
}

fn bookmark_override_path() -> Option<PathBuf> {
    non_empty_env_path("DX_BOOKMARKS_FILE")
}

fn bookmark_xdg_data_home_path() -> Option<PathBuf> {
    non_empty_env_path("XDG_DATA_HOME").map(|path| path.join("dx").join("bookmarks.toml"))
}

fn bookmark_data_dir_path() -> Option<PathBuf> {
    dirs::data_dir().map(|path| path.join("dx").join("bookmarks.toml"))
}


#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::test_support::{self, ScopedProcess, TempDir};

    use super::*;

    fn make_temp_dir(label: &str) -> TempDir {
        test_support::temp_dir(&format!("bookmark-store-{label}"))
    }

    #[test]
    fn bookmark_file_path_prefers_dx_bookmarks_file_override() {
        let mut process = ScopedProcess::new();
        let temp = make_temp_dir("path-override");
        let override_path = temp.path().join("custom/bookmarks.toml");

        process.set("DX_BOOKMARKS_FILE", &override_path);
        process.set("XDG_DATA_HOME", temp.path().join("xdg"));

        let path = bookmark_file_path();
        assert_eq!(path, override_path);
    }

    #[test]
    fn bookmark_file_path_uses_xdg_data_home_when_override_unset() {
        let mut process = ScopedProcess::new();
        let temp = make_temp_dir("path-xdg");

        process.remove("DX_BOOKMARKS_FILE");
        process.set("XDG_DATA_HOME", temp.path());

        let path = bookmark_file_path();
        assert_eq!(path, temp.path().join("dx").join("bookmarks.toml"));
    }

    #[test]
    fn read_missing_file_returns_empty_store() {
        let mut process = ScopedProcess::new();
        let temp = make_temp_dir("read-missing");
        let file = temp.path().join("bookmarks.toml");

        process.set("DX_BOOKMARKS_FILE", &file);
        process.remove("XDG_DATA_HOME");

        let store = read_store().expect("read missing store");
        assert!(store.is_empty());
    }

    #[test]
    fn write_then_read_round_trip_preserves_bookmarks() {
        let mut process = ScopedProcess::new();
        let temp = make_temp_dir("round-trip");
        let file = temp.path().join("bookmarks.toml");

        let first = temp.path().join("a");
        let second = temp.path().join("b");
        fs::create_dir_all(&first).expect("create first");
        fs::create_dir_all(&second).expect("create second");

        let mut map = BTreeMap::new();
        map.insert(
            "alpha".to_string(),
            fs::canonicalize(&first).expect("canonical first"),
        );
        map.insert(
            "beta".to_string(),
            fs::canonicalize(&second).expect("canonical second"),
        );
        let store = BookmarkStore::from_paths(map);

        process.set("DX_BOOKMARKS_FILE", &file);
        process.remove("XDG_DATA_HOME");
        write_store(&store).expect("write store");
        let loaded = read_store().expect("read store");

        assert_eq!(loaded, store);
    }

    #[test]
    fn read_corrupt_file_returns_error() {
        let mut process = ScopedProcess::new();
        let temp = make_temp_dir("corrupt");
        let file = temp.path().join("bookmarks.toml");
        fs::write(&file, "{invalid toml").expect("write corrupt file");

        process.set("DX_BOOKMARKS_FILE", &file);
        process.remove("XDG_DATA_HOME");

        let err = read_store().expect_err("corrupt file should fail");
        assert!(matches!(err, StorageError::ParseStore { .. }));
    }

    #[test]
    fn read_store_rejects_invalid_names_and_relative_paths() {
        let mut process = ScopedProcess::new();
        let temp = make_temp_dir("invalid-values");
        let file = temp.path().join("bookmarks.toml");
        process.set("DX_BOOKMARKS_FILE", &file);

        fs::write(&file, "[bookmarks]\n'bad name' = '/tmp'\n").expect("write invalid name store");
        assert!(matches!(
            read_store(),
            Err(StorageError::InvalidBookmark { .. })
        ));

        fs::write(&file, "[bookmarks]\nvalid = 'relative/path'\n")
            .expect("write relative path store");
        assert!(matches!(
            read_store(),
            Err(StorageError::InvalidBookmark { .. })
        ));
    }

    #[test]
    fn write_store_creates_parent_directory() {
        let mut process = ScopedProcess::new();
        let temp = make_temp_dir("create-parent");
        let file = temp.path().join("nested/path/bookmarks.toml");

        process.set("DX_BOOKMARKS_FILE", &file);
        process.remove("XDG_DATA_HOME");

        let store = BookmarkStore::default();
        write_store(&store).expect("write empty store");

        assert!(file.exists());
    }

    #[test]
    fn write_store_replace_failure_preserves_last_known_good_target() {
        let mut process = ScopedProcess::new();
        let temp = make_temp_dir("replace-failure");
        let file = temp.path().join("bookmarks.toml");
        let original = "[bookmarks]\nalpha = \"/persisted\"\n";
        fs::write(&file, original).expect("seed existing bookmark store");

        let target = temp.path().join("next");
        fs::create_dir_all(&target).expect("create bookmark target");

        let mut map = BTreeMap::new();
        map.insert(
            "beta".to_string(),
            fs::canonicalize(&target).expect("canonical target"),
        );
        let store = BookmarkStore::from_paths(map);

        process.set("DX_BOOKMARKS_FILE", &file);
        process.remove("XDG_DATA_HOME");

        let err = crate::common::with_replace_failure_injection_for_tests(|| write_store(&store))
            .expect_err("replace failure should surface");
        assert!(matches!(err, StorageError::ReplaceStore { .. }));

        let raw = fs::read_to_string(&file).expect("read persisted target after failure");
        assert_eq!(raw, original);
    }
}
