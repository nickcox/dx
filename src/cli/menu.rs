use std::path::{Path, PathBuf};

use clap::{Args, ValueHint};

use crate::complete::CompletionMode;
use crate::complete::filesystem::FilesystemCompletionKind;
use crate::hooks::Shell;
use crate::menu::{
    self, MenuAction, MenuMode, MenuResult, QueryStyle, parse_buffer_with_override_mode,
    tui::QueryFn,
};
use crate::resolve::Resolver;

use super::CliError;

/// Quotes `path` using `shell`'s syntax, or `None` when the path holds control
/// characters that no quoting can make safe to inject into a buffer.
fn quote_for_shell(shell: Shell, path: &str) -> Option<String> {
    if path.chars().any(char::is_control) {
        return None;
    }

    Some(match shell {
        Shell::Bash | Shell::Zsh => quote_posix(path),
        Shell::Fish => quote_fish(path),
        Shell::Pwsh => quote_pwsh(path),
    })
}

/// Converts a UTF-8 byte offset into the buffer unit the shell's line editor
/// counts in: bytes for Bash, characters for Zsh and Fish, UTF-16 code units
/// for PowerShell.
fn offset_from_byte(shell: Shell, buffer: &str, byte_offset: usize) -> Option<usize> {
    if byte_offset > buffer.len() || !buffer.is_char_boundary(byte_offset) {
        return None;
    }

    let prefix = &buffer[..byte_offset];
    Some(match shell {
        Shell::Bash => byte_offset,
        Shell::Zsh | Shell::Fish => prefix.chars().count(),
        Shell::Pwsh => prefix.encode_utf16().count(),
    })
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
    #[arg(value_hint = ValueHint::DirPath)]
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
    pub mode: Option<FilesystemCompletionKind>,

    /// Shell syntax used for replacement text
    #[arg(long, value_enum, default_value_t = Shell::Bash)]
    pub shell: Shell,
}

/// Format a resolved path for insertion into the shell buffer.
///
/// For `Paths` mode (directory browsing):
/// - Appends a trailing `/` so the user can Tab-complete into the directory.
/// - Single-quote-wraps if the path contains shell-special characters.
///   The trailing `/` is included inside quotes when quoting is needed.
///
/// For all other modes (stack, ancestors, frecents, recents):
/// - Returns the absolute path as-is — no trailing slash, no quoting needed
///   since these paths are always well-formed absolutes navigating to a
///   known destination.
///
/// Examples (Paths mode):
///   /Users/nick/Downloads          → Downloads/
///   /Users/nick/Dropbox (Maestral) → 'Dropbox (Maestral)/'
#[cfg(test)]
fn format_selected_path(path: &str, mode: MenuMode) -> String {
    let append_trailing_slash = matches!(
        mode,
        MenuMode::Completion(CompletionMode::Paths) | MenuMode::Directory
    );
    format_selected_path_with_trailing_separator(
        Path::new(path),
        append_trailing_slash,
        Shell::Bash,
    )
    .expect("test path has no control characters")
}

fn format_selected_path_with_trailing_separator(
    path: &Path,
    append_trailing_separator: bool,
    shell: Shell,
) -> Option<String> {
    let path = if append_trailing_separator {
        let mut path = path.to_path_buf();
        path.as_mut_os_string().push(std::path::MAIN_SEPARATOR_STR);
        path
    } else {
        path.to_path_buf()
    };
    quote_for_shell(shell, path.to_str()?)
}

