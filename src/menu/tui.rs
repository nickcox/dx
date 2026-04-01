//! Interactive TUI selection for `dx menu`.
//!
//! On Unix, this renders to stderr (a TTY in interactive use) while
//! stdout stays free for JSON output to the calling shell hook.
//! crossterm is built with `use-dev-tty` so `event::read()` reads from
//! `/dev/tty` directly, making it work even when stdin is redirected.
//! On non-Unix platforms the selection is a no-op (returns `None`).

#[cfg(unix)]
mod imp {
    use std::io::stderr;
    use std::path::PathBuf;

    use crossterm::{
        cursor,
        event::{self, Event, KeyCode, KeyModifiers},
        execute,
        terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{
        backend::CrosstermBackend,
        style::{Color, Modifier, Style},
        widgets::{Block, List, ListItem, ListState},
        Terminal,
    };

    /// RAII guard that restores terminal state on drop, ensuring cleanup
    /// even on panics or early `?` returns.
    struct CleanupGuard;

    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            let _ = execute!(stderr(), LeaveAlternateScreen, cursor::Show);
            let _ = terminal::disable_raw_mode();
        }
    }

    /// Present an interactive selection menu rendered to stderr.
    ///
    /// Returns `Some(index)` when the user confirms a choice, or `None`
    /// on cancel (Esc / Ctrl-C) or if the TTY is unavailable.
    pub fn select(candidates: &[PathBuf]) -> Option<usize> {
        if candidates.is_empty() {
            return None;
        }

        terminal::enable_raw_mode().ok()?;
        execute!(stderr(), EnterAlternateScreen, cursor::Hide).ok()?;

        // Guard ensures cleanup on every exit path (normal, error, panic).
        let _guard = CleanupGuard;

        run_loop(candidates)
    }

    /// Inner event loop; returning `None` triggers guard cleanup via drop.
    fn run_loop(candidates: &[PathBuf]) -> Option<usize> {
        let backend = CrosstermBackend::new(stderr());
        let mut terminal = Terminal::new(backend).ok()?;

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
                        .block(Block::bordered().title(" dx "))
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

    /// Wrap-around selection movement.
    fn move_selection(state: &mut ListState, len: usize, delta: isize) {
        let current = state.selected().unwrap_or(0) as isize;
        let next = (current + delta).rem_euclid(len as isize) as usize;
        state.select(Some(next));
    }
}

#[cfg(not(unix))]
mod imp {
    use std::path::PathBuf;

    /// Non-Unix stub — always returns `None` (non-interactive fallback).
    pub fn select(_candidates: &[PathBuf]) -> Option<usize> {
        None
    }
}

pub use imp::select;
