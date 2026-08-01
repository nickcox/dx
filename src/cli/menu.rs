//! `dx menu` — parses the shell's buffer, sources candidates, runs the TUI, and
//! prints the JSON action telling the shell how to rewrite its line.

use std::path::{Path, PathBuf};

use clap::{Args, ValueHint};

use crate::complete::CompletionMode;
use crate::complete::filesystem::FilesystemCompletionKind;
use crate::config::MenuSettings;
use crate::menu::{
    self, MenuAction, MenuMode, MenuOptions, MenuRequest, MenuResult, QueryStyle,
    parse_buffer_with_override_mode, tui::QueryFn,
};
use crate::resolve::Resolver;
use crate::shell::{self, Shell};

use super::CliError;

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

    /// Explicit mapped-command menu mode for init-generated external command hooks
    #[arg(long, value_enum)]
    pub mode: Option<FilesystemCompletionKind>,

    /// Shell syntax used for replacement text
    #[arg(long, value_enum, default_value_t = Shell::Bash)]
    pub shell: Shell,
}

impl MenuCommand {
    /// Whether the caller is PowerShell's PSReadLine menu, the only PowerShell
    /// integration that invokes `dx menu`; `--native-menu` uses PSReadLine's own
    /// completion. Derived from `--shell` rather than carried as a hidden flag,
    /// because `clap_complete` ignores `Arg::hide` and would offer it as a
    /// completion.
    fn psreadline_mode(&self) -> bool {
        self.shell == Shell::Pwsh
    }
}

fn insertion_text(path: &Path, append_trailing_separator: bool, shell: Shell) -> Option<String> {
    let path = if append_trailing_separator {
        let mut path = path.to_path_buf();
        path.as_mut_os_string().push(std::path::MAIN_SEPARATOR_STR);
        path
    } else {
        path.to_path_buf()
    };
    shell::quote_path(shell, path.to_str()?)
}

fn insertion_text_for_style(
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
        return insertion_text(selected, append_trailing_slash, shell);
    }

    match style {
        QueryStyle::Compact | QueryStyle::Absolute => {
            insertion_text(selected, append_trailing_slash, shell)
        }
        QueryStyle::HomeRelative => dirs::home_dir()
            .and_then(|home| {
                format_home_relative_path(selected, &home, append_trailing_slash, shell)
            })
            .or_else(|| insertion_text(selected, append_trailing_slash, shell)),
        QueryStyle::BareRelative | QueryStyle::DotRelative | QueryStyle::ParentRelative => {
            let relative_path = if style == QueryStyle::ParentRelative {
                crate::complete::parent_relative_path_from(cwd, selected)
            } else {
                crate::complete::relative_path_from(cwd, selected)
            };
            let Some(relative) = relative_path else {
                return insertion_text(selected, append_trailing_slash, shell);
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
                // The remaining styles are excluded by the outer arm; falling
                // back to the absolute form keeps that unverifiable claim from
                // being a panic.
                QueryStyle::ParentRelative
                | QueryStyle::Compact
                | QueryStyle::HomeRelative
                | QueryStyle::Absolute => {
                    return insertion_text(selected, append_trailing_slash, shell);
                }
            };
            insertion_text(&relative, append_trailing_slash, shell)
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
        shell::quote_path(shell, &suffix)?
    ))
}

/// `LS_COLORS` is the shell's variable, not a dx setting, so only the opt-in
/// flag lives in config; an empty or absent `LS_COLORS` still yields no styling.
fn menu_ls_colors(enabled: bool) -> Option<crate::menu::ls_colors::LsColorsConfig> {
    if !enabled {
        return None;
    }
    let raw = std::env::var("LS_COLORS").ok()?;
    if raw.trim().is_empty() {
        return None;
    }
    Some(crate::menu::ls_colors::parse_ls_colors(&raw))
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
            let Some(formatted) = insertion_text_for_style(&value, mode, cwd, style, shell) else {
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
        shell::offset_from_byte(shell, buffer, replace_start),
        shell::offset_from_byte(shell, buffer, replace_end),
    ) else {
        return MenuAction::noop();
    };
    MenuAction::replace(replace_start, replace_end, value, terminal, geometry)
}

/// The query to source candidates for, or `None` for "no query".
///
/// An empty query in `Paths` mode means "list the children of the cwd", which the
/// filesystem expander spells `./`. Every other mode treats empty as no query.
fn query_for_mode(mode: MenuMode, query: &str) -> Option<&str> {
    if !query.is_empty() {
        return Some(query);
    }
    matches!(mode, MenuMode::Completion(CompletionMode::Paths)).then_some("./")
}

