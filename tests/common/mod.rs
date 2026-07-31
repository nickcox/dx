#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

pub use tempfile::TempDir;

pub fn temp_dir(label: &str) -> TempDir {
    tempfile::Builder::new()
        .prefix(&format!("dx-it-{label}-"))
        .tempdir()
        .expect("create temporary directory")
}

pub fn dx() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dx"))
}

pub fn assert_same_path(actual: impl AsRef<Path>, expected: impl AsRef<Path>) {
    assert_eq!(canonical(actual.as_ref()), canonical(expected.as_ref()));
}

pub fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path)
        .unwrap_or_else(|source| panic!("canonicalize {}: {source}", path.display()))
}

/// The binary under test. `dx()` builds a `Command`; this is for callers that
/// need the path itself.
pub fn dx_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_dx"))
}

/// Writes a session file the way `dx stack` would, for tests that need a stack
/// in place before invoking the binary.
pub fn write_session(runtime: &Path, session: &str, state: &dx::stacks::SessionStack) {
    let dir = runtime.join("dx-sessions");
    std::fs::create_dir_all(&dir).expect("create session dir");
    std::fs::write(
        dir.join(format!("{session}.json")),
        serde_json::to_vec(state).expect("serialize session"),
    )
    .expect("write session");
}

pub fn optional_tool_available(command: &str) -> bool {
    if tool_available(command) {
        return true;
    }

    let diagnostic =
        format!("{command} is required for this external-shell test but is unavailable");
    if std::env::var_os("CI").is_some() || std::env::var_os("DX_REQUIRE_EXTERNAL_TOOLS").is_some() {
        panic!("{diagnostic}");
    }

    eprintln!("skipping external-shell test: {diagnostic}");
    false
}

pub fn tool_available(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

#[cfg(unix)]
pub struct PermissionGuard {
    path: PathBuf,
    original: std::fs::Permissions,
}

#[cfg(unix)]
impl PermissionGuard {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let original = std::fs::metadata(&path)
            .unwrap_or_else(|source| panic!("read permissions for {}: {source}", path.display()))
            .permissions();
        Self { path, original }
    }
}

#[cfg(unix)]
impl Drop for PermissionGuard {
    fn drop(&mut self) {
        let _ = std::fs::set_permissions(&self.path, self.original.clone());
    }
}
