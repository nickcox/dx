use std::env;
use std::fs;
use std::io;
use std::path::Path;

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

    match fs::rename(temp, target) {
        Ok(()) => Ok(()),
        Err(source) => {
            if target.exists() {
                let _ = fs::remove_file(target);
                if fs::rename(temp, target).is_ok() {
                    return Ok(());
                }
            }

            let _ = fs::remove_file(temp);
            Err(AtomicWriteError::Replace(source))
        }
    }
}
