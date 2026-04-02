use std::path::PathBuf;

use clap::Args;

use crate::menu::{self, parse_buffer, tui::QueryFn, MenuAction, MenuResult};
use crate::resolve::Resolver;

#[derive(Debug, Args)]
pub struct MenuCommand {
    /// Full command-line buffer text
    #[arg(long)]
    pub buffer: String,

    /// Cursor byte position within the buffer
    #[arg(long)]
    pub cursor: usize,

    /// Working directory (defaults to current directory)
    #[arg(long)]
    pub cwd: Option<PathBuf>,

    /// Session identifier (defaults to DX_SESSION env var)
    #[arg(long)]
    pub session: Option<String>,
}

pub fn run_menu(resolver: &Resolver, cmd: MenuCommand) -> i32 {
    let debug = std::env::var("DX_MENU_DEBUG").is_ok_and(|v| v == "1");
    let session = super::complete::resolve_session(cmd.session.as_deref());

    if debug {
        eprintln!(
            "[dx-menu-debug] buffer={:?} cursor={} cwd={:?} session={:?}",
            cmd.buffer, cmd.cursor, cmd.cwd, session
        );
    }

    let parsed = match parse_buffer(&cmd.buffer, cmd.cursor) {
        Some(parsed) => parsed,
        None => {
            if debug {
                eprintln!("[dx-menu-debug] parse_buffer returned None -> noop");
            }
            println!("{}", MenuAction::noop().to_json());
            return 0;
        }
    };

    if debug {
        eprintln!(
            "[dx-menu-debug] mode={:?} query={:?} replace=[{},{}) needs_space_prefix={}",
            parsed.mode,
            parsed.query,
            parsed.replace_start,
            parsed.replace_end,
            parsed.needs_space_prefix
        );
    }

    let initial_candidates = menu::source_candidates(
        resolver,
        parsed.mode.clone(),
        parsed.query.as_deref(),
        session.as_deref(),
    );

    if debug {
        eprintln!("[dx-menu-debug] candidates={}", initial_candidates.len());
    }

    if initial_candidates.is_empty() {
        if debug {
            eprintln!("[dx-menu-debug] no candidates -> noop");
        }
        println!("{}", MenuAction::noop().to_json());
        return 0;
    }

    let initial_query = parsed.query.clone().unwrap_or_default();

    // Build a re-query callback that calls source_candidates with the updated
    // filter query. This ensures filtering is always consistent with
    // `dx complete <mode>` — path prefixes (~/D, /Users/…), abbreviations,
    // and all resolver logic work correctly instead of doing in-memory string
    // matching against already-expanded paths.
    let query_fn: QueryFn<'_> = Box::new(|q: &str| {
        menu::source_candidates(
            resolver,
            parsed.mode.clone(),
            if q.is_empty() { None } else { Some(q) },
            session.as_deref(),
        )
    });

    let cwd = cmd
        .cwd
        .clone()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| std::path::PathBuf::from("/"));

    match menu::tui::select(initial_candidates, &initial_query, &cwd, query_fn) {
        Some(MenuResult::Selected { value, .. }) => {
            let selected = value.display().to_string();
            let value = if parsed.needs_space_prefix {
                format!(" {selected}")
            } else {
                selected
            };
            let action = MenuAction::replace(parsed.replace_start, parsed.replace_end, value);
            if debug {
                eprintln!(
                    "[dx-menu-debug] action=replace value={:?}",
                    action.to_json()
                );
            }
            println!("{}", action.to_json());
            0
        }
        Some(MenuResult::Cancelled {
            filter_query,
            changed_query,
        }) => {
            if changed_query {
                let value = if parsed.needs_space_prefix {
                    format!(" {filter_query}")
                } else {
                    filter_query
                };
                let action = MenuAction::replace(parsed.replace_start, parsed.replace_end, value);
                if debug {
                    eprintln!(
                        "[dx-menu-debug] cancel with changed query -> action=replace value={:?}",
                        action.to_json()
                    );
                }
                println!("{}", action.to_json());
            } else {
                if debug {
                    eprintln!("[dx-menu-debug] cancel without query change -> noop");
                }
                println!("{}", MenuAction::noop().to_json());
            }
            0
        }
        None => {
            if debug {
                eprintln!("[dx-menu-debug] tui unavailable -> noop");
            }
            println!("{}", MenuAction::noop().to_json());
            0
        }
    }
}
