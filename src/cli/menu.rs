use std::path::PathBuf;

use clap::Args;

use crate::menu::{self, parse_buffer, MenuAction};
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

    // Parse the buffer to extract mode, query, and replacement range.
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

    // Source candidates for the resolved mode.
    let candidates = menu::source_candidates(
        resolver,
        parsed.mode,
        parsed.query.as_deref(),
        session.as_deref(),
    );

    if debug {
        eprintln!("[dx-menu-debug] candidates={}", candidates.len());
    }

    if candidates.is_empty() {
        if debug {
            eprintln!("[dx-menu-debug] no candidates -> noop");
        }
        println!("{}", MenuAction::noop().to_json());
        return 0;
    }

    // Present interactive TUI on /dev/tty.  Falls back to noop when the
    // TTY is unavailable (non-interactive context) or the user cancels.
    match menu::tui::select(&candidates) {
        Some(idx) => {
            let selected = candidates[idx].display().to_string();
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
        None => {
            if debug {
                eprintln!("[dx-menu-debug] tui returned None -> noop (cancel or no TTY)");
            }
            println!("{}", MenuAction::noop().to_json());
            0
        }
    }
}
