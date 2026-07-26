//! Reading and writing per-session stack files, and pruning stale ones.

use std::env;
use std::fs;
use std::fs::DirEntry;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use thiserror::Error;

use super::SessionStack;
use crate::common;

pub const DEFAULT_STALE_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

const CLEANUP_INTERVAL: Duration = Duration::from_secs(60 * 60);
const CLEANUP_MARKER: &str = ".last-cleanup";

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
    #[error("failed to parse session file {path}: {source}")]
    ParseSession {
        path: String,
        source: serde_json::Error,
    },
    #[error("invalid session data in {path}: {source}")]
    InvalidSession {
        path: String,
        source: super::StackError,
    },
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
    let path = session_file_path(dir, session_id)?;
    maybe_cleanup_stale(dir);

    read_session_file(&path)
}

/// Sweeps at most once per process and once per [`CLEANUP_INTERVAL`] across
/// them; `dx stack push` runs on every prompt.
fn maybe_cleanup_stale(dir: &Path) {
    static SWEPT: AtomicBool = AtomicBool::new(false);
    if SWEPT.swap(true, Ordering::Relaxed) {
        return;
    }
    if !cleanup_is_due(dir) {
        return;
    }
    cleanup_stale(dir, DEFAULT_STALE_TTL);
}

/// Touches the marker before sweeping so concurrent shells do not all sweep.
fn cleanup_is_due(dir: &Path) -> bool {
    let marker = dir.join(CLEANUP_MARKER);
    if let Ok(modified) = fs::metadata(&marker).and_then(|meta| meta.modified())
        && modified.elapsed().is_ok_and(|age| age < CLEANUP_INTERVAL)
    {
        return false;
    }

    let _ = fs::write(&marker, b"");
    true
}

fn read_session_file(path: &Path) -> Result<SessionStack, StorageError> {
    let path_text = path.display().to_string();

    let raw = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(SessionStack::default()),
        Err(source) => {
            return Err(StorageError::ReadSession {
                path: path_text,
                source,
            });
        }
    };

    let stack = serde_json::from_str::<SessionStack>(&raw).map_err(|source| {
        StorageError::ParseSession {
            path: path_text.clone(),
            source,
        }
    })?;
    stack
        .validate()
        .map_err(|source| StorageError::InvalidSession {
            path: path_text,
            source,
        })?;
    Ok(stack)
}

pub fn write_session(
    dir: &Path,
    session_id: &str,
    stack: &SessionStack,
) -> Result<(), StorageError> {
    let target = session_file_path(dir, session_id)?;
    stack
        .validate()
        .map_err(|source| StorageError::InvalidSession {
            path: target.display().to_string(),
            source,
        })?;
    maybe_cleanup_stale(dir);
    fs::create_dir_all(dir).map_err(|source| StorageError::CreateSessionDir {
        path: dir.display().to_string(),
        source,
    })?;

    let payload = serde_json::to_vec(stack).map_err(StorageError::SerializeSession)?;

    common::write_atomic_replace(&target, &payload, common::Durability::Rename).map_err(|err| {
        common::map_atomic_write_error(
            err,
            |source| StorageError::WriteSession {
                path: target.display().to_string(),
                source,
            },
            |source| StorageError::ReplaceSession {
                from: target.display().to_string(),
                to: target.display().to_string(),
                source,
            },
        )
    })
}

pub fn cleanup_stale(dir: &Path, ttl: Duration) {
    let entries = match fs::read_dir(dir) {
        Ok(value) => value,
        Err(_) => return,
    };

    let now = SystemTime::now();
    for path in stale_session_paths(entries, now, ttl) {
        let _ = fs::remove_file(path);
    }
}

fn stale_session_paths(entries: fs::ReadDir, now: SystemTime, ttl: Duration) -> Vec<PathBuf> {
    let mut stale = Vec::new();

    for entry in entries.flatten() {
        if let Some(path) = stale_session_path(entry, now, ttl) {
            stale.push(path);
        }
    }

    stale
}

fn stale_session_path(entry: DirEntry, now: SystemTime, ttl: Duration) -> Option<PathBuf> {
    let path = entry.path();
    if !is_session_file(&path) {
        return None;
    }

    let modified = entry.metadata().ok()?.modified().ok()?;
    let age = now.duration_since(modified).ok()?;
    (age > ttl).then_some(path)
}