fn format_selected_path_for_query_style_checked(
    selected: &Path,
    mode: MenuMode,
    cwd: &Path,
    style: QueryStyle,
    shell: Shell,
) -> Option<String> {
    let append_trailing_slash = matches!(
        mode,
        MenuMode::Completion(CompletionMode::Paths) | MenuMode::Directory
    ) || (mode == MenuMode::Path && selected.is_dir());

    if !mode.prefers_query_relative_rendering() {
        return format_selected_path_with_trailing_separator(
            selected,
            append_trailing_slash,
            shell,
        );
    }

    match style {
        QueryStyle::Compact | QueryStyle::Absolute => {
            format_selected_path_with_trailing_separator(selected, append_trailing_slash, shell)
        }
        QueryStyle::HomeRelative => dirs::home_dir()
            .and_then(|home| {
                format_home_relative_path(selected, &home, append_trailing_slash, shell)
            })
            .or_else(|| {
                format_selected_path_with_trailing_separator(selected, append_trailing_slash, shell)
            }),
        QueryStyle::BareRelative | QueryStyle::DotRelative | QueryStyle::ParentRelative => {
            let relative_path = if style == QueryStyle::ParentRelative {
                crate::complete::parent_relative_path_from(cwd, selected)
            } else {
                crate::complete::relative_path_from(cwd, selected)
            };
            let Some(relative) = relative_path else {
                return format_selected_path_with_trailing_separator(
                    selected,
                    append_trailing_slash,
                    shell,
                );
            };

            let relative = match style {
                QueryStyle::BareRelative => relative,
                QueryStyle::DotRelative if relative == Path::new(".") => {
                    PathBuf::from(format!(".{}", std::path::MAIN_SEPARATOR))
                }
                QueryStyle::DotRelative => {
                    PathBuf::from(format!(".{}", std::path::MAIN_SEPARATOR)).join(relative)
                }
                QueryStyle::ParentRelative if relative.starts_with("..") => relative,
                // A parent-rooted query must never silently become cwd-relative.
                QueryStyle::ParentRelative => {
                    return format_selected_path_with_trailing_separator(
                        selected,
                        append_trailing_slash,
                        shell,
                    );
                }
                _ => unreachable!(),
            };
            format_selected_path_with_trailing_separator(&relative, append_trailing_slash, shell)
        }
    }
}

fn format_home_relative_path(
    selected: &Path,
    home: &Path,
    append_trailing_separator: bool,
    shell: Shell,
) -> Option<String> {
    let relative = selected.strip_prefix(home).ok()?;
    if relative.as_os_str().is_empty() {
        return Some(if append_trailing_separator {
            format!("~{}", std::path::MAIN_SEPARATOR)
        } else {
            "~".to_string()
        });
    }

    let mut suffix = relative.to_str()?.to_string();
    if append_trailing_separator {
        suffix.push(std::path::MAIN_SEPARATOR);
    }
    // Keep `~/` outside quotes: quoting a tilde prevents POSIX shells from expanding it.
    Some(format!(
        "~{}{}",
        std::path::MAIN_SEPARATOR,
        quote_for_shell(shell, &suffix)?
    ))
}

#[cfg(test)]
fn format_selected_path_for_query_style(
    selected: &Path,
    mode: MenuMode,
    cwd: &Path,
    prefer_relative_paths: bool,
) -> String {
    format_selected_path_for_query_style_checked(
        selected,
        mode,
        cwd,
        if prefer_relative_paths {
            if crate::complete::relative_path_from(cwd, selected)
                .is_some_and(|relative| relative.starts_with(".."))
            {
                QueryStyle::ParentRelative
            } else {
                QueryStyle::DotRelative
            }
        } else {
            QueryStyle::Absolute
        },
        Shell::Bash,
    )
    .expect("test path has no control characters")
}

