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
    use std::path::PathBuf;

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
        widgets::{Block, List, ListItem, ListState, Paragraph},
        Terminal, TerminalOptions, Viewport,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum MenuResult {
        Selected {
            index: usize,
            filter_query: String,
            changed_query: bool,
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

    pub fn select(candidates: &[PathBuf], initial_query: &str) -> Option<MenuResult> {
        if candidates.is_empty() {
            return Some(MenuResult::Cancelled {
                filter_query: initial_query.to_string(),
                changed_query: false,
            });
        }

        if candidates.len() == 1 {
            return Some(MenuResult::Selected {
                index: 0,
                filter_query: initial_query.to_string(),
                changed_query: false,
            });
        }

        let (cols, rows) = terminal::size().ok()?;
        let list_rows = 10u16.min(candidates.len() as u16);
        let height = list_rows + 3;

        let prompt_row = cursor_row_via_tty().unwrap_or(rows.saturating_sub(1));
        let rows_below = rows.saturating_sub(prompt_row + 1);

        let prompt_row = if rows_below < height {
            let scroll_needed = height - rows_below;
            let mut err = stderr();
            for _ in 0..scroll_needed {
                let _ = write!(err, "
");
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

        run_loop(candidates, initial_query, area)
    }

    fn run_loop(candidates: &[PathBuf], initial_query: &str, area: Rect) -> Option<MenuResult> {
        let backend = CrosstermBackend::new(stderr());
        let mut terminal = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Fixed(area),
            },
        )
        .ok()?;

        let mut filter_query = initial_query.to_string();
        let mut filtered = filtered_indices(candidates, &filter_query);
        let mut list_state = ListState::default();
        if filtered.is_empty() {
            list_state.select(None);
        } else {
            list_state.select(Some(0));
        }

        loop {
            let selected_path = list_state
                .selected()
                .and_then(|i| filtered.get(i))
                .and_then(|idx| candidates.get(*idx))
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

                    let items: Vec<ListItem> = filtered
                        .iter()
                        .map(|idx| ListItem::new(candidates[*idx].display().to_string()))
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

                    frame.render_stateful_widget(list, chunks[0], &mut list_state);

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
                        if let Some(selected_visible_idx) = list_state.selected()
                            && let Some(selected_original_idx) = filtered.get(selected_visible_idx)
                        {
                            return Some(MenuResult::Selected {
                                index: *selected_original_idx,
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
                        move_selection(&mut list_state, filtered.len(), 1);
                    }
                    (KeyCode::Up, _)
                    | (KeyCode::BackTab, _)
                    | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                        move_selection(&mut list_state, filtered.len(), -1);
                    }
                    (KeyCode::Backspace, _) => {
                        filter_query.pop();
                        filtered = filtered_indices(candidates, &filter_query);
                        reset_selection(&mut list_state, filtered.len());
                    }
                    (KeyCode::Char(ch), KeyModifiers::NONE)
                    | (KeyCode::Char(ch), KeyModifiers::SHIFT) => {
                        if !ch.is_control() {
                            filter_query.push(ch);
                            filtered = filtered_indices(candidates, &filter_query);
                            reset_selection(&mut list_state, filtered.len());
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn is_path_like_query(query: &str) -> bool {
        query.contains('/') || query.starts_with('.') || query.starts_with('~')
    }

    fn filtered_indices(candidates: &[PathBuf], filter_query: &str) -> Vec<usize> {
        if filter_query.is_empty() {
            return (0..candidates.len()).collect();
        }

        let q = filter_query.to_ascii_lowercase();
        let path_like = is_path_like_query(filter_query);

        candidates
            .iter()
            .enumerate()
            .filter_map(|(idx, candidate)| {
                let full = candidate.display().to_string();
                let full_lc = full.to_ascii_lowercase();
                let matches = if path_like {
                    full_lc.starts_with(&q)
                } else {
                    let base = candidate
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    base.to_ascii_lowercase().starts_with(&q)
                };
                matches.then_some(idx)
            })
            .collect()
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
        fn filter_basename_prefix_for_non_path_query() {
            let candidates = vec![
                PathBuf::from("/Users/nick/Desktop"),
                PathBuf::from("/Users/nick/Documents"),
                PathBuf::from("/Users/nick/Downloads"),
                PathBuf::from("/Users/nick/Dropbox"),
            ];
            let idx = filtered_indices(&candidates, "Do");
            assert_eq!(idx, vec![1, 2]);
        }

        #[test]
        fn filter_full_path_prefix_for_path_like_query() {
            let candidates = vec![
                PathBuf::from("/Users/nick/Desktop"),
                PathBuf::from("/Users/nick/Documents"),
            ];
            let idx = filtered_indices(&candidates, "/Users/nick/D");
            assert_eq!(idx, vec![0, 1]);
        }
    }
}

#[cfg(not(unix))]
mod imp {
    use std::path::PathBuf;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum MenuResult {
        Selected {
            index: usize,
            filter_query: String,
            changed_query: bool,
        },
        Cancelled {
            filter_query: String,
            changed_query: bool,
        },
    }

    pub fn select(_candidates: &[PathBuf], initial_query: &str) -> Option<MenuResult> {
        Some(MenuResult::Cancelled {
            filter_query: initial_query.to_string(),
            changed_query: false,
        })
    }
}

pub use imp::{select, MenuResult};
