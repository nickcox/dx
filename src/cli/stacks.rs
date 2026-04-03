use std::env;
use std::path::PathBuf;

use crate::stacks::{
    storage::{self, StorageError},
    SessionStack, StackError,
};

pub fn run_push(path: &str, cli_session: Option<&str>) -> i32 {
    let session_id = match resolve_session_id(cli_session) {
        Ok(value) => value,
        Err(code) => return code,
    };
    let target = match resolve_absolute_path(path) {
        Ok(value) => value,
        Err(code) => return code,
    };

    let dir = match storage::ensure_session_dir() {
        Ok(value) => value,
        Err(err) => return storage_error(err),
    };

    let mut stack = match storage::read_session(&dir, &session_id) {
        Ok(value) => value,
        Err(err) => return storage_error(err),
    };

    let output = match stack.push(target) {
        Ok(value) => value,
        Err(err) => return stack_error(err),
    };

    if let Err(err) = storage::write_session(&dir, &session_id, &stack) {
        return storage_error(err);
    }

    println!("{}", output.display());
    0
}

pub fn run_undo(cli_session: Option<&str>, target: Option<&str>) -> i32 {
    match target {
        Some(t) => run_targeted_stack_op(cli_session, t, |stack| stack.undo()),
        None => run_stack_operation(cli_session, |stack| stack.undo()),
    }
}

pub fn run_redo(cli_session: Option<&str>, target: Option<&str>) -> i32 {
    match target {
        Some(t) => run_targeted_stack_op(cli_session, t, |stack| stack.redo()),
        None => run_stack_operation(cli_session, |stack| stack.redo()),
    }
}

fn run_targeted_stack_op(
    cli_session: Option<&str>,
    target: &str,
    step: fn(&mut SessionStack) -> Result<PathBuf, StackError>,
) -> i32 {
    let target_path = PathBuf::from(target);
    if !target_path.is_absolute() {
        eprintln!("dx stacks: target must be an absolute path: {target}");
        return 1;
    }

    let session_id = match resolve_session_id(cli_session) {
        Ok(value) => value,
        Err(code) => return code,
    };

    let dir = match storage::ensure_session_dir() {
        Ok(value) => value,
        Err(err) => return storage_error(err),
    };

    let mut stack = match storage::read_session(&dir, &session_id) {
        Ok(value) => value,
        Err(err) => return storage_error(err),
    };

    // Loop step() until cwd matches target. Cap iterations to prevent infinite loops.
    let max_steps = stack.undo.len() + stack.redo.len() + 1;
    let mut result = PathBuf::new();
    let mut found = false;

    for _ in 0..max_steps {
        match step(&mut stack) {
            Ok(path) => {
                result = path.clone();
                if path == target_path {
                    found = true;
                    break;
                }
            }
            Err(err) => {
                eprintln!("dx stacks: target not reachable: {}", target_path.display());
                let _ = err;
                return 1;
            }
        }
    }

    if !found {
        eprintln!("dx stacks: target not reachable: {}", target_path.display());
        return 1;
    }

    if let Err(err) = storage::write_session(&dir, &session_id, &stack) {
        return storage_error(err);
    }

    println!("{}", result.display());
    0
}

fn run_stack_operation(
    cli_session: Option<&str>,
    operation: impl FnOnce(&mut SessionStack) -> Result<PathBuf, StackError>,
) -> i32 {
    let session_id = match resolve_session_id(cli_session) {
        Ok(value) => value,
        Err(code) => return code,
    };

    let dir = match storage::ensure_session_dir() {
        Ok(value) => value,
        Err(err) => return storage_error(err),
    };

    let mut stack = match storage::read_session(&dir, &session_id) {
        Ok(value) => value,
        Err(err) => return storage_error(err),
    };

    let output = match operation(&mut stack) {
        Ok(value) => value,
        Err(err) => return stack_error(err),
    };

    if let Err(err) = storage::write_session(&dir, &session_id, &stack) {
        return storage_error(err);
    }

    println!("{}", output.display());
    0
}

fn resolve_session_id(cli_session: Option<&str>) -> Result<String, i32> {
    if let Some(value) = cli_session.filter(|value| !value.trim().is_empty()) {
        return Ok(value.to_string());
    }

    if let Ok(value) = env::var("DX_SESSION") {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }

    eprintln!("dx stacks: missing session id (use --session or DX_SESSION)");
    Err(1)
}

fn resolve_absolute_path(raw: &str) -> Result<PathBuf, i32> {
    let input = PathBuf::from(raw);
    if input.as_os_str().is_empty() {
        eprintln!("dx push: path was empty");
        return Err(1);
    }
    if input.is_absolute() {
        return Ok(input);
    }

    match env::current_dir() {
        Ok(cwd) => Ok(cwd.join(input)),
        Err(err) => {
            eprintln!("dx push: failed to read current directory: {err}");
            Err(1)
        }
    }
}

fn storage_error(err: StorageError) -> i32 {
    eprintln!("dx stacks: {err}");
    1
}

fn stack_error(err: StackError) -> i32 {
    eprintln!("dx stacks: {err}");
    1
}