/// `DX_MENU_DEBUG` tracing. Carrying the flag in a value keeps each trace point to
/// one line, and the closure means nothing is formatted while tracing is off.
#[derive(Copy, Clone)]
struct Trace(bool);

impl Trace {
    fn from_env() -> Self {
        Self(
            std::env::var("DX_MENU_DEBUG")
                .is_ok_and(|value| crate::config::parse_bool(&value, false)),
        )
    }

    fn say<D: std::fmt::Display>(self, message: impl FnOnce() -> D) {
        if self.0 {
            eprintln!("[dx-menu-debug] {}", message());
        }
    }
}

/// Which menu outcome produced the action, recorded before the result is consumed
/// so tracing does not need a copy of it.
#[derive(Copy, Clone)]
enum Outcome {
    Selected,
    CancelledAfterEdits,
    Cancelled,
    Unavailable,
}

impl Outcome {
    fn of(result: &Option<MenuResult>) -> Self {
        match result {
            Some(MenuResult::Selected { .. }) => Self::Selected,
            Some(MenuResult::Cancelled {
                changed_query: true,
                ..
            }) => Self::CancelledAfterEdits,
            Some(MenuResult::Cancelled {
                changed_query: false,
                ..
            }) => Self::Cancelled,
            None => Self::Unavailable,
        }
    }
}

