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

/// Format a resolved path for insertion into the shell buffer:
/// - Appends a trailing `/` so the user can Tab-complete into the directory.
/// - Single-quote-wraps the path component if it contains any characters that
///   would be interpreted by the shell (spaces, parens, etc.). The trailing `/`
///   is placed outside the quotes so the shell still sees it as a word boundary.
///
/// Examples:
///   /Users/nick/Downloads      → Downloads/      (relative, simple)
///   /Users/nick/Dropbox (Maestral) → 'Dropbox (Maestral)'/
fn format_selected_path(path: &str) -> String {
    // We only need to quote the path component, not add any prefix here —
    // the caller handles needs_space_prefix.
    if needs_shell_quoting(path) {
        // Single-quote the whole string, escaping any embedded single quotes
        // as '\'', then append the trailing slash outside the quotes.
        let escaped = path.replace('\'', "'\\''");
        format!("'{escaped}'/")
    } else {
        format!("{path}/")
    }
}

/// Returns true if the string contains characters that require shell quoting.
fn needs_shell_quoting(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(
            c,
            ' ' | '\t'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '!'
                | '#'
                | '$'
                | '&'
                | '*'
                | '?'
                | ';'
                | '<'
                | '>'
                | '|'
                | '\\'
                | '\''
                | '"'
                | '`'
                | '~'
        )
    })
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

    let cwd = cmd
        .cwd
        .clone()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| std::path::PathBuf::from("/"));

    let initial_candidates = menu::source_candidates(
        resolver,
        parsed.mode.clone(),
        parsed.query.as_deref(),
        session.as_deref(),
        Some(&cwd),
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

    let query_fn: QueryFn<'_> = Box::new(|q: &str| {
        menu::source_candidates(
            resolver,
            parsed.mode.clone(),
            if q.is_empty() { None } else { Some(q) },
            session.as_deref(),
            Some(&cwd),
        )
    });

    match menu::tui::select(initial_candidates, &initial_query, &cwd, query_fn) {
        Some(MenuResult::Selected { value, .. }) => {
            let formatted = format_selected_path(&value.display().to_string());
            let replacement = if parsed.needs_space_prefix {
                format!(" {formatted}")
            } else {
                formatted
            };
            let action = MenuAction::replace(parsed.replace_start, parsed.replace_end, replacement);
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
                // On cancel the user is still typing — no trailing slash or quoting,
                // just preserve what they typed so they can continue refining.
                let replacement = if parsed.needs_space_prefix {
                    format!(" {filter_query}")
                } else {
                    filter_query
                };
                let action =
                    MenuAction::replace(parsed.replace_start, parsed.replace_end, replacement);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_path_gets_trailing_slash() {
        assert_eq!(
            format_selected_path("/Users/nick/Downloads"),
            "/Users/nick/Downloads/"
        );
    }

    #[test]
    fn path_with_spaces_is_quoted_with_trailing_slash_outside() {
        assert_eq!(
            format_selected_path("/Users/nick/Dropbox (Maestral)"),
            "'/Users/nick/Dropbox (Maestral)'/"
        );
    }

    #[test]
    fn path_with_embedded_single_quote_is_escaped() {
        assert_eq!(
            format_selected_path("/tmp/it's here"),
            "'/tmp/it'\\''s here'/"
        );
    }

    #[test]
    fn path_without_special_chars_is_not_quoted() {
        assert!(format_selected_path("/usr/local/bin").starts_with("/usr/local/bin/"));
        assert!(!format_selected_path("/usr/local/bin").contains('\''));
    }
}