/// Returns true if the string contains characters that require shell quoting.
fn needs_shell_quoting(s: &str) -> bool {
    s.chars().any(|c| {
        c.is_whitespace()
            || matches!(
                c,
                '(' | ')'
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

fn quote_posix(path: &str) -> String {
    if needs_shell_quoting(path) {
        format!("'{}'", path.replace('\'', "'\\''"))
    } else {
        path.to_string()
    }
}

fn quote_fish(path: &str) -> String {
    if !needs_shell_quoting(path) {
        return path.to_string();
    }

    path.chars().fold(String::new(), |mut escaped, ch| {
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-')) {
            escaped.push('\\');
        }
        escaped.push(ch);
        escaped
    })
}

fn quote_pwsh(path: &str) -> String {
    if needs_shell_quoting(path) {
        format!("'{}'", path.replace('\'', "''"))
    } else {
        path.to_string()
    }
}

fn parse_menu_item_max_len() -> Option<usize> {
    let default = 80usize;
    let Ok(raw) = std::env::var("DX_MENU_ITEM_MAX_LEN") else {
        return Some(default);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(default);
    }
    match trimmed.parse::<i64>() {
        Ok(value) if value <= 0 => None,
        Ok(value) => Some(value as usize),
        Err(_) => Some(default),
    }
}

fn parse_menu_border() -> bool {
    let Ok(raw) = std::env::var("DX_MENU_BORDER") else {
        return false;
    };
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn parse_menu_ls_colors() -> Option<crate::menu::ls_colors::LsColorsConfig> {
    let ls_colors_raw = std::env::var("LS_COLORS").ok()?;
    if ls_colors_raw.trim().is_empty() {
        return None;
    }
    let enabled = match std::env::var("DX_MENU_LS_COLORS") {
        Ok(val) => val.trim() == "1",
        Err(_) => return None,
    };
    if !enabled {
        return None;
    }
    Some(crate::menu::ls_colors::parse_ls_colors(&ls_colors_raw))
}

fn parse_menu_max_rows() -> u16 {
    std::env::var("DX_MENU_MAX_ROWS")
        .ok()
        .and_then(|raw| raw.trim().parse::<u16>().ok())
        .filter(|&value| value > 0)
        .unwrap_or(20)
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

fn menu_result_to_action_with_shell(
    result: Option<MenuResult>,
    parsed: &menu::ParsedBuffer,
    mode: MenuMode,
    cwd: &Path,
    style: QueryStyle,
    shell: Shell,
) -> MenuAction {
    match result {
        Some(MenuResult::Selected {
            value,
            terminal,
            geometry,
            ..
        }) => {
            let Some(formatted) =
                format_selected_path_for_query_style_checked(&value, mode, cwd, style, shell)
            else {
                return MenuAction::noop();
            };
            let replacement = if parsed.needs_space_prefix {
                format!(" {formatted}")
            } else {
                formatted
            };
            MenuAction::replace(
                parsed.replace_start,
                parsed.replace_end,
                replacement,
                terminal,
                geometry,
            )
        }
        Some(MenuResult::Cancelled {
            filter_query: _,
            changed_query: _,
            geometry,
        }) => MenuAction::cancel(geometry),
        None => MenuAction::noop(),
    }
}

fn action_for_shell(action: MenuAction, buffer: &str, shell: Shell) -> MenuAction {
    let MenuAction::Replace {
        replace_start,
        replace_end,
        value,
        terminal,
        geometry,
    } = action
    else {
        return action;
    };

    let (Some(replace_start), Some(replace_end)) = (
        offset_from_byte(shell, buffer, replace_start),
        offset_from_byte(shell, buffer, replace_end),
    ) else {
        return MenuAction::noop();
    };
    MenuAction::replace(replace_start, replace_end, value, terminal, geometry)
}

#[cfg(test)]
fn menu_result_to_action(
    result: Option<MenuResult>,
    parsed: &menu::ParsedBuffer,
    mode: MenuMode,
    cwd: &Path,
    prefer_relative_paths: bool,
) -> MenuAction {
    menu_result_to_action_with_shell(
        result,
        parsed,
        mode,
        cwd,
        if prefer_relative_paths {
            QueryStyle::DotRelative
        } else {
            QueryStyle::Absolute
        },
        Shell::Bash,
    )
}

pub fn run_menu(resolver: &Resolver, cmd: MenuCommand) -> Result<(), CliError> {
    let debug = std::env::var("DX_MENU_DEBUG").is_ok_and(|v| v == "1");
    let session = super::complete::resolve_session(cmd.session.as_deref());

    if debug {
        eprintln!(
            "[dx-menu-debug] buffer={:?} cursor={} cwd={:?} session={:?}",
            cmd.buffer, cmd.cursor, cmd.cwd, session
        );
    }

    let parsed = match parse_buffer_with_override_mode(
        &cmd.buffer,
        cmd.cursor,
        cmd.psreadline_mode,
        cmd.mode,
    ) {
        Some(parsed) => parsed,
        None => {
            if debug {
                eprintln!("[dx-menu-debug] parse_buffer returned None -> noop");
            }
            println!("{}", MenuAction::noop().to_json());
            return Ok(());
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
        return Ok(());
    }

    let initial_query = parsed.query.clone().unwrap_or_default();
    let query_style = QueryStyle::from_query(parsed.mode, parsed.query.as_deref().unwrap_or(""));

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
    let ls_colors = parse_menu_ls_colors();

    let menu_result = menu::tui::select(
        initial_candidates,
        &initial_query,
        parsed.mode,
        &cwd,
        cmd.prompt_row,
        max_rows,
        item_max_len,
        show_border,
        cmd.psreadline_mode,
        query_fn,
        ls_colors,
    );

    let action = menu_result_to_action_with_shell(
        menu_result.clone(),
        &parsed,
        parsed.mode,
        &cwd,
        query_style,
        cmd.shell,
    );
    match action_for_shell(action, &cmd.buffer, cmd.shell) {
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
        }
        MenuAction::Noop => {
            if debug && menu_result.is_none() {
                eprintln!("[dx-menu-debug] tui unavailable -> noop");
            }
            println!("{}", MenuAction::noop().to_json());
        }
        action @ MenuAction::Cancel { .. } => {
            if debug {
                eprintln!("[dx-menu-debug] action=cancel");
            }
            println!("{}", action.to_json());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::complete::StackDirection;
    use crate::menu::action::{TerminalGeometry, TerminalState};
    use crate::test_support::{ScopedProcess, temp_dir};

    use super::*;

    #[cfg(unix)]
    #[test]
    fn menu_result_to_action_passes_terminal_state_through() {
        let parsed = menu::ParsedBuffer {
            mode: MenuMode::Completion(CompletionMode::Paths),
            query: Some("foo".to_string()),
            replace_start: 3,
            replace_end: 6,
            needs_space_prefix: false,
        };

        let clean_action = menu_result_to_action(
            Some(MenuResult::Selected {
                value: PathBuf::from("/tmp/bar"),
                filter_query: "fo".to_string(),
                changed_query: true,
                terminal: TerminalState::Clean,
                geometry: None,
            }),
            &parsed,
            parsed.mode,
            Path::new("/tmp"),
            true,
        );
        assert_eq!(
            clean_action,
            MenuAction::Replace {
                replace_start: 3,
                replace_end: 6,
                value: "./bar/".to_string(),
                terminal: TerminalState::Clean,
                geometry: None,
            }
        );

        let dirty_action = menu_result_to_action(
            Some(MenuResult::Selected {
                value: PathBuf::from("/tmp/baz"),
                filter_query: "fo".to_string(),
                changed_query: true,
                terminal: TerminalState::Dirty,
                geometry: Some(TerminalGeometry {
                    redraw_row: 13,
                    scroll_rows: 10,
                }),
            }),
            &parsed,
            parsed.mode,
            Path::new("/tmp"),
            true,
        );
        let MenuAction::Replace {
            terminal, geometry, ..
        } = dirty_action
        else {
            panic!("expected Replace");
        };
        assert_eq!(terminal, TerminalState::Dirty);
        assert_eq!(
            geometry,
            Some(TerminalGeometry {
                redraw_row: 13,
                scroll_rows: 10,
            })
        );
    }

    #[cfg(unix)]
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
                geometry: Some(TerminalGeometry {
                    redraw_row: 8,
                    scroll_rows: 4,
                }),
            }),
            &parsed,
            parsed.mode,
            Path::new("/tmp"),
            true,
        );

        assert_eq!(
            action,
            MenuAction::Cancel {
                terminal: Some(TerminalState::Dirty),
                geometry: Some(TerminalGeometry {
                    redraw_row: 8,
                    scroll_rows: 4,
                }),
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn paths_mode_path_with_spaces_is_quoted_with_trailing_slash_inside() {
        assert_eq!(
            format_selected_path(
                "/Users/nick/Dropbox (Maestral)",
                MenuMode::Completion(CompletionMode::Paths),
            ),
            "'/Users/nick/Dropbox (Maestral)/'"
        );
    }

    #[cfg(unix)]
    #[test]
    fn paths_mode_path_with_embedded_single_quote_is_escaped() {
        assert_eq!(
            format_selected_path(
                "/tmp/it's here",
                MenuMode::Completion(CompletionMode::Paths),
            ),
            "'/tmp/it'\\''s here/'"
        );
    }

    #[cfg(unix)]
    #[test]
    fn selected_paths_use_shell_specific_quoting() {
        let path = "/tmp/it's here";

        assert_eq!(
            format_selected_path_with_trailing_separator(Path::new(path), false, Shell::Bash),
            Some("'/tmp/it'\\''s here'".to_string())
        );
        assert_eq!(
            format_selected_path_with_trailing_separator(Path::new(path), false, Shell::Zsh),
            Some("'/tmp/it'\\''s here'".to_string())
        );
        assert_eq!(
            format_selected_path_with_trailing_separator(Path::new(path), false, Shell::Fish),
            Some("/tmp/it\\'s\\ here".to_string())
        );
        assert_eq!(
            format_selected_path_with_trailing_separator(Path::new(path), false, Shell::Pwsh),
            Some("'/tmp/it''s here'".to_string())
        );
    }

    #[cfg(windows)]
    #[test]
    fn powershell_directory_replacements_preserve_drive_and_unc_prefixes() {
        assert_eq!(
            format_selected_path_with_trailing_separator(
                Path::new(r"C:\Project Files"),
                true,
                Shell::Pwsh,
            ),
            Some(r"'C:\Project Files\'".to_string())
        );
        assert_eq!(
            format_selected_path_with_trailing_separator(
                Path::new(r"\\server\share\folder"),
                true,
                Shell::Pwsh,
            ),
            Some(r"'\\server\share\folder\'".to_string())
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf_selected_path_returns_noop_at_action_boundary() {
        use std::os::unix::ffi::OsStringExt;

        let parsed = menu::ParsedBuffer {
            mode: MenuMode::Path,
            query: Some("x".to_string()),
            replace_start: 3,
            replace_end: 4,
            needs_space_prefix: false,
        };
        let non_utf_path = PathBuf::from(std::ffi::OsString::from_vec(vec![b'/', 0xff]));
        let action = menu_result_to_action_with_shell(
            Some(MenuResult::Selected {
                value: non_utf_path,
                filter_query: "x".to_string(),
                changed_query: false,
                terminal: TerminalState::Clean,
                geometry: None,
            }),
            &parsed,
            parsed.mode,
            Path::new("/tmp"),
            QueryStyle::Absolute,
            Shell::Bash,
        );

        assert_eq!(action, MenuAction::Noop);
    }

    #[test]
    fn selected_path_with_control_character_returns_noop() {
        let parsed = menu::ParsedBuffer {
            mode: MenuMode::Path,
            query: Some("x".to_string()),
            replace_start: 3,
            replace_end: 4,
            needs_space_prefix: false,
        };
        let action = menu_result_to_action_with_shell(
            Some(MenuResult::Selected {
                value: PathBuf::from("/tmp/line\nbreak"),
                filter_query: "x".to_string(),
                changed_query: false,
                terminal: TerminalState::Clean,
                geometry: None,
            }),
            &parsed,
            parsed.mode,
            Path::new("/tmp"),
            QueryStyle::Absolute,
            Shell::Bash,
        );

        assert_eq!(action, MenuAction::Noop);
    }

    #[test]
    fn shell_actions_convert_utf8_byte_ranges_to_native_units() {
        let action = MenuAction::replace(3, 7, "x".to_string(), TerminalState::Clean, None);
        let buffer = "cd \u{00e9}\u{00e9}";

        let zsh = action_for_shell(action.clone(), buffer, Shell::Zsh);
        let fish = action_for_shell(action.clone(), buffer, Shell::Fish);
        let pwsh = action_for_shell(action.clone(), buffer, Shell::Pwsh);
        let bash = action_for_shell(action, buffer, Shell::Bash);

        for action in [zsh, fish] {
            let MenuAction::Replace {
                replace_start,
                replace_end,
                ..
            } = action
            else {
                panic!("expected replacement");
            };
            assert_eq!((replace_start, replace_end), (3, 5));
        }
        let MenuAction::Replace {
            replace_start,
            replace_end,
            ..
        } = pwsh
        else {
            panic!("expected replacement");
        };
        assert_eq!((replace_start, replace_end), (3, 5));
        let MenuAction::Replace {
            replace_start,
            replace_end,
            ..
        } = bash
        else {
            panic!("expected replacement");
        };
        assert_eq!((replace_start, replace_end), (3, 7));
    }

    #[test]
    fn powershell_actions_use_utf16_offsets_for_astral_characters() {
        let buffer = "cd \u{1f600}";
        let action = action_for_shell(
            MenuAction::replace(3, buffer.len(), "x".to_string(), TerminalState::Clean, None),
            buffer,
            Shell::Pwsh,
        );

        let MenuAction::Replace {
            replace_start,
            replace_end,
            ..
        } = action
        else {
            panic!("expected replacement");
        };
        assert_eq!((replace_start, replace_end), (3, 5));
    }

    #[cfg(unix)]
    #[test]
    fn paths_mode_path_without_special_chars_is_not_quoted() {
        let result = format_selected_path(
            "/usr/local/bin",
            MenuMode::Completion(CompletionMode::Paths),
        );
        assert!(result.starts_with("/usr/local/bin/"));
        assert!(!result.contains('\''));
    }

    #[cfg(unix)]
    #[test]
    fn stack_mode_returns_raw_path_no_slash() {
        let result = format_selected_path(
            "/Users/nick/code",
            MenuMode::Completion(CompletionMode::Stack(StackDirection::Back)),
        );
        assert_eq!(result, "/Users/nick/code");
    }

    #[cfg(unix)]
    #[test]
    fn stack_mode_path_with_spaces_is_quoted() {
        let result = format_selected_path(
            "/Users/nick/My Project",
            MenuMode::Completion(CompletionMode::Stack(StackDirection::Back)),
        );
        assert_eq!(result, "'/Users/nick/My Project'");
    }

    #[cfg(unix)]
    #[test]
    fn ancestors_mode_returns_raw_path_no_slash() {
        let result = format_selected_path(
            "/Users/nick",
            MenuMode::Completion(CompletionMode::Ancestors),
        );
        assert_eq!(result, "/Users/nick");
    }

    #[cfg(unix)]
    #[test]
    fn frecents_mode_returns_raw_path_no_slash() {
        let result = format_selected_path(
            "/Users/nick/projects",
            MenuMode::Completion(CompletionMode::Frecents),
        );
        assert_eq!(result, "/Users/nick/projects");
    }

    #[cfg(unix)]
    #[test]
    fn frecents_mode_path_with_spaces_is_quoted_no_slash() {
        let result = format_selected_path(
            "/Users/nick/Dropbox (Maestral)/Obsidian/Notes",
            MenuMode::Completion(CompletionMode::Frecents),
        );
        assert_eq!(result, "'/Users/nick/Dropbox (Maestral)/Obsidian/Notes'");
    }

    #[cfg(unix)]
    #[test]
    fn recents_mode_returns_raw_path_no_slash() {
        let result =
            format_selected_path("/tmp/work", MenuMode::Completion(CompletionMode::Recents));
        assert_eq!(result, "/tmp/work");
    }

    #[cfg(unix)]
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

    #[cfg(unix)]
    #[test]
    fn menu_result_to_action_preserves_relative_replacement_formatting() {
        let parsed = menu::ParsedBuffer {
            mode: MenuMode::Completion(CompletionMode::Paths),
            query: Some("s".to_string()),
            replace_start: 3,
            replace_end: 4,
            needs_space_prefix: false,
        };
        let selected = PathBuf::from("/tmp/work/src");

        let action = menu_result_to_action(
            Some(MenuResult::Selected {
                filter_query: "s".to_string(),
                changed_query: false,
                value: selected,
                terminal: TerminalState::Dirty,
                geometry: Some(TerminalGeometry {
                    redraw_row: 7,
                    scroll_rows: 3,
                }),
            }),
            &parsed,
            parsed.mode,
            Path::new("/tmp/work"),
            true,
        );

        assert_eq!(
            action,
            MenuAction::Replace {
                replace_start: 3,
                replace_end: 4,
                value: "./src/".to_string(),
                terminal: TerminalState::Dirty,
                geometry: Some(TerminalGeometry {
                    redraw_row: 7,
                    scroll_rows: 3,
                }),
            }
        );
    }

    #[cfg(unix)]
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

    #[cfg(unix)]
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

    #[cfg(unix)]
    #[test]
    fn parent_relative_replacement_keeps_anchor_for_cwd_candidate() {
        let cwd = Path::new("/tmp/work");
        let result = format_selected_path_for_query_style_checked(
            cwd,
            MenuMode::Completion(CompletionMode::Paths),
            cwd,
            QueryStyle::ParentRelative,
            Shell::Bash,
        );

        assert_eq!(result, Some("../work/".to_string()));
    }

    #[test]
    fn home_relative_replacement_keeps_tilde_unquoted() {
        let home = Path::new("/tmp/home");
        assert_eq!(
            format_home_relative_path(
                Path::new("/tmp/home/Project Files"),
                home,
                true,
                Shell::Bash,
            ),
            Some("~/'Project Files/'".to_string())
        );
        assert_eq!(
            format_home_relative_path(home, home, true, Shell::Zsh),
            Some("~/".to_string())
        );
    }

    #[test]
    fn bare_relative_replacement_has_no_implicit_dot_prefix() {
        let result = format_selected_path_for_query_style_checked(
            Path::new("/tmp/work/benches"),
            MenuMode::Completion(CompletionMode::Paths),
            Path::new("/tmp/work"),
            QueryStyle::BareRelative,
            Shell::Bash,
        );

        assert_eq!(result, Some("benches/".to_string()));
    }

    #[cfg(unix)]
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

    #[cfg(unix)]
    #[test]
    fn mapped_path_mode_directory_gets_trailing_slash() {
        let temp = temp_dir("menu-path-mode-dir");
        let selected = temp.path().join("src");
        std::fs::create_dir_all(&selected).expect("selected directory should be created");

        let result =
            format_selected_path_for_query_style(&selected, MenuMode::Path, temp.path(), false);

        assert_eq!(result, selected.display().to_string() + "/");
    }

    #[cfg(unix)]
    #[test]
    fn mapped_path_mode_directory_relative_to_cwd_gets_dot_slash() {
        let temp = temp_dir("menu-path-mode-relative-dir");
        let selected = temp.path().join("src");
        std::fs::create_dir_all(&selected).expect("selected directory should be created");

        let result =
            format_selected_path_for_query_style(&selected, MenuMode::Path, temp.path(), true);

        assert_eq!(result, "./src/");
    }

    #[cfg(unix)]
    #[test]
    fn mapped_path_mode_file_does_not_get_trailing_slash() {
        let temp = temp_dir("menu-path-mode-file");
        let selected = temp.path().join("readme.md");
        std::fs::write(&selected, "test").expect("selected file should be created");

        let result =
            format_selected_path_for_query_style(&selected, MenuMode::Path, temp.path(), false);

        assert_eq!(result, selected.display().to_string());
    }

    #[cfg(unix)]
    #[test]
    fn mapped_path_mode_quoted_directory_keeps_trailing_slash_inside_quotes() {
        let temp = temp_dir("menu-path-mode-quoted-dir");
        let selected = temp.path().join("Project Files");
        std::fs::create_dir_all(&selected).expect("selected directory should be created");

        let result =
            format_selected_path_for_query_style(&selected, MenuMode::Path, temp.path(), false);
        let expected = format!("'{}/'", selected.display());

        assert_eq!(result, expected);
    }

    #[test]
    fn parse_item_max_len_unset_uses_default_cap() {
        let mut process = ScopedProcess::new();
        process.remove("DX_MENU_ITEM_MAX_LEN");
        assert_eq!(parse_menu_item_max_len(), Some(80));
    }

    #[test]
    fn parse_item_max_len_invalid_uses_default_cap() {
        let mut process = ScopedProcess::new();
        process.set("DX_MENU_ITEM_MAX_LEN", "abc");
        assert_eq!(parse_menu_item_max_len(), Some(80));
        process.set("DX_MENU_ITEM_MAX_LEN", "0");
        assert_eq!(parse_menu_item_max_len(), None);
        process.set("DX_MENU_ITEM_MAX_LEN", "-3");
        assert_eq!(parse_menu_item_max_len(), None);
        process.set("DX_MENU_ITEM_MAX_LEN", "");
        assert_eq!(parse_menu_item_max_len(), Some(80));
    }

    #[test]
    fn parse_item_max_len_positive_value() {
        let mut process = ScopedProcess::new();
        process.set("DX_MENU_ITEM_MAX_LEN", "24");
        assert_eq!(parse_menu_item_max_len(), Some(24));
    }

    #[test]
    fn parse_menu_border_defaults_off() {
        let mut process = ScopedProcess::new();
        process.remove("DX_MENU_BORDER");
        assert!(!parse_menu_border());
        process.set("DX_MENU_BORDER", "");
        assert!(!parse_menu_border());
    }

    #[test]
    fn parse_menu_border_truthy_values_enable_border() {
        let mut process = ScopedProcess::new();
        for value in ["1", "true", "TRUE", "yes", "on", " On "] {
            process.set("DX_MENU_BORDER", value);
            assert!(parse_menu_border(), "expected truthy value: {value}");
        }
    }

    #[test]
    fn parse_menu_border_falsy_values_keep_border_off() {
        let mut process = ScopedProcess::new();
        for value in ["0", "false", "FALSE", "no", "off", "random"] {
            process.set("DX_MENU_BORDER", value);
            assert!(!parse_menu_border(), "expected falsy value: {value}");
        }
    }

    #[test]
    fn parse_menu_max_results_defaults_to_1000() {
        let mut process = ScopedProcess::new();
        process.remove("DX_MAX_MENU_RESULTS");
        assert_eq!(parse_menu_max_results(), 1000);
    }

    #[test]
    fn parse_menu_max_results_uses_valid_positive_value() {
        let mut process = ScopedProcess::new();
        process.set("DX_MAX_MENU_RESULTS", "250");
        assert_eq!(parse_menu_max_results(), 250);
    }

    #[test]
    fn parse_menu_max_results_invalid_falls_back() {
        let mut process = ScopedProcess::new();
        process.set("DX_MAX_MENU_RESULTS", "0");
        assert_eq!(parse_menu_max_results(), 1000);
        process.set("DX_MAX_MENU_RESULTS", "abc");
        assert_eq!(parse_menu_max_results(), 1000);
    }

    #[test]
    fn parse_menu_max_rows_defaults_to_20() {
        let mut process = ScopedProcess::new();
        process.remove("DX_MENU_MAX_ROWS");
        assert_eq!(parse_menu_max_rows(), 20);
    }

    #[test]
    fn parse_menu_max_rows_uses_valid_positive_value() {
        let mut process = ScopedProcess::new();
        process.set("DX_MENU_MAX_ROWS", "24");
        assert_eq!(parse_menu_max_rows(), 24);
    }

    #[test]
    fn parse_menu_max_rows_invalid_falls_back() {
        let mut process = ScopedProcess::new();

        process.set("DX_MENU_MAX_ROWS", "");
        assert_eq!(parse_menu_max_rows(), 20);

        process.set("DX_MENU_MAX_ROWS", "abc");
        assert_eq!(parse_menu_max_rows(), 20);

        process.set("DX_MENU_MAX_ROWS", "0");
        assert_eq!(parse_menu_max_rows(), 20);

        process.set("DX_MENU_MAX_ROWS", "-3");
        assert_eq!(parse_menu_max_rows(), 20);
    }

    #[test]
    fn parse_menu_ls_colors_missing_both_env_vars_returns_none() {
        let mut process = ScopedProcess::new();
        process.remove("DX_MENU_LS_COLORS");
        process.remove("LS_COLORS");
        assert_eq!(parse_menu_ls_colors(), None);
    }

    #[test]
    fn parse_menu_ls_colors_missing_flag_returns_none() {
        let mut process = ScopedProcess::new();
        process.remove("DX_MENU_LS_COLORS");
        process.set("LS_COLORS", "di=01;34");
        assert_eq!(parse_menu_ls_colors(), None);
    }

    #[test]
    fn parse_menu_ls_colors_non_one_flag_returns_none() {
        let mut process = ScopedProcess::new();
        process.set("DX_MENU_LS_COLORS", "0");
        process.set("LS_COLORS", "di=01;34");
        assert_eq!(parse_menu_ls_colors(), None);
    }

    #[test]
    fn parse_menu_ls_colors_missing_ls_colors_returns_none() {
        let mut process = ScopedProcess::new();
        process.set("DX_MENU_LS_COLORS", "1");
        process.remove("LS_COLORS");
        assert_eq!(parse_menu_ls_colors(), None);
    }

    #[test]
    fn parse_menu_ls_colors_empty_ls_colors_returns_none() {
        let mut process = ScopedProcess::new();
        process.set("DX_MENU_LS_COLORS", "1");
        process.set("LS_COLORS", "");
        assert_eq!(parse_menu_ls_colors(), None);
    }

    #[test]
    fn parse_menu_ls_colors_both_set_returns_config() {
        let mut process = ScopedProcess::new();
        process.set("DX_MENU_LS_COLORS", "1");
        process.set("LS_COLORS", "di=01;34");
        assert!(parse_menu_ls_colors().is_some());
    }
}
