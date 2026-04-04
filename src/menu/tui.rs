//! Interactive TUI selection for `dx menu`.
//!
//! Renders an inline list immediately below the prompt line.
//! stdout stays free for JSON output; the TUI is drawn to stderr.
//! crossterm is built with `use-dev-tty` so `event::read()` reads from
//! `/dev/tty` directly, working even when stdin is redirected by a shell
//! completion hook.

#[cfg(unix)]
mod imp {
    use std::io::{stderr, BufWriter, Read, Write};
    use std::fs::OpenOptions;
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
        text::{Line, Span},
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

    fn use_dev_tty_backend() -> bool {
        std::env::var_os("DX_MENU_USE_DEV_TTY_BACKEND").is_some()
    }

    struct CleanupGuard {
        prompt_row: u16,
        area: Rect,
        use_tty_backend: bool,
    }

    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            if self.use_tty_backend {
                if let Ok(tty_file) = OpenOptions::new().write(true).open("/dev/tty") {
                    let mut tty = BufWriter::new(tty_file);
                    let _ = execute!(tty, cursor::MoveTo(0, self.prompt_row));
                    for row in self.area.top()..self.area.bottom() {
                        let _ = execute!(
                            tty,
                            cursor::MoveTo(0, row),
                            terminal::Clear(terminal::ClearType::CurrentLine)
                        );
                    }
                    let _ = execute!(tty, cursor::MoveTo(0, self.prompt_row), cursor::Show);
                }
            } else {
                let _ = execute!(stderr(), cursor::MoveTo(0, self.prompt_row));
                for row in self.area.top()..self.area.bottom() {
                    let _ = execute!(
                        stderr(),
                        cursor::MoveTo(0, row),
                        terminal::Clear(terminal::ClearType::CurrentLine)
                    );
                }
                let _ = execute!(stderr(), cursor::MoveTo(0, self.prompt_row), cursor::Show);
            }
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

    pub fn select(
        initial_candidates: Vec<PathBuf>,
        initial_query: &str,
        cwd: &Path,
        prompt_row_override: Option<u16>,
        item_max_len: Option<usize>,
        query_fn: QueryFn<'_>,
    ) -> Option<MenuResult> {
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
        let home = dirs::home_dir();
        let initial_labels: Vec<String> = initial_candidates
            .iter()
            .map(|p| display_label(p, cwd, home.as_deref()))
            .collect();
        let metrics = compute_layout_metrics(
            cols.saturating_sub(2) as usize,
            initial_candidates.len(),
            &initial_labels,
            item_max_len,
        );
        let list_rows = 10u16.min(metrics.rows_total.max(1) as u16);
        let height = list_rows + 3;

        let prompt_row = if let Some(row) = prompt_row_override {
            row.min(rows.saturating_sub(1))
        } else if std::env::var_os("DX_MENU_NO_CURSOR_QUERY").is_some() {
            rows.saturating_sub(height + 1)
        } else {
            cursor_row_via_tty().unwrap_or(rows.saturating_sub(1))
        };
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

        let use_tty_backend = use_dev_tty_backend();

        terminal::enable_raw_mode().ok()?;
        if use_tty_backend {
            let tty_file = OpenOptions::new().write(true).open("/dev/tty").ok()?;
            let mut tty = BufWriter::new(tty_file);
            execute!(tty, cursor::Hide).ok()?;
        } else {
            execute!(stderr(), cursor::Hide).ok()?;
        }

        let _guard = CleanupGuard {
            prompt_row,
            area,
            use_tty_backend,
        };

        run_loop(initial_candidates, initial_query, cwd, area, use_tty_backend, item_max_len, &query_fn)
    }

