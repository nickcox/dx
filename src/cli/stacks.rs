use std::env;
use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum, ValueHint};

use crate::common;
use crate::complete;
use crate::stacks::{SessionStack, StackError, storage};

use super::CliError;

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
    #[arg(value_hint = ValueHint::DirPath)]
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
    /// Print the destination without changing session history
    #[arg(long)]
    pub preview: bool,
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

pub fn run_stack(args: StackCommandArgs) -> Result<(), CliError> {
    if let Some(command) = args.command {
        if args.list || args.clear {
            return Err(CliError::StackFlagsWithSubcommand);
        }

        return match command {
            StackCommand::Push(cmd) => run_push(&cmd.path, cmd.session.as_deref()),
            StackCommand::Undo(cmd) => {
                run_undo(cmd.session.as_deref(), cmd.target.as_deref(), cmd.preview)
            }
            StackCommand::Redo(cmd) => {
                run_redo(cmd.session.as_deref(), cmd.target.as_deref(), cmd.preview)
            }
        };
    }

    if args.list && args.clear {
        return Err(CliError::StackListAndClear);
    }

    if args.list {
        return run_list(args.direction, args.json, args.session.as_deref());
    }

    if args.clear {
        return run_clear(args.direction, args.session.as_deref());
    }

    Err(CliError::StackNoAction)
}

pub fn run_push(path: &str, cli_session: Option<&str>) -> Result<(), CliError> {
    let session_id = resolve_session_id(cli_session)?;
    let target = resolve_absolute_path(path)?;

    let dir = storage::ensure_session_dir()?;
    let mut stack = storage::read_session(&dir, &session_id)?;
    let output = stack.push(target)?;
    storage::write_session(&dir, &session_id, &stack)?;

    println!("{}", output.display());
    Ok(())
}

pub fn run_undo(
    cli_session: Option<&str>,
    target: Option<&str>,
    preview: bool,
) -> Result<(), CliError> {
    match target {
        Some(t) => run_targeted_stack_op(cli_session, t, |stack| stack.undo(), !preview),
        None => run_stack_operation(cli_session, |stack| stack.undo(), !preview),
    }
}

pub fn run_redo(
    cli_session: Option<&str>,
    target: Option<&str>,
    preview: bool,
) -> Result<(), CliError> {
    match target {
        Some(t) => run_targeted_stack_op(cli_session, t, |stack| stack.redo(), !preview),
        None => run_stack_operation(cli_session, |stack| stack.redo(), !preview),
    }
}

pub fn run_list(
    direction: StackListDirection,
    json: bool,
    cli_session: Option<&str>,
) -> Result<(), CliError> {
    let session_id = resolve_session_id(cli_session)?;
    let dir = storage::ensure_session_dir()?;
    let stack = storage::read_session(&dir, &session_id)?;

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
        let payload = complete::format_json(&paths).map_err(CliError::StackJson)?;
        println!("{payload}");
    } else {
        print!("{}", complete::format_plain(&paths));
    }

    Ok(())
}

pub fn run_clear(direction: StackListDirection, cli_session: Option<&str>) -> Result<(), CliError> {
    let session_id = resolve_session_id(cli_session)?;
    let dir = storage::ensure_session_dir()?;
    let mut stack = storage::read_session(&dir, &session_id)?;

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

    storage::write_session(&dir, &session_id, &stack)?;
    Ok(())
}

fn run_targeted_stack_op(
    cli_session: Option<&str>,
    target: &str,
    step: fn(&mut SessionStack) -> Result<PathBuf, StackError>,
    commit: bool,
) -> Result<(), CliError> {
    let target_path = PathBuf::from(target);
    if !target_path.is_absolute() {
        return Err(CliError::StackTargetNotAbsolute(target.to_string()));
    }

    let session_id = resolve_session_id(cli_session)?;
    let dir = storage::ensure_session_dir()?;
    let mut stack = storage::read_session(&dir, &session_id)?;

    // Running out of history before reaching the target is as much a "not
    // reachable" outcome as stepping off the end of the stack.
    let max_steps = stack.undo.len() + stack.redo.len() + 1;
    let unreachable = || CliError::StackTargetUnreachable(target_path.clone());
    let mut reached = None;

    for _ in 0..max_steps {
        let path = step(&mut stack).map_err(|_| unreachable())?;
        if path == target_path {
            reached = Some(path);
            break;
        }
    }

    let reached = reached.ok_or_else(unreachable)?;

    if commit {
        storage::write_session(&dir, &session_id, &stack)?;
    }

    println!("{}", reached.display());
    Ok(())
}

fn run_stack_operation(
    cli_session: Option<&str>,
    operation: impl FnOnce(&mut SessionStack) -> Result<PathBuf, StackError>,
    commit: bool,
) -> Result<(), CliError> {
    let session_id = resolve_session_id(cli_session)?;
    let dir = storage::ensure_session_dir()?;
    let mut stack = storage::read_session(&dir, &session_id)?;

    let output = operation(&mut stack)?;

    if commit {
        storage::write_session(&dir, &session_id, &stack)?;
    }

    println!("{}", output.display());
    Ok(())
}

fn resolve_session_id(cli_session: Option<&str>) -> Result<String, CliError> {
    common::resolve_session(cli_session).ok_or(CliError::MissingSessionId)
}

fn resolve_absolute_path(raw: &str) -> Result<PathBuf, CliError> {
    let input = PathBuf::from(raw);
    if input.as_os_str().is_empty() {
        return Err(CliError::StackPushEmptyPath);
    }
    if input.is_absolute() {
        return Ok(input);
    }

    let cwd = env::current_dir().map_err(CliError::StackPushCurrentDir)?;
    Ok(cwd.join(input))
}
