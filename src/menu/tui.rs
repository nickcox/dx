//! Interactive TUI selection for `dx menu`.
//!
//! Renders an inline list immediately below the prompt line.
//! stdout stays free for JSON output; the TUI is drawn to stderr.
//! crossterm is built with `use-dev-tty` so `event::read()` reads from
//! `/dev/tty` directly, working even when stdin is redirected by a shell
//! completion hook.

#[cfg(unix)]
mod imp {
    use std::io::{stderr, Read, Write};
    use std::path::{Path, PathBuf};

    use crossterm::{
        cursor,
        event::{self, Event, KeyCode, KeyModifiers},
        execute, terminal,
    };
    use ratatui::{
        backend::CrosstermBackend,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::Span,
        widgets::{
            Block, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
            ScrollbarState,
        },
        Terminal, TerminalOptions, Viewport,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum MenuResult {
        Selected {
            filter_query: String,
            changed_query: bool,
            /// The selected path value.
            value: PathBuf,
        },
        Cancelled {
            filter_query: String,
            changed_query: bool,
        },
    }

    fn cursor_row_via_tty() -> Option<u16> {
        let mut tty = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .ok()?;

        tty.write_all(b"[6n").ok()?;
        tty.flush().ok()?;

        let mut buf = Vec::with_capacity(16);
        let mut byte = [0u8; 1];
        loop {
            tty.read_exact(&mut byte).ok()?;
            buf.push(byte[0]);
            if byte[0] == b'R' {
                break;
            }
            if buf.len() > 32 {
                return None;
            }
        }

        let s = std::str::from_utf8(&buf).ok()?;
        let inner = s.strip_prefix("[")?.strip_suffix('R')?;
        let (row_str, _col_str) = inner.split_once(';')?;
        let row: u16 = row_str.parse().ok()?;
        Some(row.saturating_sub(1))
    }

    struct CleanupGuard {
        prompt_row: u16,
        area: Rect,
    }

    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            let _ = execute!(stderr(), cursor::MoveTo(0, self.prompt_row));
            for row in self.area.top()..self.area.bottom() {
                let _ = execute!(
                    stderr(),
                    cursor::MoveTo(0, row),
                    terminal::Clear(terminal::ClearType::CurrentLine)
                );
            }
            let _ = execute!(stderr(), cursor::MoveTo(0, self.prompt_row), cursor::Show);
            let _ = terminal::disable_raw_mode();
        }
    }

    /// Re-query callback type: given a query string, returns fresh candidates.
    pub type QueryFn<'a> = Box<dyn Fn(&str) -> Vec<PathBuf> + 'a>;

    /// Compute a compact display label for a path:
    /// - relative to `cwd` if the path is under it (e.g. `Desktop`)
    /// - tilde-contracted if under `$HOME` (e.g. `~/code/dx`)
    /// - full absolute path otherwise
    fn display_label(path: &Path, cwd: &Path, home: Option<&Path>) -> String {
        if let Ok(rel) = path.strip_prefix(cwd) {
            let s = rel.display().to_string();
            if s.is_empty() { ".".to_string() } else { s }
        } else if let Some(h) = home {
            if let Ok(rel) = path.strip_prefix(h) {
                return format!("~/{}", rel.display());
            }
            path.display().to_string()
        } else {
            path.display().to_string()
        }
    }

    pub fn select(initial_candidates: Vec<PathBuf>, initial_query: &str, cwd: &Path, query_fn: QueryFn<'_>) -> Option<MenuResult> {
        if initial_candidates.is_empty() {
            return Some(MenuResult::Cancelled {
                filter_query: initial_query.to_string(),
                changed_query: false,
            });
        }

        if initial_candidates.len() == 1 {
            return Some(MenuResult::Selected {
                value: initial_candidates.into_iter().next().unwrap(),
                filter_query: initial_query.to_string(),
                changed_query: false,
            });
        }

        let (cols, rows) = terminal::size().ok()?;
        // Height will be recomputed once we know the initial candidate count.
        let list_rows = 10u16.min(initial_candidates.len() as u16);
        let height = list_rows + 3;

        let prompt_row = cursor_row_via_tty().unwrap_or(rows.saturating_sub(1));
        let rows_below = rows.saturating_sub(prompt_row + 1);

        let prompt_row = if rows_below < height {
            let scroll_needed = height - rows_below;
            let mut err = stderr();
            for _ in 0..scroll_needed {
                let _ = write!(err, "\n");
            }
            let _ = err.flush();
            prompt_row.saturating_sub(scroll_needed)
        } else {
            prompt_row
        };

        let menu_top = (prompt_row + 1).min(rows.saturating_sub(height));
        let area = Rect::new(0, menu_top, cols, height);

        terminal::enable_raw_mode().ok()?;
        execute!(stderr(), cursor::Hide).ok()?;

        let _guard = CleanupGuard { prompt_row, area };

        run_loop(initial_candidates, initial_query, cwd, area, &query_fn)
    }

    fn run_loop(
        initial_candidates: Vec<PathBuf>,
        initial_query: &str,
        cwd: &Path,
        area: Rect,
        query_fn: &QueryFn<'_>,
    ) -> Option<MenuResult> {
        let backend = CrosstermBackend::new(stderr());
        let mut terminal = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Fixed(area),
            },
        )
        .ok()?;

        let mut filter_query = initial_query.to_string();
        let mut candidates = initial_candidates;
        let mut list_state = ListState::default();
        if candidates.is_empty() {
            list_state.select(None);
        } else {
            list_state.select(Some(0));
        }

        let home = dirs::home_dir();

        loop {
            let selected_path = list_state
                .selected()
                .and_then(|i| candidates.get(i))
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(no matches)".to_string());

            let filter_label = if filter_query.is_empty() {
                "(empty)".to_string()
            } else {
                filter_query.clone()
            };

            terminal
                .draw(|frame| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(1), Constraint::Length(1)])
                        .split(frame.area());

                    let list_area = chunks[0];
                    let n = candidates.len();

                    let items: Vec<ListItem> = candidates
                        .iter()
                        .map(|p| ListItem::new(display_label(p, cwd, home.as_deref())))
                        .collect();

                    let list = List::new(items)
                        .block(Block::bordered())
                        .highlight_style(
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol("▸ ");

                    frame.render_stateful_widget(list, list_area, &mut list_state);

                    // Scrollbar — only rendered when candidates exceed visible rows.
                    // Inner height = list_area height minus 2 border rows.
                    let visible_rows = list_area.height.saturating_sub(2) as usize;
                    if n > visible_rows {
                        let selected = list_state.selected().unwrap_or(0);
                        let mut scrollbar_state = ScrollbarState::new(n)
                            .position(selected);
                        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                            .begin_symbol(None)
                            .end_symbol(None);
                        // Render inside the border: inset the list area by 1 on each side.
                        let scrollbar_area = Rect {
                            x: list_area.x,
                            y: list_area.y + 1,
                            width: list_area.width,
                            height: list_area.height.saturating_sub(2),
                        };
                        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
                    }

                    let status = Paragraph::new(Span::styled(
                        format!(" filter: {filter_label} | {selected_path}"),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::DIM),
                    ));
                    frame.render_widget(status, chunks[1]);
                })
                .ok()?;

            match event::read().ok()? {
                Event::Key(key) => match (key.code, key.modifiers) {
                    (KeyCode::Enter, _) => {
                        if let Some(idx) = list_state.selected()
                            && let Some(value) = candidates.get(idx).cloned()
                        {
                            return Some(MenuResult::Selected {
                                value,
                                filter_query: filter_query.clone(),
                                changed_query: filter_query != initial_query,
                            });
                        }
                    }
                    (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        return Some(MenuResult::Cancelled {
                            filter_query: filter_query.clone(),
                            changed_query: filter_query != initial_query,
                        });
                    }
                    (KeyCode::Down, _)
                    | (KeyCode::Tab, KeyModifiers::NONE)
                    | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                        move_selection(&mut list_state, candidates.len(), 1);
                    }
                    (KeyCode::Up, _)
                    | (KeyCode::BackTab, _)
                    | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                        move_selection(&mut list_state, candidates.len(), -1);
                    }
                    (KeyCode::Backspace, _) => {
                        filter_query.pop();
                        candidates = query_fn(&filter_query);
                        reset_selection(&mut list_state, candidates.len());
                    }
                    (KeyCode::Char(ch), KeyModifiers::NONE)
                    | (KeyCode::Char(ch), KeyModifiers::SHIFT) => {
                        if !ch.is_control() {
                            filter_query.push(ch);
                            candidates = query_fn(&filter_query);
                            reset_selection(&mut list_state, candidates.len());
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn reset_selection(state: &mut ListState, len: usize) {
        if len == 0 {
            state.select(None);
        } else {
            state.select(Some(0));
        }
    }

    fn move_selection(state: &mut ListState, len: usize, delta: isize) {
        if len == 0 {
            state.select(None);
            return;
        }
        let current = state.selected().unwrap_or(0) as isize;
        let next = (current + delta).rem_euclid(len as isize) as usize;
        state.select(Some(next));
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn display_label_relative_under_cwd() {
            let cwd = Path::new("/Users/nick");
            let path = Path::new("/Users/nick/Desktop");
            assert_eq!(display_label(path, cwd, None), "Desktop");
        }

        #[test]
        fn display_label_tilde_when_under_home_but_not_cwd() {
            let cwd = Path::new("/tmp");
            let home = Path::new("/Users/nick");
            let path = Path::new("/Users/nick/code/dx");
            assert_eq!(display_label(path, cwd, Some(home)), "~/code/dx");
        }

        #[test]
        fn display_label_absolute_when_outside_home() {
            let cwd = Path::new("/tmp");
            let home = Path::new("/Users/nick");
            let path = Path::new("/opt/homebrew/bin");
            assert_eq!(display_label(path, cwd, Some(home)), "/opt/homebrew/bin");
        }

        #[test]
        fn display_label_cwd_itself_shows_dot() {
            let cwd = Path::new("/Users/nick");
            let path = Path::new("/Users/nick");
            assert_eq!(display_label(path, cwd, None), ".");
        }
    }
}

#[cfg(not(unix))]
mod imp {
    use std::path::{Path, PathBuf};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum MenuResult {
        Selected {
            filter_query: String,
            changed_query: bool,
            value: PathBuf,
        },
        Cancelled {
            filter_query: String,
            changed_query: bool,
        },
    }

    pub type QueryFn<'a> = Box<dyn Fn(&str) -> Vec<PathBuf> + 'a>;

    pub fn select(_candidates: Vec<PathBuf>, initial_query: &str, _cwd: &Path, _query_fn: QueryFn<'_>) -> Option<MenuResult> {
        Some(MenuResult::Cancelled {
            filter_query: initial_query.to_string(),
            changed_query: false,
        })
    }
}

pub use imp::{select, MenuResult, QueryFn};
