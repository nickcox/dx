use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(test)]
use std::cell::Cell;

pub fn truncate_with_has_more<T>(mut values: Vec<T>, limit: Option<usize>) -> (Vec<T>, bool) {
    let mut has_more = false;
    if let Some(max) = limit
        && values.len() > max
    {
        values.truncate(max);
        has_more = true;
    }

    (values, has_more)
}

/// Appends `candidate` unless an equal path was already pushed.
pub fn push_unique(
    output: &mut Vec<PathBuf>,
    seen: &mut std::collections::HashSet<PathBuf>,
    candidate: PathBuf,
) {
    if seen.insert(candidate.clone()) {
        output.push(candidate);
    }
}

/// ASCII-lowercases raw path bytes, which may not be UTF-8.
pub fn ascii_lowercase(value: &[u8]) -> Vec<u8> {
    value.iter().map(u8::to_ascii_lowercase).collect()
}

pub fn is_valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'_')
}

pub fn resolve_session(cli_session: Option<&str>) -> Option<String> {
    if let Some(value) = cli_session.filter(|value| !value.trim().is_empty()) {
        return Some(value.to_string());
    }

    if let Ok(value) = env::var("DX_SESSION")
        && !value.trim().is_empty()
    {
        return Some(value);
    }

    None
}

#[derive(Debug)]
pub enum AtomicWriteError {
    Write(io::Error),
    Replace(io::Error),
}

/// Whether [`write_atomic_replace`] fsyncs before renaming.
/// The rename is atomic either way, so this only affects crash during write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    Flush,
    Rename,
}

pub fn write_atomic_replace(
    target: &Path,
    payload: &[u8],
    durability: Durability,
) -> Result<(), AtomicWriteError> {
    let (temp, mut file) = create_temp_file(target).map_err(AtomicWriteError::Write)?;
    let written = file.write_all(payload).and_then(|()| match durability {
        Durability::Flush => file.sync_all(),
        Durability::Rename => Ok(()),
    });
    if let Err(source) = written {
        let _ = fs::remove_file(&temp);
        return Err(AtomicWriteError::Write(source));
    }
    drop(file);

    match replace_file(&temp, target) {
        Ok(()) => Ok(()),
        Err(source) => {
            let _ = fs::remove_file(temp);
            Err(AtomicWriteError::Replace(source))
        }
    }
}

fn create_temp_file(target: &Path) -> io::Result<(PathBuf, fs::File)> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let name = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("dx");

    for _ in 0..32 {
        let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
        let temp = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), nonce));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
        {
            Ok(file) => return Ok((temp, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a unique temporary file",
    ))
}

pub fn map_atomic_write_error<T, FWrite, FReplace>(
    err: AtomicWriteError,
    map_write: FWrite,
    map_replace: FReplace,
) -> T
where
    FWrite: FnOnce(io::Error) -> T,
    FReplace: FnOnce(io::Error) -> T,
{
    match err {
        AtomicWriteError::Write(source) => map_write(source),
        AtomicWriteError::Replace(source) => map_replace(source),
    }
}

fn replace_file(from: &Path, to: &Path) -> io::Result<()> {
    #[cfg(test)]
    {
        if test_replace_seam::should_fail_replace_once() {
            return Err(io::Error::other("injected replace failure"));
        }
    }

    replace_file_platform(from, to)
}

#[cfg(not(windows))]
fn replace_file_platform(from: &Path, to: &Path) -> io::Result<()> {
    fs::rename(from, to)
}

#[cfg(windows)]
fn replace_file_platform(from: &Path, to: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MOVEFILE_REPLACE_EXISTING, MoveFileExW};

    let from = from
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let to = to
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    // SAFETY: both buffers are NUL-terminated UTF-16 paths that remain alive for the call.
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), MOVEFILE_REPLACE_EXISTING) } != 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(test)]
pub(crate) fn with_replace_failure_injection_for_tests<T>(operation: impl FnOnce() -> T) -> T {
    test_replace_seam::with_replace_failure(operation)
}

#[cfg(test)]
mod tests {
    use std::{fs, io};

    use crate::test_support;

    use super::{AtomicWriteError, Durability, map_atomic_write_error, write_atomic_replace};

    #[test]
    fn atomic_write_error_mapping_dispatches_to_correct_closure() {
        let write_mapped = map_atomic_write_error(
            AtomicWriteError::Write(io::Error::other("write")),
            |source| format!("write:{}", source),
            |source| format!("replace:{}", source),
        );
        assert_eq!(write_mapped, "write:write");

        let replace_mapped = map_atomic_write_error(
            AtomicWriteError::Replace(io::Error::other("replace")),
            |source| format!("write:{}", source),
            |source| format!("replace:{}", source),
        );
        assert_eq!(replace_mapped, "replace:replace");
    }

    #[test]
    fn both_durabilities_replace_atomically_and_leave_no_temporary_files() {
        for durability in [Durability::Flush, Durability::Rename] {
            let temp = test_support::temp_dir("atomic-durability");
            let target = temp.path().join("state.json");
            fs::write(&target, "old").expect("seed target");

            write_atomic_replace(&target, b"new", durability).expect("replace target");

            assert_eq!(
                fs::read(&target).expect("read target"),
                b"new",
                "{durability:?}"
            );
            assert_eq!(
                fs::read_dir(temp.path())
                    .expect("read temp directory")
                    .count(),
                1,
                "{durability:?} left a temporary file behind"
            );
        }
    }

    #[test]
    fn atomic_write_replaces_existing_file_without_leaving_temporary_files() {
        let temp = test_support::temp_dir("atomic-replace");
        let target = temp.path().join("state.json");
        fs::write(&target, "old").expect("seed target");

        write_atomic_replace(&target, b"new", Durability::Flush).expect("replace target");

        assert_eq!(fs::read(&target).expect("read target"), b"new");
        assert_eq!(
            fs::read_dir(temp.path())
                .expect("read temp directory")
                .count(),
            1
        );
    }
}

#[cfg(test)]
mod test_replace_seam {
    use std::sync::{Mutex, OnceLock};

    use super::Cell;

    thread_local! {
        static FAIL_REPLACE_ONCE: Cell<bool> = const { Cell::new(false) };
    }

    fn replace_failure_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    pub(super) fn with_replace_failure<T>(operation: impl FnOnce() -> T) -> T {
        let _guard = replace_failure_lock()
            .lock()
            .expect("replace failure lock poisoned");
        FAIL_REPLACE_ONCE.with(|flag| flag.set(true));

        struct ResetOnDrop;
        impl Drop for ResetOnDrop {
            fn drop(&mut self) {
                FAIL_REPLACE_ONCE.with(|flag| flag.set(false));
            }
        }
        let _reset = ResetOnDrop;

        operation()
    }

    pub(super) fn should_fail_replace_once() -> bool {
        FAIL_REPLACE_ONCE.with(|flag| flag.replace(false))
    }
}
