use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

pub use tempfile::TempDir;

/// Returns the single global lock that must be used by all tests mutating
/// process environment variables so env-dependent tests stay serialized across
/// modules.
pub fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    match LOCK.get_or_init(|| Mutex::new(())).lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub fn temp_dir(label: &str) -> TempDir {
    tempfile::Builder::new()
        .prefix(&format!("dx-{label}-"))
        .tempdir()
        .expect("create temporary directory")
}

pub struct ScopedProcess {
    _guard: MutexGuard<'static, ()>,
    originals: HashMap<OsString, Option<OsString>>,
    original_cwd: Option<PathBuf>,
}

impl ScopedProcess {
    pub fn new() -> Self {
        Self {
            _guard: env_lock(),
            originals: HashMap::new(),
            original_cwd: None,
        }
    }

    pub fn set(&mut self, name: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
        let name = name.as_ref().to_os_string();
        self.remember(&name);
        // SAFETY: ScopedEnv holds the single lock used by all env-mutating unit tests.
        unsafe { std::env::set_var(name, value) };
    }

    pub fn remove(&mut self, name: impl AsRef<OsStr>) {
        let name = name.as_ref().to_os_string();
        self.remember(&name);
        // SAFETY: ScopedEnv holds the single lock used by all env-mutating unit tests.
        unsafe { std::env::remove_var(name) };
    }

    pub fn set_current_dir(&mut self, path: impl AsRef<Path>) {
        if self.original_cwd.is_none() {
            self.original_cwd = Some(std::env::current_dir().expect("read current directory"));
        }
        std::env::set_current_dir(path).expect("set current directory");
    }

    fn remember(&mut self, name: &OsStr) {
        self.originals
            .entry(name.to_os_string())
            .or_insert_with(|| std::env::var_os(name));
    }
}

impl Default for ScopedProcess {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ScopedProcess {
    fn drop(&mut self) {
        if let Some(cwd) = self.original_cwd.take() {
            let _ = std::env::set_current_dir(cwd);
        }
        for (name, value) in self.originals.drain() {
            // SAFETY: the environment lock remains held until after restoration.
            unsafe {
                if let Some(value) = value {
                    std::env::set_var(name, value);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ScopedProcess, temp_dir};

    #[test]
    fn scoped_process_restores_environment_after_repeated_changes() {
        const NAME: &str = "DX_TEST_SCOPED_PROCESS_RESTORE";
        let original = std::env::var_os(NAME);

        {
            let mut process = ScopedProcess::new();
            process.set(NAME, "first");
            process.set(NAME, "second");
            process.remove(NAME);
            assert_eq!(std::env::var_os(NAME), None);
        }

        assert_eq!(std::env::var_os(NAME), original);
    }

    #[test]
    fn scoped_process_restores_current_directory() {
        let original = std::env::current_dir().expect("read current directory");
        let temp = temp_dir("scoped-cwd");

        {
            let mut process = ScopedProcess::new();
            process.set_current_dir(temp.path());
            assert_eq!(
                std::fs::canonicalize(std::env::current_dir().expect("read changed cwd"))
                    .expect("canonical changed cwd"),
                std::fs::canonicalize(temp.path()).expect("canonical temp cwd")
            );
        }

        assert_eq!(
            std::env::current_dir().expect("read restored cwd"),
            original
        );
    }
}
