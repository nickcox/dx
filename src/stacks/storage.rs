use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use thiserror::Error;

use super::SessionStack;

pub const DEFAULT_STALE_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("invalid session id: {0}")]
    InvalidSessionId(String),
    #[error("failed to create session directory {path}: {source}")]
    CreateSessionDir { path: String, source: io::Error },
    #[error("failed to read session file {path}: {source}")]
    ReadSession { path: String, source: io::Error },
    #[error("failed to write session file {path}: {source}")]
    WriteSession { path: String, source: io::Error },
    #[error("failed to replace session file {to} from {from}: {source}")]
    ReplaceSession {
        from: String,
        to: String,
        source: io::Error,
    },
    #[error("failed to serialize session json: {0}")]
    SerializeSession(serde_json::Error),
}

pub fn session_directory() -> PathBuf {
    match env::var("XDG_RUNTIME_DIR") {
        Ok(value) if !value.trim().is_empty() => PathBuf::from(value).join("dx-sessions"),
        _ => env::temp_dir().join("dx-sessions"),
    }
}

pub fn ensure_session_dir() -> Result<PathBuf, StorageError> {
    let dir = session_directory();
    fs::create_dir_all(&dir).map_err(|source| StorageError::CreateSessionDir {
        path: dir.display().to_string(),
        source,
    })?;
    Ok(dir)
}

pub fn read_session(dir: &Path, session_id: &str) -> Result<SessionStack, StorageError> {
    cleanup_stale(dir, DEFAULT_STALE_TTL);
    let path = session_file_path(dir, session_id)?;

    let raw = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(SessionStack::default()),
        Err(source) => {
            return Err(StorageError::ReadSession {
                path: path.display().to_string(),
                source,
            })
        }
    };

    let mut stack = match serde_json::from_str::<SessionStack>(&raw) {
        Ok(value) => value,
        Err(_) => return Ok(SessionStack::default()),
    };
    stack.sanitize();
    Ok(stack)
}

pub fn write_session(
    dir: &Path,
    session_id: &str,
    stack: &SessionStack,
) -> Result<(), StorageError> {
    cleanup_stale(dir, DEFAULT_STALE_TTL);
    fs::create_dir_all(dir).map_err(|source| StorageError::CreateSessionDir {
        path: dir.display().to_string(),
        source,
    })?;

    let target = session_file_path(dir, session_id)?;
    let temp = temp_session_path(dir, session_id);
    let payload = serde_json::to_vec(stack).map_err(StorageError::SerializeSession)?;

    fs::write(&temp, payload).map_err(|source| StorageError::WriteSession {
        path: temp.display().to_string(),
        source,
    })?;

    match fs::rename(&temp, &target) {
        Ok(()) => Ok(()),
        Err(source) => {
            if target.exists() {
                let _ = fs::remove_file(&target);
                if fs::rename(&temp, &target).is_ok() {
                    return Ok(());
                }
            }
            let _ = fs::remove_file(&temp);
            Err(StorageError::ReplaceSession {
                from: temp.display().to_string(),
                to: target.display().to_string(),
                source,
            })
        }
    }
}

pub fn cleanup_stale(dir: &Path, ttl: Duration) {
    let entries = match fs::read_dir(dir) {
        Ok(value) => value,
        Err(_) => return,
    };

    let now = SystemTime::now();
    for entry_result in entries {
        let entry = match entry_result {
            Ok(value) => value,
            Err(_) => continue,
        };

        let path = entry.path();
        if !is_session_file(&path) {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(value) => value,
            Err(_) => continue,
        };

        let modified = match metadata.modified() {
            Ok(value) => value,
            Err(_) => continue,
        };

        let age = match now.duration_since(modified) {
            Ok(value) => value,
            Err(_) => continue,
        };

        if age > ttl {
            let _ = fs::remove_file(path);
        }
    }
}

fn session_file_path(dir: &Path, session_id: &str) -> Result<PathBuf, StorageError> {
    if !is_valid_session_id(session_id) {
        return Err(StorageError::InvalidSessionId(session_id.to_string()));
    }
    Ok(dir.join(format!("{session_id}.json")))
}

fn temp_session_path(dir: &Path, session_id: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    dir.join(format!(
        ".{session_id}.{}.{}.tmp",
        std::process::id(),
        nonce
    ))
}

