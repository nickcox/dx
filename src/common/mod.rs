use std::env;
use std::fs;
use std::io;
use std::path::Path;

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

pub fn write_atomic_replace(temp: &Path, target: &Path, payload: &[u8]) -> Result<(), AtomicWriteError> {
    fs::write(temp, payload).map_err(AtomicWriteError::Write)?;

    match replace_file(temp, target) {
        Ok(()) => Ok(()),
        Err(source) => {
            let _ = fs::remove_file(temp);
            Err(AtomicWriteError::Replace(source))
        }
    }
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

    fs::rename(from, to)
}

#[cfg(test)]
pub(crate) fn with_replace_failure_injection_for_tests<T>(operation: impl FnOnce() -> T) -> T {
    test_replace_seam::with_replace_failure(operation)
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::{AtomicWriteError, map_atomic_write_error};

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
        let _guard = replace_failure_lock().lock().expect("replace failure lock poisoned");
        FAIL_REPLACE_ONCE.with(|flag| flag.set(true));

        struct ResetOnDrop;
        impl Drop for ResetOnDrop {
            fn drop(&mut self) {
                FAIL_REPLACE_ONCE.with(|flag| flag.set(false));
            }
        }
        let _reset = ResetOnDrop;

        let result = operation();
        result
    }

    pub(super) fn should_fail_replace_once() -> bool {
        FAIL_REPLACE_ONCE.with(|flag| flag.replace(false))
    }
}