fn session_file_path(dir: &Path, session_id: &str) -> Result<PathBuf, StorageError> {
    if !is_valid_session_id(session_id) {
        return Err(StorageError::InvalidSessionId(session_id.to_string()));
    }
    Ok(dir.join(format!("{session_id}.json")))
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
    common::is_valid_identifier(value)
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::test_support::{self, ScopedProcess, TempDir};

    use super::*;

    fn make_temp_dir(label: &str) -> TempDir {
        test_support::temp_dir(&format!("stacks-{label}"))
    }

    #[test]
    fn session_directory_prefers_xdg_runtime_when_set() {
        let mut process = ScopedProcess::new();
        let runtime = make_temp_dir("xdg");
        process.set("XDG_RUNTIME_DIR", runtime.path());

        let dir = session_directory();
        assert_eq!(dir, runtime.path().join("dx-sessions"));
    }

    #[test]
    fn session_directory_falls_back_to_temp_dir() {
        let mut process = ScopedProcess::new();
        process.remove("XDG_RUNTIME_DIR");

        let dir = session_directory();
        assert_eq!(dir, env::temp_dir().join("dx-sessions"));
    }

    #[test]
    fn read_missing_file_returns_empty_session() {
        let dir = make_temp_dir("read-missing");
        let stack = read_session(dir.path(), "123").expect("read session");
        assert_eq!(stack, SessionStack::default());
    }

    #[test]
    fn read_corrupt_file_returns_error_without_overwriting() {
        let dir = make_temp_dir("read-corrupt");
        let file = dir.path().join("123.json");
        fs::write(&file, "{invalid json").expect("write corrupt file");

        let error = read_session(dir.path(), "123").expect_err("corrupt session fails");
        assert!(matches!(error, StorageError::ParseSession { .. }));
        assert_eq!(
            fs::read_to_string(&file).expect("read corrupt file"),
            "{invalid json"
        );
    }

    #[test]
    fn write_then_read_round_trip_succeeds() {
        let dir = make_temp_dir("write-read");
        let mut stack = SessionStack::default();
        stack.push(dir.path().join("a")).expect("push cwd");
        stack.push(dir.path().join("b")).expect("push cwd");

        write_session(dir.path(), "200", &stack).expect("write session");
        let loaded = read_session(dir.path(), "200").expect("read session");

        assert_eq!(loaded, stack);
    }

    #[test]
    fn cleanup_is_rate_limited_by_the_marker_file() {
        let dir = make_temp_dir("cleanup-rate-limit");

        assert!(
            cleanup_is_due(dir.path()),
            "first check with no marker should sweep"
        );
        assert!(
            dir.path().join(CLEANUP_MARKER).exists(),
            "the marker is written before sweeping so concurrent shells do not pile up"
        );
        assert!(
            !cleanup_is_due(dir.path()),
            "a fresh marker should suppress the next sweep"
        );
    }

    #[test]
    fn cleanup_check_never_creates_the_session_directory() {
        let temp = make_temp_dir("cleanup-no-mkdir");
        let missing = temp.path().join("absent");

        assert!(cleanup_is_due(&missing));
        assert!(
            !missing.exists(),
            "reading a session must not create the session directory"
        );
    }

    #[test]
    fn cleanup_marker_is_not_treated_as_a_session_file() {
        let dir = make_temp_dir("cleanup-marker-skipped");
        let marker = dir.path().join(CLEANUP_MARKER);
        fs::write(&marker, b"").expect("write marker");

        thread::sleep(Duration::from_millis(5));
        cleanup_stale(dir.path(), Duration::from_secs(0));

        assert!(marker.exists(), "the sweep must not delete its own marker");
    }

    #[test]
    fn cleanup_removes_files_older_than_ttl() {
        let dir = make_temp_dir("cleanup-old");
        let stale = dir.path().join("old_1.json");
        fs::write(&stale, "{}").expect("write stale file");

        thread::sleep(Duration::from_millis(5));
        cleanup_stale(dir.path(), Duration::from_secs(0));

        assert!(!stale.exists());
    }

    #[test]
    fn cleanup_preserves_recent_files() {
        let dir = make_temp_dir("cleanup-recent");
        let recent = dir.path().join("recent_1.json");
        fs::write(&recent, "{}").expect("write recent file");

        cleanup_stale(dir.path(), Duration::from_secs(60 * 60));

        assert!(recent.exists());
    }

    #[test]
    fn cleanup_skips_non_session_files() {
        let dir = make_temp_dir("cleanup-pattern");
        let non_session_json = dir.path().join("bad$.json");
        let lock_file = dir.path().join("active.lock");
        let temp_file = dir.path().join("session.tmp");

        fs::write(&non_session_json, "{}").expect("write bad json");
        fs::write(&lock_file, "lock").expect("write lock");
        fs::write(&temp_file, "tmp").expect("write tmp");

        thread::sleep(Duration::from_millis(5));
        cleanup_stale(dir.path(), Duration::from_secs(0));

        assert!(non_session_json.exists());
        assert!(lock_file.exists());
        assert!(temp_file.exists());
    }

    #[test]
    fn cleanup_permission_errors_do_not_propagate() {
        let dir = make_temp_dir("cleanup-perm");
        let file = dir.path().join("not-a-dir");
        fs::write(&file, "x").expect("write file");

        cleanup_stale(&file, Duration::from_secs(0));

        assert!(file.exists());
    }

    #[test]
    fn write_session_replace_failure_preserves_last_known_good_target() {
        let dir = make_temp_dir("replace-failure");
        let session_id = "123";
        let file = dir.path().join(format!("{session_id}.json"));
        let original = "{\"cwd\":\"/persisted\",\"undo\":[],\"redo\":[]}";
        fs::write(&file, original).expect("seed existing session file");

        let mut next = SessionStack::default();
        next.push(dir.path().join("next")).expect("push next cwd");

        let err = crate::common::with_replace_failure_injection_for_tests(|| {
            write_session(dir.path(), session_id, &next)
        })
        .expect_err("replace failure should surface");
        assert!(matches!(err, StorageError::ReplaceSession { .. }));

        let raw = fs::read_to_string(&file).expect("read persisted target after failure");
        assert_eq!(raw, original);
    }
}
