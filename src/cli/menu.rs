use std::path::PathBuf;

use clap::Args;

use crate::complete::CompletionMode;
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

/// Format a resolved path for insertion into the shell buffer.
///
/// For `Paths` mode (directory browsing):
/// - Appends a trailing `/` so the user can Tab-complete into the directory.
/// - Single-quote-wraps if the path contains shell-special characters.
///   The trailing `/` is placed outside quotes so the shell sees it as a
///   word boundary.
///
/// For all other modes (stack, ancestors, frecents, recents):
/// - Returns the absolute path as-is — no trailing slash, no quoting needed
///   since these paths are always well-formed absolutes navigating to a
///   known destination.
///
/// Examples (Paths mode):
///   /Users/nick/Downloads          → Downloads/
///   /Users/nick/Dropbox (Maestral) → 'Dropbox (Maestral)'/
fn format_selected_path(path: &str, mode: &CompletionMode) -> String {
    match mode {
        CompletionMode::Paths => {
            if needs_shell_quoting(path) {
                let escaped = path.replace('\'', "'\\''");
                format!("'{escaped}'/")
            } else {
                format!("{path}/")
            }
        }
        // Stack, ancestors, frecents, recents — absolute paths, no slash needed.
        _ => path.to_string(),
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
            let formatted = format_selected_path(&value.display().to_string(), &parsed.mode);
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
    use crate::complete::StackDirection;

    use super::*;

    #[test]
    fn paths_mode_simple_path_gets_trailing_slash() {
        assert_eq!(
            format_selected_path("/Users/nick/Downloads", &CompletionMode::Paths),
            "/Users/nick/Downloads/"
        );
    }

    #[test]
    fn paths_mode_path_with_spaces_is_quoted_with_trailing_slash_outside() {
        assert_eq!(
            format_selected_path("/Users/nick/Dropbox (Maestral)", &CompletionMode::Paths),
            "'/Users/nick/Dropbox (Maestral)'/"
        );
    }

    #[test]
    fn paths_mode_path_with_embedded_single_quote_is_escaped() {
        assert_eq!(
            format_selected_path("/tmp/it's here", &CompletionMode::Paths),
            "'/tmp/it'\\''s here'/"
        );
    }

    #[test]
    fn paths_mode_path_without_special_chars_is_not_quoted() {
        let result = format_selected_path("/usr/local/bin", &CompletionMode::Paths);
        assert!(result.starts_with("/usr/local/bin/"));
        assert!(!result.contains('\''));
    }

    #[test]
    fn stack_mode_returns_raw_path_no_slash() {
        let result = format_selected_path(
            "/Users/nick/code",
            &CompletionMode::Stack(StackDirection::Back),
        );
        assert_eq!(result, "/Users/nick/code");
    }

    #[test]
    fn stack_mode_path_with_spaces_not_quoted() {
        let result = format_selected_path(
            "/Users/nick/My Project",
            &CompletionMode::Stack(StackDirection::Back),
        );
        assert_eq!(result, "/Users/nick/My Project");
    }

    #[test]
    fn ancestors_mode_returns_raw_path_no_slash() {
        let result = format_selected_path("/Users/nick", &CompletionMode::Ancestors);
        assert_eq!(result, "/Users/nick");
    }

    #[test]
    fn frecents_mode_returns_raw_path_no_slash() {
        let result = format_selected_path("/Users/nick/projects", &CompletionMode::Frecents);
        assert_eq!(result, "/Users/nick/projects");
    }

    #[test]
    fn recents_mode_returns_raw_path_no_slash() {
        let result = format_selected_path("/tmp/work", &CompletionMode::Recents);
        assert_eq!(result, "/tmp/work");
    }
}
