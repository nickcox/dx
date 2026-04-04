use std::env;
use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

use crate::complete;
use crate::stacks::{
    storage::{self, StorageError},
    SessionStack, StackError,
};

#[derive(Debug, Subcommand)]
pub enum StackCommand {
    Push(StackPushCommand),
    Undo(StackStepCommand),
    Redo(StackStepCommand),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum StackListDirection {
    Undo,
    Redo,
    Both,
}

#[derive(Debug, Args)]
pub struct StackPushCommand {
    pub path: String,
    #[arg(long)]
    pub session: Option<String>,
}

#[derive(Debug, Args)]
pub struct StackStepCommand {
    #[arg(long)]
    pub session: Option<String>,
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Debug, Args)]
pub struct StackCommandArgs {
    #[arg(long)]
    pub list: bool,

    #[arg(long)]
    pub clear: bool,

    #[arg(long, value_enum, default_value_t = StackListDirection::Both)]
    pub direction: StackListDirection,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub session: Option<String>,

    #[command(subcommand)]
    pub command: Option<StackCommand>,
}

pub fn run_stack(args: StackCommandArgs) -> i32 {
    if let Some(command) = args.command {
        if args.list || args.clear {
            eprintln!("dx stack: cannot combine --list/--clear with subcommands");
            return 1;
        }

        return match command {
            StackCommand::Push(cmd) => run_push(&cmd.path, cmd.session.as_deref()),
            StackCommand::Undo(cmd) => run_undo(cmd.session.as_deref(), cmd.target.as_deref()),
            StackCommand::Redo(cmd) => run_redo(cmd.session.as_deref(), cmd.target.as_deref()),
        };
    }

    if args.list && args.clear {
        eprintln!("dx stack: cannot combine --list and --clear");
        return 1;
    }

    if args.list {
        return run_list(args.direction, args.json, args.session.as_deref());
    }

    if args.clear {
        return run_clear(args.direction, args.session.as_deref());
    }

    eprintln!("dx stack: provide one of --list, --clear, or a subcommand");
    1
}

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

pub fn run_list(direction: StackListDirection, json: bool, cli_session: Option<&str>) -> i32 {
    let session_id = match resolve_session_id(cli_session) {
        Ok(value) => value,
        Err(code) => return code,
    };

    let dir = match storage::ensure_session_dir() {
        Ok(value) => value,
        Err(err) => return storage_error(err),
    };

    let stack = match storage::read_session(&dir, &session_id) {
        Ok(value) => value,
        Err(err) => return storage_error(err),
    };

    let mut paths = Vec::new();
    if matches!(
        direction,
        StackListDirection::Undo | StackListDirection::Both
    ) {
        paths.extend(stack.undo.iter().rev().cloned());
    }
    if matches!(
        direction,
        StackListDirection::Redo | StackListDirection::Both
    ) {
        paths.extend(stack.redo.iter().rev().cloned());
    }

    if json {
        match complete::format_json(&paths) {
            Ok(payload) => {
                println!("{payload}");
                0
            }
            Err(err) => {
                eprintln!("dx stack: failed to serialize json: {err}");
                1
            }
        }
    } else {
        let output = complete::format_plain(&paths);
        print!("{output}");
        0
    }
}

pub fn run_clear(direction: StackListDirection, cli_session: Option<&str>) -> i32 {
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

    if matches!(
        direction,
        StackListDirection::Undo | StackListDirection::Both
    ) {
        stack.undo.clear();
    }
    if matches!(
        direction,
        StackListDirection::Redo | StackListDirection::Both
    ) {
        stack.redo.clear();
    }

    if let Err(err) = storage::write_session(&dir, &session_id, &stack) {
        return storage_error(err);
    }

    0
}

fn run_targeted_stack_op(
    cli_session: Option<&str>,
    target: &str,
    step: fn(&mut SessionStack) -> Result<PathBuf, StackError>,
) -> i32 {
    let target_path = PathBuf::from(target);
    if !target_path.is_absolute() {
        eprintln!("dx stack: target must be an absolute path: {target}");
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
            Err(_) => {
                eprintln!("dx stack: target not reachable: {}", target_path.display());
                return 1;
            }
        }
    }

    if !found {
        eprintln!("dx stack: target not reachable: {}", target_path.display());
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

    eprintln!("dx stack: missing session id (use --session or DX_SESSION)");
    Err(1)
}

fn resolve_absolute_path(raw: &str) -> Result<PathBuf, i32> {
    let input = PathBuf::from(raw);
    if input.as_os_str().is_empty() {
        eprintln!("dx stack push: path was empty");
        return Err(1);
    }
    if input.is_absolute() {
        return Ok(input);
    }

    match env::current_dir() {
        Ok(cwd) => Ok(cwd.join(input)),
        Err(err) => {
            eprintln!("dx stack push: failed to read current directory: {err}");
            Err(1)
        }
    }
}

fn storage_error(err: StorageError) -> i32 {
    eprintln!("dx stack: {err}");
    1
}

fn stack_error(err: StackError) -> i32 {
    eprintln!("dx stack: {err}");
    1
}
