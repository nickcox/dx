//! Reads zoxide's database, when present, to rank directories by frequency and
//! recency. Absent zoxide, lookups return nothing rather than failing.
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

/// Whether the generated hooks should install `z` and `cdf`.
///
/// Checked when `dx init` builds the hook rather than when the commands run, so
/// that a shell without zoxide simply does not get them. Wrappers that silently
/// find nothing are worse than a missing command, and defining `z` regardless
/// would shadow an existing one from zsh-z or rupa/z with a version that cannot
/// work.
///
/// `DX_FRECENCY` overrides the detection either way. Detection reads `PATH`
/// directly instead of running `zoxide --version`: `dx init` is evaluated by
/// every new shell, so this must not cost a process spawn.
pub fn frecency_commands_available() -> bool {
    match std::env::var("DX_FRECENCY") {
        Ok(value) if !value.trim().is_empty() => {
            crate::config::parse_bool(&value, zoxide_on_path())
        }
        _ => zoxide_on_path(),
    }
}

fn zoxide_on_path() -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };

    std::env::split_paths(&path).any(|dir| {
        if dir.as_os_str().is_empty() {
            return false;
        }
        executable_exists(&dir.join(ZOXIDE_BINARY))
    })
}

#[cfg(unix)]
fn executable_exists(candidate: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    candidate
        .metadata()
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn executable_exists(candidate: &std::path::Path) -> bool {
    // Windows carries the executable bit in the extension instead.
    [".exe", ".cmd", ".bat", ""].iter().any(|extension| {
        candidate
            .with_extension(extension.trim_start_matches('.'))
            .is_file()
    })
}

const ZOXIDE_BINARY: &str = "zoxide";

pub trait FrecencyProvider {
    fn query(&self, filter: &str) -> Vec<PathBuf>;
    fn is_available(&self) -> bool;
}

#[derive(Debug)]
pub struct ZoxideProvider {
    binary: String,
    available: OnceLock<bool>,
}

impl ZoxideProvider {
    pub fn new() -> Self {
        Self {
            binary: "zoxide".to_string(),
            available: OnceLock::new(),
        }
    }

    #[cfg(test)]
    fn with_binary(binary: &str) -> Self {
        Self {
            binary: binary.to_string(),
            available: OnceLock::new(),
        }
    }

    fn detect_availability(&self) -> bool {
        Command::new(&self.binary)
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
}

impl Default for ZoxideProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FrecencyProvider for ZoxideProvider {
    fn query(&self, filter: &str) -> Vec<PathBuf> {
        if !self.is_available() {
            return Vec::new();
        }

        let mut command = Command::new(&self.binary);
        command.arg("query").arg("--list");

        let trimmed = filter.trim();
        if !trimmed.is_empty() {
            command.arg(trimmed);
        }

        let output = match command.output() {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };

        if !output.status.success() {
            return Vec::new();
        }

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(PathBuf::from)
            .collect()
    }

    fn is_available(&self) -> bool {
        *self.available.get_or_init(|| self.detect_availability())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{FrecencyProvider, ZoxideProvider};

    #[derive(Debug)]
    struct MockProvider {
        available: bool,
        values: Vec<PathBuf>,
    }

    impl FrecencyProvider for MockProvider {
        fn query(&self, _filter: &str) -> Vec<PathBuf> {
            self.values.clone()
        }

        fn is_available(&self) -> bool {
            self.available
        }
    }

    #[test]
    fn unavailable_zoxide_provider_reports_false() {
        let provider = ZoxideProvider::with_binary("dx-zoxide-missing-for-test");
        assert!(!provider.is_available());
    }

    #[test]
    fn unavailable_zoxide_provider_returns_empty_query_results() {
        let provider = ZoxideProvider::with_binary("dx-zoxide-missing-for-test");
        assert!(provider.query("proj").is_empty());
    }

    #[test]
    fn frecency_provider_trait_contract_can_be_mocked() {
        let provider = MockProvider {
            available: true,
            values: vec![PathBuf::from("/a"), PathBuf::from("/b")],
        };

        assert!(provider.is_available());
        assert_eq!(
            provider.query("anything"),
            vec![PathBuf::from("/a"), PathBuf::from("/b")]
        );
    }
}

#[cfg(test)]
mod availability_tests {
    use std::fs;

    use crate::test_support::{self, ScopedProcess};

    use super::frecency_commands_available;

    fn fake_zoxide(dir: &std::path::Path) {
        let path = dir.join("zoxide");
        fs::write(&path, b"#!/bin/sh\n").expect("write fake zoxide");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
                .expect("make it executable");
        }
    }

    #[test]
    fn detects_zoxide_on_path() {
        let temp = test_support::temp_dir("frecency-on-path");
        let mut process = ScopedProcess::new();
        process.remove("DX_FRECENCY");

        process.set("PATH", temp.path());
        assert!(!frecency_commands_available());

        fake_zoxide(temp.path());
        assert!(frecency_commands_available());
    }

    /// A file that is not executable is not the zoxide anyone is running.
    #[cfg(unix)]
    #[test]
    fn a_non_executable_file_does_not_count() {
        use std::os::unix::fs::PermissionsExt;

        let temp = test_support::temp_dir("frecency-not-executable");
        let mut process = ScopedProcess::new();
        process.remove("DX_FRECENCY");
        process.set("PATH", temp.path());

        let path = temp.path().join("zoxide");
        fs::write(&path, b"not a program").expect("write file");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("clear exec bits");

        assert!(!frecency_commands_available());
    }

    /// The override wins either way, so detection is never a dead end.
    #[test]
    fn dx_frecency_overrides_detection_in_both_directions() {
        let temp = test_support::temp_dir("frecency-override");
        let mut process = ScopedProcess::new();
        process.set("PATH", temp.path());

        process.set("DX_FRECENCY", "1");
        assert!(frecency_commands_available(), "forced on without zoxide");

        fake_zoxide(temp.path());
        process.set("DX_FRECENCY", "0");
        assert!(!frecency_commands_available(), "forced off with zoxide");

        // An unreadable value falls through to detection rather than meaning off.
        process.set("DX_FRECENCY", "banana");
        assert!(frecency_commands_available());
    }
}
