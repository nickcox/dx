use std::env;

use clap::{Subcommand, ValueEnum};

use crate::complete::{
    self, ancestors, paths as paths_mode, recents as recents_mode, stack as stack_mode,
    StackDirection,
};
use crate::frecency::ZoxideProvider;
use crate::resolve::Resolver;

#[derive(Debug, Subcommand)]
pub enum CompleteCommand {
    Paths {
        query: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Ancestors {
        query: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Frecents {
        query: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Recents {
        query: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Stack {
        #[arg(long, value_enum)]
        direction: StackDirectionArg,
        query: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum StackDirectionArg {
    Back,
    Forward,
}

impl From<StackDirectionArg> for StackDirection {
    fn from(value: StackDirectionArg) -> Self {
        match value {
            StackDirectionArg::Back => StackDirection::Back,
            StackDirectionArg::Forward => StackDirection::Forward,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum NavigateMode {
    Up,
    Back,
    Forward,
}

pub fn run_complete(resolver: &Resolver, command: CompleteCommand) -> i32 {
    let (candidates, json) = match command {
        CompleteCommand::Paths { query, json } => {
            let value = query.unwrap_or_default();
            (paths_mode::complete(resolver, &value), json)
        }
        CompleteCommand::Ancestors { query, json } => (ancestors::complete(query.as_deref()), json),
        CompleteCommand::Frecents { query, json } => {
            let provider = ZoxideProvider::default();
            (
                complete::complete_frecents(&provider, query.as_deref()),
                json,
            )
        }
        CompleteCommand::Recents {
            query,
            session,
            json,
        } => {
            let session = resolve_session(session.as_deref());
            (
                recents_mode::complete(session.as_deref(), query.as_deref()),
                json,
            )
        }
        CompleteCommand::Stack {
            direction,
            query,
            session,
            json,
        } => {
            let session = resolve_session(session.as_deref());
            (
                stack_mode::complete(session.as_deref(), direction.into(), query.as_deref()),
                json,
            )
        }
    };

    if json {
        match complete::format_json(&candidates) {
            Ok(output) => {
                print!("{output}");
                0
            }
            Err(err) => {
                eprintln!("dx complete: failed to serialize json: {err}");
                1
            }
        }
    } else {
        print!("{}", complete::format_plain(&candidates));
        0
    }
}

pub fn run_navigate(mode: NavigateMode, selector: Option<&str>, session: Option<&str>) -> i32 {
    let session = resolve_session(session);
    let candidates = match mode {
        NavigateMode::Up => ancestors::complete(None),
        NavigateMode::Back => stack_mode::complete(session.as_deref(), StackDirection::Back, None),
        NavigateMode::Forward => {
            stack_mode::complete(session.as_deref(), StackDirection::Forward, None)
        }
    };

    match complete::select_candidate(&candidates, selector) {
        Ok(path) => {
            println!("{}", path.display());
            0
        }
        Err(err) => {
            eprintln!("dx navigate: {err}");
            1
        }
    }
}

pub(super) fn resolve_session(cli_session: Option<&str>) -> Option<String> {
    if let Some(value) = cli_session.filter(|value| !value.trim().is_empty()) {
        return Some(value.to_string());
    }

    if let Ok(value) = env::var("DX_SESSION") {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::stacks::{storage, SessionStack};
    use crate::test_support;

    use super::{run_navigate, NavigateMode};

    fn make_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "dx-navigate-{label}-{nonce}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        test_support::env_lock()
    }

    #[test]
    fn navigate_back_out_of_range_fails() {
        let _guard = env_lock();
        let temp = make_temp_dir("out-of-range");
        let runtime = temp.join("runtime");
        fs::create_dir_all(&runtime).expect("create runtime");
        unsafe { std::env::set_var("XDG_RUNTIME_DIR", runtime.display().to_string()) };

        let dir = storage::ensure_session_dir().expect("session dir");
        let stack = SessionStack {
            cwd: Some(PathBuf::from("/now")),
            undo: vec![PathBuf::from("/a")],
            redo: Vec::new(),
        };
        storage::write_session(&dir, "s1", &stack).expect("write session");

        let code = run_navigate(NavigateMode::Back, Some("2"), Some("s1"));
        assert_eq!(code, 1);

        unsafe { std::env::remove_var("XDG_RUNTIME_DIR") };
        let _ = fs::remove_dir_all(temp);
    }
}
