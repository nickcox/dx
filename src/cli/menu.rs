use std::path::{Path, PathBuf};

use clap::{Args, ValueEnum};

use crate::complete::CompletionMode;
use crate::menu::{
    self, MenuAction, MenuMode, MenuResult, parse_buffer_with_override_mode, tui::QueryFn,
};
use crate::resolve::Resolver;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum MenuModeArg {
    Path,
    Directory,
    File,
}

impl MenuModeArg {
    fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Directory => "directory",
            Self::File => "file",
        }
    }
}

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

    /// Prompt row override for shells that can provide buffer cursor row
    #[arg(long)]
    pub prompt_row: Option<u16>,

    /// Internal compatibility mode for PowerShell PSReadLine menu integration
    #[arg(long, hide = true)]
    pub psreadline_mode: bool,

    /// Explicit mapped-command menu mode for init-generated external command hooks
    #[arg(long, value_enum)]
    pub mode: Option<MenuModeArg>,
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
fn format_selected_path(path: &str, mode: MenuMode) -> String {
    let formatted = if needs_shell_quoting(path) {
        let escaped = path.replace('\'', "'\\''");
        format!("'{escaped}'")
    } else {
        path.to_string()
    };

    match mode {
        MenuMode::Completion(CompletionMode::Paths) | MenuMode::Directory => {
            format!("{formatted}/")
        }
        // Stack, ancestors, frecents, recents — no trailing slash needed.
        _ => formatted,
    }
}

fn sanitize_relative_components(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut cleaned = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => cleaned.push(part),
            Component::ParentDir => cleaned.push(".."),
            Component::RootDir | Component::Prefix(_) => {}
        }
    }
    cleaned
}

fn format_selected_path_for_query_style(
    selected: &Path,
    mode: MenuMode,
    cwd: &Path,
    prefer_relative_paths: bool,
) -> String {
    match mode {
        MenuMode::Completion(CompletionMode::Paths)
        | MenuMode::Path
        | MenuMode::Directory
        | MenuMode::File
            if prefer_relative_paths =>
        {
            let selected_str = selected.display().to_string();
            if let Ok(rel) = selected.strip_prefix(cwd) {
                use std::path::Component;

                let cleaned = sanitize_relative_components(rel);
                let rel_text = if cleaned.as_os_str().is_empty() {
                    "./".to_string()
                } else {
                    let starts_with_parent = cleaned
                        .components()
                        .next()
                        .is_some_and(|component| matches!(component, Component::ParentDir));
                    if starts_with_parent {
                        format!("{}/", cleaned.display())
                    } else {
                        format!("./{}/", cleaned.display())
                    }
                };
                let without_trailing = rel_text.trim_end_matches('/');
                format_selected_path(without_trailing, mode)
            } else {
                format_selected_path(&selected_str, mode)
            }
        }
        _ => format_selected_path(&selected.display().to_string(), mode),
    }
}

fn has_explicit_absolute_input(query: Option<&str>, mode: MenuMode) -> bool {
    mode.prefers_query_relative_rendering() && query.is_some_and(|q| q.starts_with('/'))
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

fn parse_menu_item_max_len() -> Option<usize> {
    let Ok(raw) = std::env::var("DX_MENU_ITEM_MAX_LEN") else {
        // Default: multicolumn on, with no artificial truncation cap.
        return Some(usize::MAX);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(usize::MAX);
    }
    match trimmed.parse::<i64>() {
        Ok(value) if value <= 0 => None,
        Ok(value) => Some(value as usize),
        Err(_) => Some(usize::MAX),
    }
}

fn parse_menu_border() -> bool {
    let Ok(raw) = std::env::var("DX_MENU_BORDER") else {
        return false;
    };
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        _ => false,
    }
}

fn parse_menu_max_rows() -> u16 {
    std::env::var("DX_MENU_MAX_ROWS")
        .ok()
        .and_then(|raw| raw.trim().parse::<u16>().ok())
        .filter(|&value| value > 0)
        .unwrap_or(10)
}

