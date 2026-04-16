use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::BookmarkStore;
use crate::common::{self, AtomicWriteError};

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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookmarkFile {
    #[serde(default)]
    pub bookmarks: BTreeMap<String, String>,
}

pub fn bookmark_file_path() -> PathBuf {
    if let Ok(value) = env::var("DX_BOOKMARKS_FILE") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    if let Ok(value) = env::var("XDG_DATA_HOME") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("dx").join("bookmarks.toml");
        }
    }

    if let Some(path) = dirs::data_dir() {
        return path.join("dx").join("bookmarks.toml");
    }

    env::temp_dir().join("dx").join("bookmarks.toml")
}

pub fn read_store() -> Result<BookmarkStore, StorageError> {
    let path = bookmark_file_path();
    let raw = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Ok(BookmarkStore::default())
        }
        Err(source) => {
            return Err(StorageError::ReadStore {
                path: path.display().to_string(),
                source,
            })
        }
    };

    let parsed =
        toml::from_str::<BookmarkFile>(&raw).map_err(|source| StorageError::ParseStore {
            path: path.display().to_string(),
            source,
        })?;

    let bookmarks = parsed
        .bookmarks
        .into_iter()
        .map(|(name, value)| (name, PathBuf::from(value)))
        .collect::<BTreeMap<_, _>>();

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
        bookmarks: store
            .list()
            .into_iter()
            .map(|(name, path)| (name, path.display().to_string()))
            .collect::<BTreeMap<_, _>>(),
    };
    let raw = toml::to_string(&payload).map_err(StorageError::SerializeStore)?;

    let temp = temp_store_path(&target);
    common::write_atomic_replace(&temp, &target, raw.as_bytes()).map_err(|err| match err {
        AtomicWriteError::Write(source) => StorageError::WriteStore {
            path: temp.display().to_string(),
            source,
        },
        AtomicWriteError::Replace(source) => StorageError::ReplaceStore {
            from: temp.display().to_string(),
            to: target.display().to_string(),
            source,
        },
    })
}

fn temp_store_path(target: &Path) -> PathBuf {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    parent.join(format!(".bookmarks.{}.{}.tmp", std::process::id(), nonce))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::test_support;

    use super::*;

    fn make_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "dx-bookmark-store-{label}-{nonce}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        test_support::env_lock()
    }

    fn set_env(name: &str, value: Option<String>) {
        if let Some(value) = value {
            unsafe { env::set_var(name, value) };
        } else {
            unsafe { env::remove_var(name) };
        }
    }

    fn reset_env(dx_bookmarks_file: Option<String>, xdg_data_home: Option<String>) {
        set_env("DX_BOOKMARKS_FILE", dx_bookmarks_file);
        set_env("XDG_DATA_HOME", xdg_data_home);
    }

    #[test]
    fn bookmark_file_path_prefers_dx_bookmarks_file_override() {
        let _guard = env_lock();
        let temp = make_temp_dir("path-override");
        let override_path = temp.join("custom/bookmarks.toml");

        reset_env(
            Some(override_path.display().to_string()),
            Some(temp.join("xdg").display().to_string()),
        );

        let path = bookmark_file_path();
        assert_eq!(path, override_path);

        reset_env(None, None);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn bookmark_file_path_uses_xdg_data_home_when_override_unset() {
        let _guard = env_lock();
        let temp = make_temp_dir("path-xdg");

        reset_env(None, Some(temp.display().to_string()));

        let path = bookmark_file_path();
        assert_eq!(path, temp.join("dx").join("bookmarks.toml"));

        reset_env(None, None);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn read_missing_file_returns_empty_store() {
        let _guard = env_lock();
        let temp = make_temp_dir("read-missing");
        let file = temp.join("bookmarks.toml");

        reset_env(Some(file.display().to_string()), None);

        let store = read_store().expect("read missing store");
        assert!(store.is_empty());

        reset_env(None, None);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn write_then_read_round_trip_preserves_bookmarks() {
        let _guard = env_lock();
        let temp = make_temp_dir("round-trip");
        let file = temp.join("bookmarks.toml");

        let first = temp.join("a");
        let second = temp.join("b");
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

        reset_env(Some(file.display().to_string()), None);
        write_store(&store).expect("write store");
        let loaded = read_store().expect("read store");

        assert_eq!(loaded, store);

        reset_env(None, None);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn read_corrupt_file_returns_error() {
        let _guard = env_lock();
        let temp = make_temp_dir("corrupt");
        let file = temp.join("bookmarks.toml");
        fs::write(&file, "{invalid toml").expect("write corrupt file");

        reset_env(Some(file.display().to_string()), None);

        let err = read_store().expect_err("corrupt file should fail");
        assert!(matches!(err, StorageError::ParseStore { .. }));

        reset_env(None, None);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn write_store_creates_parent_directory() {
        let _guard = env_lock();
        let temp = make_temp_dir("create-parent");
        let file = temp.join("nested/path/bookmarks.toml");

        reset_env(Some(file.display().to_string()), None);

        let store = BookmarkStore::default();
        write_store(&store).expect("write empty store");

        assert!(file.exists());

        reset_env(None, None);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn write_store_replace_failure_preserves_last_known_good_target() {
        let _guard = env_lock();
        let temp = make_temp_dir("replace-failure");
        let file = temp.join("bookmarks.toml");
        let original = "[bookmarks]\nalpha = \"/persisted\"\n";
        fs::write(&file, original).expect("seed existing bookmark store");

        let target = temp.join("next");
        fs::create_dir_all(&target).expect("create bookmark target");

        let mut map = BTreeMap::new();
        map.insert(
            "beta".to_string(),
            fs::canonicalize(&target).expect("canonical target"),
        );
        let store = BookmarkStore::from_paths(map);

        reset_env(Some(file.display().to_string()), None);

        let err = crate::common::with_replace_failure_injection_for_tests(|| write_store(&store))
            .expect_err("replace failure should surface");
        assert!(matches!(err, StorageError::ReplaceStore { .. }));

        let raw = fs::read_to_string(&file).expect("read persisted target after failure");
        assert_eq!(raw, original);

        reset_env(None, None);
        let _ = fs::remove_dir_all(temp);
    }
}