pub fn run_menu(
    settings: &MenuSettings,
    resolver: &Resolver,
    cmd: MenuCommand,
) -> Result<(), CliError> {
    let trace = Trace::from_env();
    let session = super::complete::resolve_session(cmd.session.as_deref());

    trace.say(|| {
        format!(
            "buffer={:?} cursor={} cwd={:?} session={:?}",
            cmd.buffer, cmd.cursor, cmd.cwd, session
        )
    });

    let parsed = match parse_buffer_with_override_mode(
        &cmd.buffer,
        cmd.cursor,
        cmd.psreadline_mode(),
        cmd.mode,
    ) {
        Some(parsed) => parsed,
        None => {
            trace.say(|| "parse_buffer returned None -> noop");
            println!("{}", MenuAction::noop().to_json());
            return Ok(());
        }
    };

    trace.say(|| {
        format!(
            "mode={:?} query={:?} replace=[{},{}) needs_space_prefix={}",
            parsed.mode,
            parsed.query,
            parsed.replace_start,
            parsed.replace_end,
            parsed.needs_space_prefix
        )
    });

    let cwd = cmd
        .cwd
        .clone()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| std::path::PathBuf::from("/"));

    let menu_limit = settings.max_results;

    let initial_candidates = menu::source_candidates_with_meta(
        resolver,
        parsed.mode,
        query_for_mode(parsed.mode, parsed.query.as_deref().unwrap_or("")),
        session.as_deref(),
        Some(&cwd),
        Some(menu_limit),
    );

    trace.say(|| {
        format!(
            "candidates={} has_more={}",
            initial_candidates.paths.len(),
            initial_candidates.has_more
        )
    });

    if initial_candidates.paths.is_empty() {
        trace.say(|| "no candidates -> noop");
        println!("{}", MenuAction::noop().to_json());
        return Ok(());
    }

    let initial_query = parsed.query.clone().unwrap_or_default();
    let query_style = QueryStyle::from_query(parsed.mode, parsed.query.as_deref().unwrap_or(""));

    let query_fn: QueryFn<'_> = Box::new(|q: &str| {
        menu::source_candidates_with_meta(
            resolver,
            parsed.mode,
            query_for_mode(parsed.mode, q),
            session.as_deref(),
            Some(&cwd),
            Some(menu_limit),
        )
    });

    let options = MenuOptions {
        max_rows: settings.max_rows,
        item_max_len: settings.item_max_len,
        show_border: settings.border,
        use_tty_backend: cmd.psreadline_mode(),
        ls_colors: menu_ls_colors(settings.ls_colors),
    };

    let menu_result = menu::tui::select(
        MenuRequest {
            candidates: initial_candidates,
            query: &initial_query,
            mode: parsed.mode,
            cwd: &cwd,
            prompt_row: cmd.prompt_row,
            query_fn,
        },
        &options,
    );

    let outcome = Outcome::of(&menu_result);
    let action = menu_result_to_action_with_shell(
        menu_result,
        &parsed,
        parsed.mode,
        &cwd,
        query_style,
        cmd.shell,
    );
    match action_for_shell(action, &cmd.buffer, cmd.shell) {
        action @ MenuAction::Replace { .. } => {
            match outcome {
                Outcome::Selected => {
                    trace.say(|| format!("action=replace value={:?}", action.to_json()));
                }
                Outcome::CancelledAfterEdits => trace.say(|| {
                    format!(
                        "explicit cancel after query edits -> action=cancel value={:?}",
                        action.to_json()
                    )
                }),
                Outcome::Cancelled => trace.say(|| "explicit cancel -> action=cancel"),
                Outcome::Unavailable => {}
            }
            println!("{}", action.to_json());
        }
        MenuAction::Noop => {
            if matches!(outcome, Outcome::Unavailable) {
                trace.say(|| "tui unavailable -> noop");
            }
            println!("{}", MenuAction::noop().to_json());
        }
        action @ MenuAction::Cancel { .. } => {
            trace.say(|| "action=cancel");
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
    /// Pins the default shell so the assertions below read as formatting cases.
    /// `Paths` mode gets a trailing `/` and shell quoting; other modes insert the
    /// absolute path unchanged.
    fn format_selected_path(path: &str, mode: MenuMode) -> String {
        let append_trailing_slash = matches!(
            mode,
            MenuMode::Completion(CompletionMode::Paths) | MenuMode::Directory
        );
        insertion_text(Path::new(path), append_trailing_slash, Shell::Bash)
            .expect("test path has no control characters")
    }

    fn format_selected_path_for_query_style(
        selected: &Path,
        mode: MenuMode,
        cwd: &Path,
        prefer_relative_paths: bool,
    ) -> String {
        insertion_text_for_style(
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

    #[test]
    fn ls_colors_needs_both_the_flag_and_a_populated_ls_colors() {
        let mut process = ScopedProcess::new();

        process.set("LS_COLORS", "di=01;34");
        assert!(menu_ls_colors(true).is_some(), "flag on, LS_COLORS set");
        assert!(menu_ls_colors(false).is_none(), "flag off");

        process.set("LS_COLORS", "   ");
        assert!(menu_ls_colors(true).is_none(), "LS_COLORS blank");

        process.remove("LS_COLORS");
        assert!(menu_ls_colors(true).is_none(), "LS_COLORS unset");
    }

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
            insertion_text(Path::new(path), false, Shell::Bash),
            Some("'/tmp/it'\\''s here'".to_string())
        );
        assert_eq!(
            insertion_text(Path::new(path), false, Shell::Zsh),
            Some("'/tmp/it'\\''s here'".to_string())
        );
        assert_eq!(
            insertion_text(Path::new(path), false, Shell::Fish),
            Some("/tmp/it\\'s\\ here".to_string())
        );
        assert_eq!(
            insertion_text(Path::new(path), false, Shell::Pwsh),
            Some("'/tmp/it''s here'".to_string())
        );
    }

    #[cfg(windows)]
    #[test]
    fn powershell_directory_replacements_preserve_drive_and_unc_prefixes() {
        assert_eq!(
            insertion_text(Path::new(r"C:\Project Files"), true, Shell::Pwsh,),
            Some(r"'C:\Project Files\'".to_string())
        );
        assert_eq!(
            insertion_text(Path::new(r"\\server\share\folder"), true, Shell::Pwsh,),
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
        let result = insertion_text_for_style(
            cwd,
            MenuMode::Completion(CompletionMode::Paths),
            cwd,
            QueryStyle::ParentRelative,
            Shell::Bash,
        );

        assert_eq!(result, Some("../work/".to_string()));
    }

    #[cfg(unix)]
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

    #[cfg(unix)]
    #[test]
    fn bare_relative_replacement_has_no_implicit_dot_prefix() {
        let result = insertion_text_for_style(
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
    /// The rule the initial query and the re-query closure used to spell out
    /// separately: empty means "list the cwd" in `Paths` mode, and "no query"
    /// everywhere else.
    #[test]
    fn empty_query_means_the_cwd_only_in_paths_mode() {
        let paths = MenuMode::Completion(CompletionMode::Paths);
        assert_eq!(query_for_mode(paths, ""), Some("./"));
        assert_eq!(query_for_mode(paths, "src"), Some("src"));

        for mode in [
            MenuMode::Completion(CompletionMode::Ancestors),
            MenuMode::Completion(CompletionMode::Recents),
            MenuMode::Completion(CompletionMode::Frecents),
            MenuMode::Path,
            MenuMode::Directory,
            MenuMode::File,
        ] {
            assert_eq!(
                query_for_mode(mode, ""),
                None,
                "{mode:?} should have no query"
            );
            assert_eq!(query_for_mode(mode, "src"), Some("src"));
        }
    }
}