    fn run_loop(
        initial_candidates: Vec<PathBuf>,
        initial_query: &str,
        cwd: &Path,
        area: Rect,
        use_tty_backend: bool,
        item_max_len: Option<usize>,
        query_fn: &QueryFn<'_>,
    ) -> Option<MenuResult> {
        let writer: Box<dyn Write> = if use_tty_backend {
            Box::new(OpenOptions::new().write(true).open("/dev/tty").ok()?)
        } else {
            Box::new(stderr())
        };

        let backend = CrosstermBackend::new(writer);
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
                    let selected = list_state.selected().unwrap_or(0);
                    let inner_width = list_area.width.saturating_sub(2) as usize;
                    let visible_rows = list_area.height.saturating_sub(2) as usize;
                    let labels: Vec<String> = candidates
                        .iter()
                        .map(|p| display_label(p, cwd, home.as_deref()))
                        .collect();
                    let metrics = compute_layout_metrics(inner_width, n, &labels, item_max_len);
                    let columns = metrics.columns;
                    let use_grid = metrics.use_grid;

                    if use_grid {
                        let rows_total = metrics.rows_total;
                        let selected_row = selected / columns;
                        let top_row = if visible_rows == 0 {
                            0
                        } else if selected_row >= visible_rows {
                            selected_row - visible_rows + 1
                        } else {
                            0
                        };

                        let mut lines: Vec<Line> = Vec::new();
                        for vr in 0..visible_rows {
                            let row = top_row + vr;
                            if row >= rows_total {
                                lines.push(Line::from(""));
                                continue;
                            }

                            let mut spans: Vec<Span> = Vec::new();
                            for col in 0..columns {
                                let idx = row * columns + col;
                                if idx >= n {
                                    break;
                                }
                                let label = display_label(&candidates[idx], cwd, home.as_deref());
                                let content_width = metrics.column_widths[col].saturating_sub(2).max(1);
                                let trunc = truncate_for_cell(&label, content_width);
                                let text = pad_to_width(&trunc, metrics.column_widths[col]);
                                let span = if idx == selected {
                                    Span::styled(
                                        text,
                                        Style::default()
                                            .fg(Color::Black)
                                            .bg(Color::Cyan)
                                            .add_modifier(Modifier::BOLD),
                                    )
                                } else {
                                    Span::raw(text)
                                };
                                spans.push(span);
                            }
                            lines.push(Line::from(spans));
                        }

                        let grid = Paragraph::new(lines).block(Block::bordered());
                        frame.render_widget(grid, list_area);

                        if rows_total > visible_rows && visible_rows > 0 {
                            let mut scrollbar_state = ScrollbarState::new(rows_total)
                                .position(selected_row);
                            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                                .begin_symbol(None)
                                .end_symbol(None);
                            let scrollbar_area = Rect {
                                x: list_area.x,
                                y: list_area.y + 1,
                                width: list_area.width,
                                height: list_area.height.saturating_sub(2),
                            };
                            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
                        }
                    } else {
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

                        if n > visible_rows && visible_rows > 0 {
                            let mut scrollbar_state = ScrollbarState::new(n)
                                .position(selected);
                            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                                .begin_symbol(None)
                                .end_symbol(None);
                            let scrollbar_area = Rect {
                                x: list_area.x,
                                y: list_area.y + 1,
                                width: list_area.width,
                                height: list_area.height.saturating_sub(2),
                            };
                            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
                        }
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
                Event::Key(key) => {
                    let len = candidates.len();
                    let labels: Vec<String> = candidates
                        .iter()
                        .map(|p| display_label(p, cwd, home.as_deref()))
                        .collect();
                    let metrics = compute_layout_metrics(
                        area.width.saturating_sub(2) as usize,
                        len,
                        &labels,
                        item_max_len,
                    );
                    let columns = metrics.columns;
                    let use_grid = metrics.use_grid;

                    match (key.code, key.modifiers) {
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
                        (KeyCode::Right, _) if use_grid => {
                            move_selection(&mut list_state, len, 1);
                        }
                        (KeyCode::Left, _) if use_grid => {
                            move_selection(&mut list_state, len, -1);
                        }
                        (KeyCode::Down, _) if use_grid => {
                            move_selection_grid_vertical(&mut list_state, len, columns, 1);
                        }
                        (KeyCode::Up, _) if use_grid => {
                            move_selection_grid_vertical(&mut list_state, len, columns, -1);
                        }
                        (KeyCode::Tab, KeyModifiers::NONE) if use_grid => {
                            move_selection(&mut list_state, len, 1);
                        }
                        (KeyCode::BackTab, _) if use_grid => {
                            move_selection(&mut list_state, len, -1);
                        }
                        (KeyCode::Down, _)
                        | (KeyCode::Tab, KeyModifiers::NONE)
                        | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                            move_selection(&mut list_state, len, 1);
                        }
                        (KeyCode::Up, _)
                        | (KeyCode::BackTab, _)
                        | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                            move_selection(&mut list_state, len, -1);
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
                    }
                }
                _ => {}
            }
        }
    }

    #[derive(Debug, Clone)]
    struct LayoutMetrics {
        columns: usize,
        rows_total: usize,
        use_grid: bool,
        column_widths: Vec<usize>,
    }

    fn compute_layout_metrics(
        inner_width: usize,
        item_count: usize,
        labels: &[String],
        item_max_len: Option<usize>,
    ) -> LayoutMetrics {
        let effective_max = effective_item_max_len(labels, item_max_len);
        let base_cell_width = effective_max.map(|m| m + 2).unwrap_or(inner_width.max(1));
        let raw_columns = effective_max
            .map(|_| std::cmp::max(1, inner_width / std::cmp::max(1, base_cell_width)))
            .unwrap_or(1);
        let columns = if item_count == 0 {
            raw_columns
        } else {
            std::cmp::max(1, std::cmp::min(raw_columns, item_count))
        };
        let use_grid = columns > 1;
        let rows_total = if item_count == 0 {
            0
        } else {
            item_count.div_ceil(columns)
        };

        let mut column_widths = vec![base_cell_width.max(1); columns];
        if columns > 0 {
            let used = base_cell_width.saturating_mul(columns);
            let remainder = inner_width.saturating_sub(used);
            let extra_each = remainder / columns;
            let extra_left = remainder % columns;
            for (idx, width) in column_widths.iter_mut().enumerate() {
                *width = width.saturating_add(extra_each);
                if idx < extra_left {
                    *width = width.saturating_add(1);
                }
            }
        }

        LayoutMetrics {
            columns,
            rows_total,
            use_grid,
            column_widths,
        }
    }

    fn effective_item_max_len(labels: &[String], item_max_len: Option<usize>) -> Option<usize> {
        let configured = item_max_len?;
        if configured < 1 {
            return None;
        }
        let actual = labels
            .iter()
            .map(|s| s.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        Some(std::cmp::min(configured, actual))
    }

    fn truncate_for_cell(input: &str, max: usize) -> String {
        if max == 0 {
            return String::new();
        }
        let count = input.chars().count();
        if count <= max {
            return input.to_string();
        }
        if max == 1 {
            return "…".to_string();
        }
        let tail_len = max - 1;
        let tail: String = input
            .chars()
            .rev()
            .take(tail_len)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("…{tail}")
    }

    fn pad_to_width(input: &str, width: usize) -> String {
        let mut out = input.to_string();
        let len = out.chars().count();
        if width > len {
            out.push_str(&" ".repeat(width - len));
        }
        out
    }

    fn move_selection_grid_vertical(
        state: &mut ListState,
        len: usize,
        columns: usize,
        direction: isize,
    ) {
        if len == 0 || columns == 0 {
            state.select(None);
            return;
        }

        let idx = state.selected().unwrap_or(0);
        let col = idx % columns;
        let row = idx / columns;
        let rows = len.div_ceil(columns);

        let next = if direction >= 0 {
            let direct = (row + 1) * columns + col;
            if row + 1 < rows && direct < len {
                direct
            } else {
                let next_col = (col + 1) % columns;
                next_col
            }
        } else if row > 0 {
            (row - 1) * columns + col
        } else {
            let prev_col = if col == 0 { columns - 1 } else { col - 1 };
            let mut prev_row = rows - 1;
            loop {
                let candidate = prev_row * columns + prev_col;
                if candidate < len {
                    break candidate;
                }
                if prev_row == 0 {
                    break prev_col;
                }
                prev_row -= 1;
            }
        };

        state.select(Some(next));
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

        #[test]
        fn truncate_for_cell_uses_ellipsis_and_tail() {
            assert_eq!(truncate_for_cell("abcdef", 4), "…def");
            assert_eq!(truncate_for_cell("abcdef", 1), "…");
            assert_eq!(truncate_for_cell("abc", 5), "abc");
        }

        #[test]
        fn pad_to_width_fills_column_width() {
            assert_eq!(pad_to_width("ab", 6), "ab    ");
        }

        #[test]
        fn compute_layout_metrics_distributes_remainder() {
            let labels = vec!["abcdefgh".to_string(), "beta".to_string(), "gamma".to_string()];
            let m = compute_layout_metrics(43, 3, &labels, Some(8));
            assert_eq!(m.columns, 3);
            assert_eq!(m.column_widths, vec![15, 14, 14]);
        }

        #[test]
        fn effective_item_max_len_uses_actual_max() {
            let labels = vec!["a".to_string(), "abc".to_string()];
            assert_eq!(effective_item_max_len(&labels, Some(10)), Some(3));
            assert_eq!(effective_item_max_len(&labels, Some(2)), Some(2));
            assert_eq!(effective_item_max_len(&labels, None), None);
        }

        #[test]
        fn move_selection_grid_vertical_wraps_to_adjacent_column() {
            let mut state = ListState::default();

            // Grid for len=7, cols=3:
            // [0,1,2]
            // [3,4,5]
            // [6]

            state.select(Some(6));
            move_selection_grid_vertical(&mut state, 7, 3, 1);
            assert_eq!(state.selected(), Some(1));

            state.select(Some(1));
            move_selection_grid_vertical(&mut state, 7, 3, -1);
            assert_eq!(state.selected(), Some(6));

            state.select(Some(5));
            move_selection_grid_vertical(&mut state, 7, 3, 1);
            assert_eq!(state.selected(), Some(0));

            state.select(Some(0));
            move_selection_grid_vertical(&mut state, 7, 3, -1);
            assert_eq!(state.selected(), Some(5));
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

    pub fn select(
        _candidates: Vec<PathBuf>,
        initial_query: &str,
        _cwd: &Path,
        _prompt_row_override: Option<u16>,
        _item_max_len: Option<usize>,
        _query_fn: QueryFn<'_>,
    ) -> Option<MenuResult> {
        Some(MenuResult::Cancelled {
            filter_query: initial_query.to_string(),
            changed_query: false,
        })
    }
}

pub use imp::{select, MenuResult, QueryFn};
