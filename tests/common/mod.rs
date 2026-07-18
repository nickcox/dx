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