fn parse_menu_max_results() -> usize {
    let default = 1000usize;
    let Ok(raw) = std::env::var("DX_MAX_MENU_RESULTS") else {
        return default;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return default;
    }
    match trimmed.parse::<usize>() {
        Ok(value) if value >= 1 => value,
        _ => default,
    }
}

fn menu_result_to_action(
    result: Option<MenuResult>,
    parsed: &menu::ParsedBuffer,
    mode: MenuMode,
    cwd: &Path,
    prefer_relative_paths: bool,
) -> MenuAction {
    match result {
        Some(MenuResult::Selected { value, .. }) => {
            let formatted =
                format_selected_path_for_query_style(&value, mode, cwd, prefer_relative_paths);
            let replacement = if parsed.needs_space_prefix {
                format!(" {formatted}")
            } else {
                formatted
            };
            MenuAction::replace(parsed.replace_start, parsed.replace_end, replacement)
        }
        Some(MenuResult::Cancelled {
            filter_query: _,
            changed_query: _,
        }) => MenuAction::cancel(),
        None => MenuAction::noop(),
    }
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

    let override_mode = cmd.mode.map(MenuModeArg::as_str);
    let parsed = match parse_buffer_with_override_mode(
        &cmd.buffer,
        cmd.cursor,
        cmd.psreadline_mode,
        override_mode,
    ) {
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

    // For Paths mode, an empty/absent query means "list children of cwd".
    // Substitute "./" so expand_filesystem_prefix enumerates the current directory.
    let is_paths = matches!(parsed.mode, MenuMode::Completion(CompletionMode::Paths));
    let query_is_empty = parsed.query.is_none() || parsed.query.as_deref() == Some("");
    let initial_query_str: &str = if is_paths && query_is_empty {
        "./"
    } else {
        parsed.query.as_deref().unwrap_or("")
    };
    let menu_limit = parse_menu_max_results();

    let initial_candidates = menu::source_candidates_with_meta(
        resolver,
        parsed.mode,
        if initial_query_str.is_empty() {
            None
        } else {
            Some(initial_query_str)
        },
        session.as_deref(),
        Some(&cwd),
        Some(menu_limit),
    );

    if debug {
        eprintln!(
            "[dx-menu-debug] candidates={} has_more={}",
            initial_candidates.paths.len(),
            initial_candidates.has_more
        );
    }

    if initial_candidates.paths.is_empty() {
        if debug {
            eprintln!("[dx-menu-debug] no candidates -> noop");
        }
        println!("{}", MenuAction::noop().to_json());
        return 0;
    }

    let initial_query = parsed.query.clone().unwrap_or_default();
    let prefer_relative_paths = !has_explicit_absolute_input(parsed.query.as_deref(), parsed.mode);

    let query_fn: QueryFn<'_> = Box::new(|q: &str| {
        let resolved_q =
            if q.is_empty() && matches!(parsed.mode, MenuMode::Completion(CompletionMode::Paths)) {
                Some("./")
            } else if q.is_empty() {
                None
            } else {
                Some(q)
            };
        menu::source_candidates_with_meta(
            resolver,
            parsed.mode,
            resolved_q,
            session.as_deref(),
            Some(&cwd),
            Some(menu_limit),
        )
    });

    let item_max_len = parse_menu_item_max_len();
    let show_border = parse_menu_border();
    let max_rows = parse_menu_max_rows();

    let menu_result = menu::tui::select(
        initial_candidates,
        &initial_query,
        &cwd,
        prefer_relative_paths,
        cmd.prompt_row,
        max_rows,
        item_max_len,
        show_border,
        cmd.psreadline_mode,
        query_fn,
    );

    match menu_result_to_action(
        menu_result.clone(),
        &parsed,
        parsed.mode,
        &cwd,
        prefer_relative_paths,
    ) {
        action @ MenuAction::Replace { .. } => {
            if debug {
                match menu_result {
                    Some(MenuResult::Selected { .. }) => {
                        eprintln!(
                            "[dx-menu-debug] action=replace value={:?}",
                            action.to_json()
                        );
                    }
                    Some(MenuResult::Cancelled {
                        changed_query: true,
                        ..
                    }) => {
                        eprintln!(
                            "[dx-menu-debug] explicit cancel after query edits -> action=cancel value={:?}",
                            action.to_json()
                        );
                    }
                    Some(MenuResult::Cancelled {
                        changed_query: false,
                        ..
                    }) => {
                        eprintln!("[dx-menu-debug] explicit cancel -> action=cancel");
                    }
                    _ => {}
                }
            }
            println!("{}", action.to_json());
            0
        }
        MenuAction::Noop => {
            if debug {
                match menu_result {
                    None => {
                        eprintln!("[dx-menu-debug] tui unavailable -> noop");
                    }
                    _ => {}
                }
            }
            println!("{}", MenuAction::noop().to_json());
            0
        }
        MenuAction::Cancel => {
            if debug {
                eprintln!("[dx-menu-debug] action=cancel");
            }
            println!("{}", MenuAction::cancel().to_json());
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::complete::StackDirection;
    use crate::test_support::env_lock;

    use super::*;

    #[test]
    fn paths_mode_simple_path_gets_trailing_slash() {
        assert_eq!(
            format_selected_path(
                "/Users/nick/Downloads",
                MenuMode::Completion(CompletionMode::Paths),
            ),
            "/Users/nick/Downloads/"
        );
    }

    #[test]
    fn menu_result_to_action_returns_noop_for_tui_resize_failure() {
        let parsed = menu::ParsedBuffer {
            mode: MenuMode::Completion(CompletionMode::Paths),
            query: Some("foo".to_string()),
            replace_start: 3,
            replace_end: 6,
            needs_space_prefix: false,
        };

        let action = menu_result_to_action(None, &parsed, parsed.mode, Path::new("/tmp"), true);

        assert_eq!(action, MenuAction::Noop);
    }

    #[test]
    fn menu_result_to_action_maps_cancel_to_explicit_cancel_action() {
        let parsed = menu::ParsedBuffer {
            mode: MenuMode::Completion(CompletionMode::Paths),
            query: Some("D".to_string()),
            replace_start: 3,
            replace_end: 4,
            needs_space_prefix: false,
        };

        let action = menu_result_to_action(
            Some(MenuResult::Cancelled {
                filter_query: "Do".to_string(),
                changed_query: true,
            }),
            &parsed,
            parsed.mode,
            Path::new("/tmp"),
            true,
        );

        assert_eq!(action, MenuAction::Cancel);
    }

    #[test]
    fn paths_mode_path_with_spaces_is_quoted_with_trailing_slash_outside() {
        assert_eq!(
            format_selected_path(
                "/Users/nick/Dropbox (Maestral)",
                MenuMode::Completion(CompletionMode::Paths),
            ),
            "'/Users/nick/Dropbox (Maestral)'/"
        );
    }

    #[test]
    fn paths_mode_path_with_embedded_single_quote_is_escaped() {
        assert_eq!(
            format_selected_path(
                "/tmp/it's here",
                MenuMode::Completion(CompletionMode::Paths),
            ),
            "'/tmp/it'\\''s here'/"
        );
    }

    #[test]
    fn paths_mode_path_without_special_chars_is_not_quoted() {
        let result = format_selected_path(
            "/usr/local/bin",
            MenuMode::Completion(CompletionMode::Paths),
        );
        assert!(result.starts_with("/usr/local/bin/"));
        assert!(!result.contains('\''));
    }

    #[test]
    fn stack_mode_returns_raw_path_no_slash() {
        let result = format_selected_path(
            "/Users/nick/code",
            MenuMode::Completion(CompletionMode::Stack(StackDirection::Back)),
        );
        assert_eq!(result, "/Users/nick/code");
    }

    #[test]
    fn stack_mode_path_with_spaces_is_quoted() {
        let result = format_selected_path(
            "/Users/nick/My Project",
            MenuMode::Completion(CompletionMode::Stack(StackDirection::Back)),
        );
        assert_eq!(result, "'/Users/nick/My Project'");
    }

    #[test]
    fn ancestors_mode_returns_raw_path_no_slash() {
        let result = format_selected_path(
            "/Users/nick",
            MenuMode::Completion(CompletionMode::Ancestors),
        );
        assert_eq!(result, "/Users/nick");
    }

    #[test]
    fn frecents_mode_returns_raw_path_no_slash() {
        let result = format_selected_path(
            "/Users/nick/projects",
            MenuMode::Completion(CompletionMode::Frecents),
        );
        assert_eq!(result, "/Users/nick/projects");
    }

    #[test]
    fn frecents_mode_path_with_spaces_is_quoted_no_slash() {
        let result = format_selected_path(
            "/Users/nick/Dropbox (Maestral)/Obsidian/Notes",
            MenuMode::Completion(CompletionMode::Frecents),
        );
        assert_eq!(result, "'/Users/nick/Dropbox (Maestral)/Obsidian/Notes'");
    }

    #[test]
    fn recents_mode_returns_raw_path_no_slash() {
        let result =
            format_selected_path("/tmp/work", MenuMode::Completion(CompletionMode::Recents));
        assert_eq!(result, "/tmp/work");
    }

    #[test]
    fn paths_mode_relative_cwd_descendant_formats_as_dot_slash() {
        let cwd = Path::new("/tmp/work");
        let selected = Path::new("/tmp/work/./benches");
        let result = format_selected_path_for_query_style(
            selected,
            MenuMode::Completion(CompletionMode::Paths),
            cwd,
            true,
        );
        assert_eq!(result, "./benches/");
    }

    #[test]
    fn paths_mode_parent_relative_prefix_preserved_in_replacement() {
        let cwd = Path::new("/tmp/work");
        let selected = Path::new("/tmp/work/../sibling");
        let result = format_selected_path_for_query_style(
            selected,
            MenuMode::Completion(CompletionMode::Paths),
            cwd,
            true,
        );
        assert_eq!(result, "../sibling/");
    }

    #[test]
    fn paths_mode_multi_parent_relative_prefix_preserved_in_replacement() {
        let cwd = Path::new("/tmp/work");
        let selected = Path::new("/tmp/work/../../outer");
        let result = format_selected_path_for_query_style(
            selected,
            MenuMode::Completion(CompletionMode::Paths),
            cwd,
            true,
        );
        assert_eq!(result, "../../outer/");
    }

    #[test]
    fn paths_mode_explicit_absolute_input_preserves_absolute_output() {
        let cwd = Path::new("/tmp/work");
        let selected = Path::new("/tmp/work/./benches");
        let result = format_selected_path_for_query_style(
            selected,
            MenuMode::Completion(CompletionMode::Paths),
            cwd,
            false,
        );
        assert_eq!(result, "/tmp/work/./benches/");
    }

    #[test]
    fn parse_item_max_len_unset_is_none() {
        let _guard = env_lock();
        unsafe { std::env::remove_var("DX_MENU_ITEM_MAX_LEN") };
        assert_eq!(parse_menu_item_max_len(), Some(usize::MAX));
    }

    #[test]
    fn parse_item_max_len_invalid_is_none() {
        let _guard = env_lock();
        unsafe { std::env::set_var("DX_MENU_ITEM_MAX_LEN", "abc") };
        assert_eq!(parse_menu_item_max_len(), Some(usize::MAX));
        unsafe { std::env::set_var("DX_MENU_ITEM_MAX_LEN", "0") };
        assert_eq!(parse_menu_item_max_len(), None);
        unsafe { std::env::set_var("DX_MENU_ITEM_MAX_LEN", "-3") };
        assert_eq!(parse_menu_item_max_len(), None);
        unsafe { std::env::set_var("DX_MENU_ITEM_MAX_LEN", "") };
        assert_eq!(parse_menu_item_max_len(), Some(usize::MAX));
    }

    #[test]
    fn parse_item_max_len_positive_value() {
        let _guard = env_lock();
        unsafe { std::env::set_var("DX_MENU_ITEM_MAX_LEN", "24") };
        assert_eq!(parse_menu_item_max_len(), Some(24));
    }

    #[test]
    fn parse_menu_border_defaults_off() {
        let _guard = env_lock();
        unsafe { std::env::remove_var("DX_MENU_BORDER") };
        assert!(!parse_menu_border());
        unsafe { std::env::set_var("DX_MENU_BORDER", "") };
        assert!(!parse_menu_border());
    }

    #[test]
    fn parse_menu_border_truthy_values_enable_border() {
        let _guard = env_lock();
        for value in ["1", "true", "TRUE", "yes", "on", " On "] {
            unsafe { std::env::set_var("DX_MENU_BORDER", value) };
            assert!(parse_menu_border(), "expected truthy value: {value}");
        }
    }

    #[test]
    fn parse_menu_border_falsy_values_keep_border_off() {
        let _guard = env_lock();
        for value in ["0", "false", "FALSE", "no", "off", "random"] {
            unsafe { std::env::set_var("DX_MENU_BORDER", value) };
            assert!(!parse_menu_border(), "expected falsy value: {value}");
        }
    }

    #[test]
    fn parse_menu_max_results_defaults_to_1000() {
        let _guard = env_lock();
        unsafe { std::env::remove_var("DX_MAX_MENU_RESULTS") };
        assert_eq!(parse_menu_max_results(), 1000);
    }

    #[test]
    fn parse_menu_max_results_uses_valid_positive_value() {
        let _guard = env_lock();
        unsafe { std::env::set_var("DX_MAX_MENU_RESULTS", "250") };
        assert_eq!(parse_menu_max_results(), 250);
    }

    #[test]
    fn parse_menu_max_results_invalid_falls_back() {
        let _guard = env_lock();
        unsafe { std::env::set_var("DX_MAX_MENU_RESULTS", "0") };
        assert_eq!(parse_menu_max_results(), 1000);
        unsafe { std::env::set_var("DX_MAX_MENU_RESULTS", "abc") };
        assert_eq!(parse_menu_max_results(), 1000);
    }

    #[test]
    fn parse_menu_max_rows_defaults_to_10() {
        let _guard = env_lock();
        unsafe { std::env::remove_var("DX_MENU_MAX_ROWS") };
        assert_eq!(parse_menu_max_rows(), 10);
    }

    #[test]
    fn parse_menu_max_rows_uses_valid_positive_value() {
        let _guard = env_lock();
        unsafe { std::env::set_var("DX_MENU_MAX_ROWS", "24") };
        assert_eq!(parse_menu_max_rows(), 24);
    }

    #[test]
    fn parse_menu_max_rows_invalid_falls_back() {
        let _guard = env_lock();

        unsafe { std::env::set_var("DX_MENU_MAX_ROWS", "") };
        assert_eq!(parse_menu_max_rows(), 10);

        unsafe { std::env::set_var("DX_MENU_MAX_ROWS", "abc") };
        assert_eq!(parse_menu_max_rows(), 10);

        unsafe { std::env::set_var("DX_MENU_MAX_ROWS", "0") };
        assert_eq!(parse_menu_max_rows(), 10);

        unsafe { std::env::set_var("DX_MENU_MAX_ROWS", "-3") };
        assert_eq!(parse_menu_max_rows(), 10);
    }
}
