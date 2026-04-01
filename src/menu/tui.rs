//! Interactive TUI selection for `dx menu`.
//!
//! Renders an inline list immediately below the prompt line.
//! stdout stays free for JSON output; the TUI is drawn to stderr.
//! crossterm is built with `use-dev-tty` so `event::read()` reads from
//! `/dev/tty` directly, working even when stdin is redirected by a shell
//! completion hook.
//!
//! We need the cursor row to position the menu, but we cannot use
//! `Viewport::Inline` (which queries `\033[6n` and reads the response
//! from stdin — lost in a `$(...)` pipe context).  Instead we open
//! `/dev/tty` directly for the query/response roundtrip, then use
//! `Viewport::Fixed` anchored just below the prompt row.

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
        layout::Rect,
        style::{Color, Modifier, Style},
        widgets::{Block, List, ListItem, ListState},
        Terminal, TerminalOptions, Viewport,
    };

    const MENU_HEIGHT: u16 = 10;

    /// Query the cursor row by writing `\033[6n` to and reading `\033[<row>;<col>R`
    /// from `/dev/tty` directly.  This works even when stdout is a pipe.
    fn cursor_row_via_tty() -> Option<u16> {
        let mut tty = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .ok()?;

        tty.write_all(b"\x1b[6n").ok()?;
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
        let inner = s.strip_prefix("\x1b[")?.strip_suffix('R')?;
        let (row_str, _col_str) = inner.split_once(';')?;
        let row: u16 = row_str.parse().ok()?;
        Some(row.saturating_sub(1)) // convert 1-based to 0-based
    }

    /// RAII guard: moves cursor back to the prompt row, clears the menu
    /// area, and restores terminal state on drop.
    struct CleanupGuard {
        prompt_row: u16,
        area: Rect,
    }

    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            // Move cursor back to prompt row first so the shell redraws
            // the prompt in the right place after we exit.
            let _ = execute!(stderr(), cursor::MoveTo(0, self.prompt_row));
            // Clear the menu lines.
            for row in self.area.top()..self.area.bottom() {
                let _ = execute!(
                    stderr(),
                    cursor::MoveTo(0, row),
                    terminal::Clear(terminal::ClearType::CurrentLine)
                );
            }
            // Return cursor to prompt row and restore state.
            let _ = execute!(stderr(), cursor::MoveTo(0, self.prompt_row), cursor::Show);
            let _ = terminal::disable_raw_mode();
        }
    }

    /// Present an inline selection list immediately below the prompt line.
    ///
    /// Returns `Some(index)` on Enter, `None` on Esc / Ctrl-C / error.
    pub fn select(candidates: &[PathBuf]) -> Option<usize> {
        if candidates.is_empty() {
            return None;
        }

        let (cols, rows) = terminal::size().ok()?;
        let height = MENU_HEIGHT.min(candidates.len() as u16 + 2); // +2 for border

        // Find out where the cursor (prompt line) currently is.
        let prompt_row = cursor_row_via_tty().unwrap_or(rows.saturating_sub(1));

        // How many rows are available below the prompt?
        let rows_below = rows.saturating_sub(prompt_row + 1);

        // If there isn't enough room, scroll up to make space.
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

        run_loop(candidates, area)
    }

    fn run_loop(candidates: &[PathBuf], area: Rect) -> Option<usize> {
        let backend = CrosstermBackend::new(stderr());
        let mut terminal = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Fixed(area),
            },
        )
        .ok()?;

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        loop {
            terminal
                .draw(|frame| {
                    let items: Vec<ListItem> = candidates
                        .iter()
                        .map(|p| ListItem::new(p.display().to_string()))
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

                    frame.render_stateful_widget(list, frame.area(), &mut list_state);
                })
                .ok()?;

            match event::read().ok()? {
                Event::Key(key) => match (key.code, key.modifiers) {
                    (KeyCode::Enter, _) => return list_state.selected(),
                    (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        return None;
                    }
                    (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                        move_selection(&mut list_state, candidates.len(), -1);
                    }
                    (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                        move_selection(&mut list_state, candidates.len(), 1);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn move_selection(state: &mut ListState, len: usize, delta: isize) {
        let current = state.selected().unwrap_or(0) as isize;
        let next = (current + delta).rem_euclid(len as isize) as usize;
        state.select(Some(next));
    }
}

#[cfg(not(unix))]
mod imp {
    use std::path::PathBuf;

    pub fn select(_candidates: &[PathBuf]) -> Option<usize> {
        None
    }
}

pub use imp::select;
