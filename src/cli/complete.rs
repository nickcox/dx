//! `dx complete` and `dx navigate`: candidate output for shell hooks, and
//! selecting one of those candidates by index or filter.

use clap::{Subcommand, ValueEnum};

use crate::common;
use crate::complete::{
    self, StackDirection, ancestors, filesystem as filesystem_mode,
    filesystem::FilesystemCompletionKind, paths as paths_mode, recents as recents_mode,
    stack as stack_mode,
};
use crate::frecency::ZoxideProvider;
use crate::resolve::Resolver;

use super::CliError;

#[derive(Debug, Subcommand)]
pub enum CompleteCommand {
    Paths {
        query: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long = "limit", alias = "list")]
        limit: Option<usize>,
    },
    Ancestors {
        query: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long = "limit", alias = "list")]
        limit: Option<usize>,
    },
    Frecents {
        query: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long = "limit", alias = "list")]
        limit: Option<usize>,
    },
    Recents {
        query: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long = "limit", alias = "list")]
        limit: Option<usize>,
    },
    Stack {
        #[arg(long, value_enum)]
        direction: StackDirection,
        query: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long = "limit", alias = "list")]
        limit: Option<usize>,
    },
    Filesystem {
        #[arg(value_enum)]
        kind: FilesystemCompletionKind,
        query: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long = "limit", alias = "list")]
        limit: Option<usize>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum NavigateMode {
    Up,
    Back,
    Forward,
}

pub fn run_complete(resolver: &Resolver, command: CompleteCommand) -> Result<(), CliError> {
    let (mut candidates, json, limit) = match command {
        CompleteCommand::Paths { query, json, limit } => {
            let value = query.unwrap_or_default();
            (paths_mode::complete(resolver, &value), json, limit)
        }
        CompleteCommand::Ancestors { query, json, limit } => {
            (ancestors::complete(query.as_deref()), json, limit)
        }
        CompleteCommand::Frecents { query, json, limit } => {
            let provider = ZoxideProvider::default();
            (
                complete::complete_frecents(&provider, query.as_deref()),
                json,
                limit,
            )
        }
        CompleteCommand::Recents {
            query,
            session,
            json,
            limit,
        } => {
            let session = resolve_session(session.as_deref());
            (
                recents_mode::complete(session.as_deref(), query.as_deref()),
                json,
                limit,
            )
        }
        CompleteCommand::Stack {
            direction,
            query,
            session,
            json,
            limit,
        } => {
            let session = resolve_session(session.as_deref());
            (
                stack_mode::complete(session.as_deref(), direction, query.as_deref()),
                json,
                limit,
            )
        }
        CompleteCommand::Filesystem {
            kind,
            query,
            json,
            limit,
        } => {
            let candidates =
                filesystem_mode::complete(resolver, query.as_deref(), None, limit, kind);
            (candidates.paths, json, None)
        }
    };

    if let Some(limit) = limit {
        candidates.truncate(limit);
    }

    if json {
        let output = complete::format_json(&candidates).map_err(CliError::CompleteJson)?;
        println!("{output}");
    } else {
        // `format_plain` already terminates its last line.
        print!("{}", complete::format_plain(&candidates));
    }

    Ok(())
}

pub fn run_navigate(
    mode: NavigateMode,
    selector: Option<&str>,
    session: Option<&str>,
) -> Result<(), CliError> {
    let session = resolve_session(session);
    let candidates = match mode {
        NavigateMode::Up => ancestors::complete(None),
        NavigateMode::Back => stack_mode::complete(session.as_deref(), StackDirection::Back, None),
        NavigateMode::Forward => {
            stack_mode::complete(session.as_deref(), StackDirection::Forward, None)
        }
    };

    let path = complete::select_candidate(&candidates, selector)?;
    println!("{}", path.display());
    Ok(())
}

pub(super) fn resolve_session(cli_session: Option<&str>) -> Option<String> {
    common::resolve_session(cli_session)
}

#[cfg(test)]
mod tests {
    use crate::complete::SelectorError;
    use crate::stacks::{SessionStack, storage};
    use crate::test_support::{ScopedProcess, temp_dir};
    use std::fs;

    use super::{CliError, NavigateMode, run_navigate};

    #[test]
    fn navigate_back_out_of_range_fails() {
        let mut process = ScopedProcess::new();
        let temp = temp_dir("navigate-out-of-range");
        let runtime = temp.path().join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        process.set("XDG_RUNTIME_DIR", runtime.as_os_str());

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack {
            cwd: Some(temp.path().join("now")),
            undo: vec![temp.path().join("a")],
            redo: Vec::new(),
        };
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let error = run_navigate(NavigateMode::Back, Some("2"), Some("s1"))
            .expect_err("selector beyond the stack must fail");

        assert!(matches!(
            error,
            CliError::Navigate(SelectorError::OutOfRange { index: 2, total: 1 })
        ));
    }
}