fn is_session_file(path: &Path) -> bool {
    if path.extension().and_then(|value| value.to_str()) != Some("json") {
        return false;
    }
    path.file_stem()
        .and_then(|value| value.to_str())
        .map(is_valid_session_id)
        .unwrap_or(false)
}

fn is_valid_session_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'_')
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::test_support;

    use super::*;

    fn make_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path =
            env::temp_dir().join(format!("dx-stacks-{label}-{nonce}-{}", std::process::id()));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        test_support::env_lock()
    }

    #[test]
    fn session_directory_prefers_xdg_runtime_when_set() {
        let _guard = env_lock();
        let runtime = make_temp_dir("xdg");
        env::set_var("XDG_RUNTIME_DIR", runtime.display().to_string());

        let dir = session_directory();
        assert_eq!(dir, runtime.join("dx-sessions"));

        env::remove_var("XDG_RUNTIME_DIR");
        let _ = fs::remove_dir_all(runtime);
    }

    #[test]
    fn session_directory_falls_back_to_temp_dir() {
        let _guard = env_lock();
        env::remove_var("XDG_RUNTIME_DIR");

        let dir = session_directory();
        assert_eq!(dir, env::temp_dir().join("dx-sessions"));
    }

    #[test]
    fn read_missing_file_returns_empty_session() {
        let dir = make_temp_dir("read-missing");
        let stack = read_session(&dir, "123").expect("read session");
        assert_eq!(stack, SessionStack::default());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn read_corrupt_file_returns_default_and_write_overwrites() {
        let dir = make_temp_dir("read-corrupt");
        let file = dir.join("123.json");
        fs::write(&file, "{invalid json").expect("write corrupt file");

        let stack = read_session(&dir, "123").expect("read session");
        assert_eq!(stack, SessionStack::default());

        let mut next = SessionStack::default();
        next.push(PathBuf::from("/home/user")).expect("push path");
        write_session(&dir, "123", &next).expect("write session");

        let raw = fs::read_to_string(&file).expect("read repaired file");
        let parsed = serde_json::from_str::<SessionStack>(&raw).expect("parse repaired file");
        assert_eq!(parsed.cwd, Some(PathBuf::from("/home/user")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn write_then_read_round_trip_succeeds() {
        let dir = make_temp_dir("write-read");
        let mut stack = SessionStack::default();
        stack.push(PathBuf::from("/a")).expect("push cwd");
        stack.push(PathBuf::from("/b")).expect("push cwd");

        write_session(&dir, "200", &stack).expect("write session");
        let loaded = read_session(&dir, "200").expect("read session");

        assert_eq!(loaded, stack);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cleanup_removes_files_older_than_ttl() {
        let dir = make_temp_dir("cleanup-old");
        let stale = dir.join("old_1.json");
        fs::write(&stale, "{}").expect("write stale file");

        thread::sleep(Duration::from_millis(5));
        cleanup_stale(&dir, Duration::from_secs(0));

        assert!(!stale.exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cleanup_preserves_recent_files() {
        let dir = make_temp_dir("cleanup-recent");
        let recent = dir.join("recent_1.json");
        fs::write(&recent, "{}").expect("write recent file");

        cleanup_stale(&dir, Duration::from_secs(60 * 60));

        assert!(recent.exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cleanup_skips_non_session_files() {
        let dir = make_temp_dir("cleanup-pattern");
        let non_session_json = dir.join("bad$.json");
        let lock_file = dir.join("active.lock");
        let temp_file = dir.join("session.tmp");

        fs::write(&non_session_json, "{}").expect("write bad json");
        fs::write(&lock_file, "lock").expect("write lock");
        fs::write(&temp_file, "tmp").expect("write tmp");

        thread::sleep(Duration::from_millis(5));
        cleanup_stale(&dir, Duration::from_secs(0));

        assert!(non_session_json.exists());
        assert!(lock_file.exists());
        assert!(temp_file.exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cleanup_permission_errors_do_not_propagate() {
        let dir = make_temp_dir("cleanup-perm");
        let file = dir.join("not-a-dir");
        fs::write(&file, "x").expect("write file");

        cleanup_stale(&file, Duration::from_secs(0));

        assert!(file.exists());
        let _ = fs::remove_dir_all(dir);
    }
}
