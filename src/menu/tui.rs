//! Interactive TUI selection for `dx menu`.
//!
//! Renders an inline list immediately below the prompt line.
//! stdout stays free for JSON output; the TUI is drawn to stderr.
//! crossterm is built with `use-dev-tty` so `event::read()` reads from
//! `/dev/tty` directly, working even when stdin is redirected by a shell
//! completion hook.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuResult {
    Selected {
        filter_query: String,
        changed_query: bool,
        value: std::path::PathBuf,
        terminal: crate::menu::action::TerminalState,
    },
    Cancelled {
        filter_query: String,
        changed_query: bool,
    },
}

// The interactive menu TUI currently targets Unix TTY semantics (`/dev/tty`,
// cursor queries, explicit terminal scrolling). Non-Unix builds use the stub
// implementation below, which preserves the JSON/noop contract for shell
// fallback paths without enabling the inline TUI yet.
#[cfg(unix)]
mod imp {
    use std::fs::OpenOptions;
    use std::io::{BufWriter, Read, Write, stderr};
    use std::path::{Path, PathBuf};

    use crossterm::{
        cursor,
        event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
        execute, terminal,
    };
    use ratatui::{
        Terminal, TerminalOptions, Viewport,
        backend::CrosstermBackend,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{
            Block, Clear, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
            ScrollbarState,
        },
    };

    use crate::resolve::CompletionCandidates;

    use super::MenuResult;

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
        use_tty_backend: bool,
    }

    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            if self.use_tty_backend {
                if let Ok(tty_file) = OpenOptions::new().write(true).open("/dev/tty") {
                    let mut tty = BufWriter::new(tty_file);
                    let _ = execute!(tty, cursor::MoveTo(0, self.prompt_row));
                    for row in self.prompt_row.saturating_add(1)..self.area.bottom() {
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
                for row in self.prompt_row.saturating_add(1)..self.area.bottom() {
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
    pub type QueryFn<'a> = Box<dyn Fn(&str) -> CompletionCandidates + 'a>;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct FilterState {
        // Interactive edits can only refine the initial query; they never broaden it.
        initial_query: String,
        typed_refinement: String,
    }

    impl FilterState {
        fn new(initial_query: &str) -> Self {
            Self {
                initial_query: initial_query.to_string(),
                typed_refinement: String::new(),
            }
        }

        fn effective_query(&self) -> String {
            format!("{}{}", self.initial_query, self.typed_refinement)
        }

        fn changed_query(&self) -> bool {
            !self.typed_refinement.is_empty()
        }

        fn typed_refinement(&self) -> &str {
            &self.typed_refinement
        }

        fn push(&mut self, ch: char) {
            self.typed_refinement.push(ch);
        }

        fn backspace(&mut self) -> bool {
            self.typed_refinement.pop().is_some()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum MenuKeyAction {
        Submit,
        Cancel,
        MoveLinear(isize),
        MoveGridVertical(isize),
        Backspace,
        InputChar(char),
        Ignore,
    }

    fn map_key_event(key: KeyEvent, use_grid: bool) -> MenuKeyAction {
        match (key.code, key.modifiers) {
            (KeyCode::Enter, _) => MenuKeyAction::Submit,
            (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                MenuKeyAction::Cancel
            }
            (KeyCode::Right, _) if use_grid => MenuKeyAction::MoveLinear(1),
            (KeyCode::Left, _) if use_grid => MenuKeyAction::MoveLinear(-1),
            (KeyCode::Down, _) if use_grid => MenuKeyAction::MoveGridVertical(1),
            (KeyCode::Up, _) if use_grid => MenuKeyAction::MoveGridVertical(-1),
            (KeyCode::Tab, KeyModifiers::NONE) if use_grid => MenuKeyAction::MoveLinear(1),
            (KeyCode::BackTab, _) if use_grid => MenuKeyAction::MoveLinear(-1),
            (KeyCode::Down, _) | (KeyCode::Tab, KeyModifiers::NONE) => MenuKeyAction::MoveLinear(1),
            (KeyCode::Up, _) | (KeyCode::BackTab, _) => MenuKeyAction::MoveLinear(-1),
            (KeyCode::Backspace, _) => MenuKeyAction::Backspace,
            (KeyCode::Char(ch), KeyModifiers::NONE) | (KeyCode::Char(ch), KeyModifiers::SHIFT) => {
                if ch.is_control() {
                    MenuKeyAction::Ignore
                } else {
                    MenuKeyAction::InputChar(ch)
                }
            }
            _ => MenuKeyAction::Ignore,
        }
    }

    fn selected_result(filter_state: &FilterState, value: PathBuf) -> MenuResult {
        MenuResult::Selected {
            value,
            filter_query: filter_state.effective_query(),
            changed_query: filter_state.changed_query(),
            terminal: crate::menu::action::TerminalState::Dirty,
        }
    }

    fn cancelled_result(filter_state: &FilterState) -> MenuResult {
        MenuResult::Cancelled {
            filter_query: filter_state.effective_query(),
            changed_query: filter_state.changed_query(),
        }
    }

    fn apply_filter_edit(
        filter_state: &mut FilterState,
        completion: &mut CompletionCandidates,
        list_state: &mut ListState,
        action: MenuKeyAction,
        query_fn: &QueryFn<'_>,
    ) {
        let query_changed = match action {
            MenuKeyAction::Backspace => filter_state.backspace(),
            MenuKeyAction::InputChar(ch) => {
                filter_state.push(ch);
                true
            }
            _ => return,
        };

        if !query_changed {
            return;
        }

        let query = filter_state.effective_query();
        *completion = query_fn(&query);
        reset_selection(list_state, completion.paths.len());
    }

    fn selected_label(labels: &[String], selected: Option<usize>) -> String {
        selected
            .and_then(|idx| labels.get(idx))
            .cloned()
            .unwrap_or_else(|| "(no matches)".to_string())
    }

    /// Compute a compact display label for a path:
    /// - relative to `cwd` if the path is under it (e.g. `Desktop`)
    /// - tilde-contracted if under `$HOME` (e.g. `~/code/dx`)
    /// - full absolute path otherwise
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

    fn display_label(
        path: &Path,
        cwd: &Path,
        home: Option<&Path>,
        prefer_relative_paths: bool,
    ) -> String {
        if prefer_relative_paths && let Ok(rel) = path.strip_prefix(cwd) {
            use std::path::Component;

            let cleaned = sanitize_relative_components(rel);
            if cleaned.as_os_str().is_empty() {
                "./".to_string()
            } else {
                let starts_with_parent = cleaned
                    .components()
                    .next()
                    .is_some_and(|component| matches!(component, Component::ParentDir));
                if starts_with_parent {
                    cleaned.display().to_string()
                } else {
                    format!("./{}", cleaned.display())
                }
            }
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
        initial_candidates: CompletionCandidates,
        initial_query: &str,
        cwd: &Path,
        prefer_relative_paths: bool,
        prompt_row_override: Option<u16>,
        max_rows: u16,
        item_max_len: Option<usize>,
        show_border: bool,
        psreadline_mode: bool,
        query_fn: QueryFn<'_>,
    ) -> Option<MenuResult> {
        if initial_candidates.paths.is_empty() {
            return Some(MenuResult::Cancelled {
                filter_query: initial_query.to_string(),
                changed_query: false,
            });
        }

        if initial_candidates.paths.len() == 1 && !initial_candidates.has_more {
            return Some(MenuResult::Selected {
                value: initial_candidates.paths.into_iter().next().unwrap(),
                filter_query: initial_query.to_string(),
                changed_query: false,
                terminal: crate::menu::action::TerminalState::Clean,
            });
        }

        let (cols, rows) = terminal::size().ok()?;
        let home = dirs::home_dir();
        let initial_labels: Vec<String> = initial_candidates
            .paths
            .iter()
            .map(|p| display_label(p, cwd, home.as_deref(), prefer_relative_paths))
            .collect();
        let height = compute_rendered_height(
            cols,
            initial_candidates.paths.len(),
            &initial_labels,
            max_rows,
            item_max_len,
            show_border,
        );
        let required_rows = required_rows_below(height, show_border);

        let skip_cursor_query = psreadline_mode;
        let prompt_row = if let Some(row) = prompt_row_override {
            row.min(rows.saturating_sub(1))
        } else if skip_cursor_query {
            rows.saturating_sub(required_rows + 1)
        } else {
            cursor_row_via_tty().unwrap_or(rows.saturating_sub(1))
        };
        let prompt_row = reserve_space_on_tty(prompt_row, rows, required_rows);

        let menu_top = menu_top_row(prompt_row, rows, height, show_border);
        let area = Rect::new(0, menu_top, cols, height);

        let use_tty_backend = psreadline_mode;

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

        run_loop(
            initial_candidates,
            initial_query,
            cwd,
            prefer_relative_paths,
            area,
            max_rows,
            use_tty_backend,
            item_max_len,
            show_border,
            &query_fn,
        )
    }

    fn render_grid_items(
        completion: &CompletionCandidates,
        labels: &[String],
        layout: &MenuLayoutPlan,
        selected: usize,
    ) -> Vec<Line<'static>> {
        let n = completion.paths.len();
        let visible_rows = layout.visible_rows;
        let metrics = &layout.metrics;
        let columns = metrics.columns;
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
                let content_width = metrics.column_widths[col].saturating_sub(2).max(1);
                let trunc = truncate_for_cell(&labels[idx], content_width);
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

        lines
    }

    fn render_grid(
        frame: &mut ratatui::Frame<'_>,
        completion: &CompletionCandidates,
        labels: &[String],
        layout: &MenuLayoutPlan,
        content_area: Rect,
        scrollbar_area: Option<Rect>,
        selected: usize,
        show_border: bool,
    ) {
        let lines = render_grid_items(completion, labels, layout, selected);
        let mut grid = Paragraph::new(lines);
        if show_border {
            grid = grid.block(Block::bordered());
        }
        frame.render_widget(grid, content_area);

        if layout.scrollbar_needed {
            let selected_row = selected / layout.metrics.columns;
            let mut scrollbar_state =
                ScrollbarState::new(layout.metrics.rows_total).position(selected_row);
            let scrollbar = build_scrollbar(show_border);
            let scrollbar_render_area =
                menu_scrollbar_render_area(content_area, scrollbar_area, show_border);
            frame.render_stateful_widget(scrollbar, scrollbar_render_area, &mut scrollbar_state);
        }
    }

    fn render_list(
        frame: &mut ratatui::Frame<'_>,
        labels: &[String],
        layout: &MenuLayoutPlan,
        content_area: Rect,
        scrollbar_area: Option<Rect>,
        list_state: &mut ListState,
        show_border: bool,
    ) {
        let items: Vec<ListItem> = labels.iter().cloned().map(ListItem::new).collect();

        let mut list = List::new(items)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸ ");
        if show_border {
            list = list.block(Block::bordered());
        }

        frame.render_stateful_widget(list, content_area, list_state);

        if layout.scrollbar_needed {
            let selected = list_state.selected().unwrap_or(0);
            let mut scrollbar_state = ScrollbarState::new(labels.len()).position(selected);
            let scrollbar = build_scrollbar(show_border);
            let scrollbar_render_area =
                menu_scrollbar_render_area(content_area, scrollbar_area, show_border);
            frame.render_stateful_widget(scrollbar, scrollbar_render_area, &mut scrollbar_state);
        }
    }

    fn render_status(
        frame: &mut ratatui::Frame<'_>,
        chunks: &[Rect],
        divider_area: Option<Rect>,
        selected_path: &str,
        overflow: &str,
        typed_refinement: &str,
    ) {
        let status_text =
            build_status_text(chunks[1].width, selected_path, overflow, typed_refinement);
        let status = Paragraph::new(Span::styled(
            status_text,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::DIM),
        ));
        if let Some(divider_area) = divider_area {
            let divider = Paragraph::new("─".repeat(divider_area.width as usize)).style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            );
            frame.render_widget(divider, divider_area);
        }
        frame.render_widget(status, chunks[1]);
    }

    fn run_loop(
        initial_candidates: CompletionCandidates,
        initial_query: &str,
        cwd: &Path,
        prefer_relative_paths: bool,
        area: Rect,
        max_rows: u16,
        use_tty_backend: bool,
        item_max_len: Option<usize>,
        show_border: bool,
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

        let mut filter_state = FilterState::new(initial_query);
        let mut completion = initial_candidates;
        let mut list_state = ListState::default();
        if completion.paths.is_empty() {
            list_state.select(None);
        } else {
            list_state.select(Some(0));
        }

        let home = dirs::home_dir();
        let mut previous_height = area.height;

        loop {
            let labels: Vec<String> = completion
                .paths
                .iter()
                .map(|p| display_label(p, cwd, home.as_deref(), prefer_relative_paths))
                .collect();
            let target_height = compute_rendered_height(
                area.width,
                completion.paths.len(),
                &labels,
                max_rows,
                item_max_len,
                show_border,
            );
            let current_height = bounded_rendered_height(area.height, target_height);
            let current_area = rendered_menu_area(area, current_height);
            let list_area = Rect {
                x: current_area.x,
                y: current_area.y,
                width: current_area.width,
                height: current_area.height.saturating_sub(1),
            };
            let layout = build_menu_layout(
                list_area,
                show_border,
                completion.paths.len(),
                &labels,
                item_max_len,
            );

            let selected_path = selected_label(&labels, list_state.selected());

            let overflow = overflow_note(completion.paths.len(), completion.has_more);

            terminal
                .draw(|frame| {
                    frame.render_widget(Clear, current_area);
                    if let Some(vacated_area) =
                        cleared_trailing_area(frame.area(), previous_height, current_height)
                    {
                        frame.render_widget(Clear, vacated_area);
                    }

                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(1), Constraint::Length(1)])
                        .split(current_area);

                    let selected = list_state.selected().unwrap_or(0);
                    let content_area = layout.content_area;
                    let divider_area = layout.divider_area;
                    let scrollbar_area = layout.scrollbar_area;
                    let use_grid = layout.metrics.use_grid;

                    if use_grid {
                        render_grid(
                            frame,
                            &completion,
                            &labels,
                            &layout,
                            content_area,
                            scrollbar_area,
                            selected,
                            show_border,
                        );
                    } else {
                        render_list(
                            frame,
                            &labels,
                            &layout,
                            content_area,
                            scrollbar_area,
                            &mut list_state,
                            show_border,
                        );
                    }

                    render_status(
                        frame,
                        &chunks,
                        divider_area,
                        &selected_path,
                        &overflow,
                        filter_state.typed_refinement(),
                    );
                })
                .ok()?;

            previous_height = current_height;

            if let Event::Key(key) = event::read().ok()? {
                let len = completion.paths.len();
                let columns = layout.metrics.columns;
                let use_grid = layout.metrics.use_grid;

                let action = map_key_event(key, use_grid);

                match action {
                    MenuKeyAction::Submit => {
                        if let Some(idx) = list_state.selected()
                            && let Some(value) = completion.paths.get(idx).cloned()
                        {
                            return Some(selected_result(&filter_state, value));
                        }
                    }
                    MenuKeyAction::Cancel => {
                        return Some(cancelled_result(&filter_state));
                    }
                    MenuKeyAction::MoveLinear(delta) => {
                        move_selection(&mut list_state, len, delta);
                    }
                    MenuKeyAction::MoveGridVertical(direction) => {
                        move_selection_grid_vertical(&mut list_state, len, columns, direction);
                    }
                    MenuKeyAction::Backspace | MenuKeyAction::InputChar(_) => apply_filter_edit(
                        &mut filter_state,
                        &mut completion,
                        &mut list_state,
                        action,
                        query_fn,
                    ),
                    MenuKeyAction::Ignore => {}
                }
            }
        }
    }

    fn compute_list_rows(rows_total: usize, max_rows: u16) -> u16 {
        let cap = max_rows.max(1);
        cap.min(rows_total.max(1) as u16)
    }

    fn compute_rendered_height(
        width: u16,
        item_count: usize,
        labels: &[String],
        max_rows: u16,
        item_max_len: Option<usize>,
        show_border: bool,
    ) -> u16 {
        let frame_inset = if show_border { 2 } else { 0 };
        let metrics = compute_layout_metrics(
            width.saturating_sub(frame_inset) as usize,
            item_count,
            labels,
            item_max_len,
        );
        let list_rows = compute_list_rows(metrics.rows_total, max_rows);
        list_rows + menu_chrome_height(show_border)
    }

    fn menu_chrome_height(show_border: bool) -> u16 {
        if show_border { 3 } else { 2 }
    }

    fn bounded_rendered_height(reserved_height: u16, target_height: u16) -> u16 {
        reserved_height.min(target_height)
    }

    fn rendered_menu_area(area: Rect, rendered_height: u16) -> Rect {
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: rendered_height.min(area.height),
        }
    }

    fn cleared_trailing_area(
        area: Rect,
        previous_height: u16,
        current_height: u16,
    ) -> Option<Rect> {
        if current_height >= previous_height {
            return None;
        }

        Some(Rect {
            x: area.x,
            y: area.y + current_height,
            width: area.width,
            height: previous_height - current_height,
        })
    }

    fn prompt_gap_rows(show_border: bool) -> u16 {
        if show_border { 0 } else { 1 }
    }

    fn required_rows_below(rendered_height: u16, show_border: bool) -> u16 {
        rendered_height.saturating_add(prompt_gap_rows(show_border))
    }

    fn menu_top_row(
        prompt_row: u16,
        terminal_rows: u16,
        rendered_height: u16,
        show_border: bool,
    ) -> u16 {
        prompt_row
            .saturating_add(1)
            .saturating_add(prompt_gap_rows(show_border))
            .min(terminal_rows.saturating_sub(rendered_height))
    }

    fn scroll_rows_needed(prompt_row: u16, terminal_rows: u16, needed_height: u16) -> u16 {
        let rows_below = terminal_rows.saturating_sub(prompt_row + 1);
        needed_height.saturating_sub(rows_below)
    }

    // Reserve vertical space for the inline menu by scrolling the active TTY
    // viewport upward when there are not enough rows below the prompt. This is
    // Unix-only for now because it depends on `/dev/tty` and the Unix TUI path.
    fn reserve_space_on_tty(prompt_row: u16, terminal_rows: u16, needed_height: u16) -> u16 {
        let scroll_needed = scroll_rows_needed(prompt_row, terminal_rows, needed_height);
        if scroll_needed == 0 {
            return prompt_row;
        }

        let next_prompt_row = prompt_row.saturating_sub(scroll_needed);

        if let Ok(tty_file) = OpenOptions::new().write(true).open("/dev/tty") {
            let mut tty = BufWriter::new(tty_file);
            let _ = execute!(
                tty,
                cursor::MoveTo(0, terminal_rows.saturating_sub(1)),
                terminal::ScrollUp(scroll_needed),
                cursor::MoveTo(0, next_prompt_row)
            );
            let _ = tty.flush();
        } else {
            let _ = execute!(
                stderr(),
                cursor::MoveTo(0, terminal_rows.saturating_sub(1)),
                terminal::ScrollUp(scroll_needed),
                cursor::MoveTo(0, next_prompt_row)
            );
        }

        next_prompt_row
    }

    #[derive(Debug, Clone)]
    struct LayoutMetrics {
        columns: usize,
        rows_total: usize,
        use_grid: bool,
        column_widths: Vec<usize>,
    }

    #[derive(Debug, Clone)]
    struct MenuLayoutPlan {
        content_area: Rect,
        divider_area: Option<Rect>,
        scrollbar_area: Option<Rect>,
        visible_rows: usize,
        scrollbar_needed: bool,
        metrics: LayoutMetrics,
    }

    /// Builds the final menu layout with a two-pass calculation.
    ///
    /// Borderless mode can reserve a dedicated scrollbar column, but whether that
    /// column is needed depends on the row/column layout, which itself depends on
    /// the available width. We therefore do a provisional width probe first,
    /// decide whether a scrollbar column is needed, and then recompute the final
    /// layout using the true content width.
    fn build_menu_layout(
        list_area: Rect,
        show_border: bool,
        item_count: usize,
        labels: &[String],
        item_max_len: Option<usize>,
    ) -> MenuLayoutPlan {
        let provisional_content_area = menu_content_area(list_area, show_border);
        let visible_rows = if show_border {
            provisional_content_area.height.saturating_sub(2) as usize
        } else {
            provisional_content_area.height as usize
        };
        let provisional_inner_width = if show_border {
            provisional_content_area.width.saturating_sub(2) as usize
        } else {
            provisional_content_area.width as usize
        };
        let width_probe_metrics =
            compute_layout_metrics(provisional_inner_width, item_count, labels, item_max_len);
        let scrollbar_probe_needed = if width_probe_metrics.use_grid {
            width_probe_metrics.rows_total > visible_rows && visible_rows > 0
        } else {
            item_count > visible_rows && visible_rows > 0
        };
        let (content_area, scrollbar_area) = split_menu_areas(
            provisional_content_area,
            show_border,
            scrollbar_probe_needed,
        );
        let final_inner_width = if show_border {
            content_area.width.saturating_sub(2) as usize
        } else {
            content_area.width as usize
        };
        let final_metrics =
            compute_layout_metrics(final_inner_width, item_count, labels, item_max_len);
        let scrollbar_needed = if final_metrics.use_grid {
            final_metrics.rows_total > visible_rows && visible_rows > 0
        } else {
            item_count > visible_rows && visible_rows > 0
        };

        MenuLayoutPlan {
            content_area,
            divider_area: menu_divider_area(list_area, show_border),
            scrollbar_area,
            visible_rows,
            scrollbar_needed,
            metrics: final_metrics,
        }
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

    fn menu_content_area(list_area: Rect, show_border: bool) -> Rect {
        if show_border {
            list_area
        } else {
            Rect {
                x: list_area.x,
                y: list_area.y,
                width: list_area.width,
                height: list_area.height.saturating_sub(1),
            }
        }
    }

    fn menu_divider_area(list_area: Rect, show_border: bool) -> Option<Rect> {
        if show_border || list_area.height == 0 {
            None
        } else {
            Some(Rect {
                x: list_area.x,
                y: list_area.y + list_area.height.saturating_sub(1),
                width: list_area.width,
                height: 1,
            })
        }
    }

    fn build_scrollbar(show_border: bool) -> Scrollbar<'static> {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        if show_border {
            scrollbar
        } else {
            scrollbar
                .track_symbol(Some("│"))
                .thumb_symbol("┃")
                .track_style(
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                )
                .thumb_style(Style::default().fg(Color::Gray).add_modifier(Modifier::DIM))
        }
    }

    fn menu_scrollbar_render_area(
        content_area: Rect,
        scrollbar_area: Option<Rect>,
        show_border: bool,
    ) -> Rect {
        if show_border {
            Rect {
                x: content_area.x,
                y: content_area.y + 1,
                width: content_area.width,
                height: content_area.height.saturating_sub(2),
            }
        } else {
            scrollbar_area.expect("borderless scrollbar area expected")
        }
    }

    fn split_menu_areas(
        content_area: Rect,
        show_border: bool,
        scrollbar_needed: bool,
    ) -> (Rect, Option<Rect>) {
        if show_border || !scrollbar_needed || content_area.width <= 1 {
            (content_area, None)
        } else {
            (
                Rect {
                    x: content_area.x,
                    y: content_area.y,
                    width: content_area.width.saturating_sub(1),
                    height: content_area.height,
                },
                Some(Rect {
                    x: content_area.x + content_area.width.saturating_sub(1),
                    y: content_area.y,
                    width: 1,
                    height: content_area.height,
                }),
            )
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

    fn text_width(input: &str) -> usize {
        input.chars().count()
    }

    fn build_status_text(
        width: u16,
        selected_path: &str,
        overflow: &str,
        typed_refinement: &str,
    ) -> String {
        let width = width as usize;
        if width == 0 {
            return String::new();
        }

        let refinement = if typed_refinement.is_empty() {
            None
        } else {
            Some(format!("/{typed_refinement}"))
        };
        let selected_with_overflow = format!("{selected_path}{overflow}");

        if refinement.is_none() {
            if text_width(&selected_with_overflow) <= width {
                return selected_with_overflow;
            }
            return truncate_for_cell(selected_path, width);
        }

        let refinement = refinement.unwrap();
        if let Some(text) = join_status_parts(width, &selected_with_overflow, &refinement) {
            return text;
        }

        if let Some(text) = join_status_parts(width, selected_path, &refinement) {
            return text;
        }

        let min_selection_width = width.min(12);
        let min_refinement_width = 4;
        if width >= min_selection_width + 1 + min_refinement_width {
            let max_refinement_width = (width - min_selection_width - 1)
                .min(refinement_cap(width, &refinement))
                .max(min_refinement_width);
            let refinement = truncate_for_cell(&refinement, max_refinement_width);
            let selected_width = width - text_width(&refinement) - 1;
            let selected = truncate_for_cell(selected_path, selected_width);
            let gap = width - text_width(&selected) - text_width(&refinement);
            return format!("{selected}{}{refinement}", " ".repeat(gap));
        }

        truncate_for_cell(selected_path, width)
    }

    fn refinement_cap(width: usize, refinement: &str) -> usize {
        let natural = text_width(refinement);
        let cap = (width / 3).max(4).min(32);
        natural.min(cap)
    }

    fn join_status_parts(width: usize, left: &str, right: &str) -> Option<String> {
        let left_width = text_width(left);
        let right_width = refinement_cap(width, right);
        if width < left_width + 1 + right_width {
            return None;
        }

        let right = truncate_for_cell(right, right_width);
        let gap = width - left_width - text_width(&right);
        Some(format!("{left}{}{right}", " ".repeat(gap)))
    }

    fn pad_to_width(input: &str, width: usize) -> String {
        let mut out = input.to_string();
        let len = out.chars().count();
        if width > len {
            out.push_str(&" ".repeat(width - len));
        }
        out
    }

    fn overflow_note(displayed: usize, has_more: bool) -> String {
        if has_more {
            format!(" | showing first {displayed}")
        } else {
            String::new()
        }
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
                (col + 1) % columns
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
        use std::cell::RefCell;

        #[test]
        fn display_label_relative_under_cwd() {
            let cwd = Path::new("/Users/nick");
            let path = Path::new("/Users/nick/Desktop");
            assert_eq!(display_label(path, cwd, None, true), "./Desktop");
        }

        #[test]
        fn display_label_tilde_when_under_home_but_not_cwd() {
            let cwd = Path::new("/tmp");
            let home = Path::new("/Users/nick");
            let path = Path::new("/Users/nick/code/dx");
            assert_eq!(display_label(path, cwd, Some(home), true), "~/code/dx");
        }

        #[test]
        fn display_label_absolute_when_outside_home() {
            let cwd = Path::new("/tmp");
            let home = Path::new("/Users/nick");
            let path = Path::new("/opt/homebrew/bin");
            assert_eq!(
                display_label(path, cwd, Some(home), true),
                "/opt/homebrew/bin"
            );
        }

        #[test]
        fn display_label_cwd_itself_shows_dot() {
            let cwd = Path::new("/Users/nick");
            let path = Path::new("/Users/nick");
            assert_eq!(display_label(path, cwd, None, true), "./");
        }

        #[test]
        fn display_label_paths_mode_relative_under_cwd_uses_dot_slash() {
            let cwd = Path::new("/tmp/work");
            let path = Path::new("/tmp/work/./benches");
            assert_eq!(display_label(path, cwd, None, true), "./benches");
        }

        #[test]
        fn display_label_paths_mode_parent_relative_prefix_is_preserved() {
            let cwd = Path::new("/tmp/work");
            let path = Path::new("/tmp/work/../sibling");
            assert_eq!(display_label(path, cwd, None, true), "../sibling");
        }

        #[test]
        fn display_label_paths_mode_multi_parent_relative_prefix_is_preserved() {
            let cwd = Path::new("/tmp/work");
            let path = Path::new("/tmp/work/../../outer");
            assert_eq!(display_label(path, cwd, None, true), "../../outer");
        }

        #[test]
        fn display_label_explicit_absolute_mode_preserves_absolute_path() {
            let cwd = Path::new("/tmp/work");
            let path = Path::new("/tmp/work/./benches");
            assert_eq!(display_label(path, cwd, None, false), "/tmp/work/./benches");
        }

        #[test]
        fn borderless_menu_content_area_reserves_one_separator_row() {
            let list_area = Rect::new(0, 0, 80, 12);
            let content = menu_content_area(list_area, false);
            assert_eq!(content.height, 11);
        }

        #[test]
        fn bordered_menu_content_area_uses_full_list_area() {
            let list_area = Rect::new(0, 0, 80, 12);
            let content = menu_content_area(list_area, true);
            assert_eq!(content, list_area);
        }

        #[test]
        fn borderless_menu_divider_area_uses_last_row() {
            let list_area = Rect::new(3, 4, 80, 12);
            let divider = menu_divider_area(list_area, false).expect("divider expected");
            assert_eq!(divider, Rect::new(3, 15, 80, 1));
        }

        #[test]
        fn bordered_menu_has_no_divider_area() {
            let list_area = Rect::new(0, 0, 80, 12);
            assert_eq!(menu_divider_area(list_area, true), None);
        }

        #[test]
        fn borderless_scrollbar_reserves_rightmost_column() {
            let content_area = Rect::new(2, 3, 20, 8);
            let (content, scrollbar) = split_menu_areas(content_area, false, true);
            assert_eq!(content, Rect::new(2, 3, 19, 8));
            assert_eq!(scrollbar, Some(Rect::new(21, 3, 1, 8)));
        }

        #[test]
        fn bordered_scrollbar_uses_full_content_area() {
            let content_area = Rect::new(2, 3, 20, 8);
            let (content, scrollbar) = split_menu_areas(content_area, true, true);
            assert_eq!(content, content_area);
            assert_eq!(scrollbar, None);
        }

        #[test]
        fn borderless_scrollbar_render_area_uses_reserved_column() {
            let area = menu_scrollbar_render_area(
                Rect::new(2, 3, 19, 8),
                Some(Rect::new(21, 3, 1, 8)),
                false,
            );
            assert_eq!(area, Rect::new(21, 3, 1, 8));
        }

        #[test]
        fn bordered_scrollbar_render_area_uses_inner_height() {
            let area = menu_scrollbar_render_area(Rect::new(2, 3, 20, 8), None, true);
            assert_eq!(area, Rect::new(2, 4, 20, 6));
        }

        #[test]
        fn build_menu_layout_reserves_scrollbar_column_when_needed() {
            let labels = vec!["one".to_string(); 50];
            let layout = build_menu_layout(Rect::new(0, 0, 20, 8), false, 50, &labels, Some(8));
            assert!(layout.scrollbar_needed);
            assert_eq!(layout.scrollbar_area, Some(Rect::new(19, 0, 1, 7)));
            assert_eq!(layout.content_area, Rect::new(0, 0, 19, 7));
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
            let labels = vec![
                "abcdefgh".to_string(),
                "beta".to_string(),
                "gamma".to_string(),
            ];
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

        #[test]
        fn overflow_note_reports_hidden_results() {
            assert_eq!(overflow_note(1000, true), " | showing first 1000");
            assert_eq!(overflow_note(10, false), "");
            assert_eq!(overflow_note(0, false), "");
        }

        #[test]
        fn status_text_shows_selection_without_refinement() {
            assert_eq!(build_status_text(20, "./Downloads", "", ""), "./Downloads");
        }

        #[test]
        fn status_text_right_aligns_typed_refinement_only() {
            let status = build_status_text(20, "./Documents", "", "w");

            assert!(status.starts_with("./Documents"));
            assert!(status.ends_with("/w"));
            assert_eq!(text_width(&status), 20);
            assert!(!status.contains("/Dow"));
            assert!(!status.contains("filter:"));
        }

        #[test]
        fn status_text_places_overflow_between_selection_and_refinement() {
            let status = build_status_text(45, "./Downloads", " | showing first 1000", "w");

            assert!(status.starts_with("./Downloads | showing first 1000"));
            assert!(status.ends_with("/w"));
            assert!(status.find("showing first").unwrap() < status.find("/w").unwrap());
            assert_eq!(text_width(&status), 45);
        }

        #[test]
        fn status_text_drops_overflow_before_refinement() {
            let status = build_status_text(20, "./Downloads", " | showing first 1000", "w");

            assert!(status.starts_with("./Downloads"));
            assert!(status.ends_with("/w"));
            assert!(!status.contains("showing first"));
            assert_eq!(text_width(&status), 20);
        }

        #[test]
        fn status_text_truncates_long_selection_but_keeps_refinement() {
            let status = build_status_text(24, "./very/deep/path/to/tui.rs", "", "abc");

            assert!(status.starts_with('…'));
            assert!(status.contains("path/to/tui.rs"));
            assert!(status.ends_with("/abc"));
            assert_eq!(text_width(&status), 24);
        }

        #[test]
        fn status_text_caps_long_refinement_to_preserve_selection() {
            let status = build_status_text(30, "./selected/path", "", "ridiculously-long-filter");

            assert!(status.starts_with("./selected/path"));
            assert!(status.ends_with("…ng-filter"));
            assert_eq!(text_width(&status), 30);
        }

        #[test]
        fn status_text_hides_refinement_when_terminal_is_tiny() {
            let status = build_status_text(12, "selected-path", "", "abcdef");

            assert_eq!(status, "…lected-path");
            assert!(!status.contains('/'));
        }

        #[test]
        fn compute_list_rows_honors_cap_and_minimum() {
            assert_eq!(compute_list_rows(50, 10), 10);
            assert_eq!(compute_list_rows(3, 10), 3);
            assert_eq!(compute_list_rows(0, 10), 1);
            assert_eq!(compute_list_rows(5, 0), 1);
        }

        #[test]
        fn compute_rendered_height_keeps_minimal_no_match_floor() {
            assert_eq!(compute_rendered_height(80, 0, &[], 10, None, false), 3);
            assert_eq!(compute_rendered_height(80, 0, &[], 10, None, true), 4);
        }

        #[test]
        fn compute_rendered_height_shrinks_with_filtered_single_column_results() {
            let labels = vec!["alpha".to_string(); 8];
            let smaller = vec!["alpha".to_string(); 1];

            let tall = compute_rendered_height(80, labels.len(), &labels, 10, None, false);
            let short = compute_rendered_height(80, smaller.len(), &smaller, 10, None, false);

            assert!(short < tall);
            assert_eq!(short, 3);
        }

        #[test]
        fn compute_rendered_height_shrinks_with_filtered_bordered_single_column_results() {
            let labels = vec!["alpha".to_string(); 8];
            let smaller = vec!["alpha".to_string(); 1];

            let tall = compute_rendered_height(20, labels.len(), &labels, 10, None, true);
            let short = compute_rendered_height(20, smaller.len(), &smaller, 10, None, true);

            assert!(short < tall);
            assert_eq!(short, 4);
        }

        #[test]
        fn compute_rendered_height_shrinks_with_filtered_multicolumn_results() {
            let many = vec!["abcdefgh".to_string(); 9];
            let few = vec!["abcdefgh".to_string(); 3];

            let tall = compute_rendered_height(36, many.len(), &many, 10, Some(8), true);
            let short = compute_rendered_height(36, few.len(), &few, 10, Some(8), true);

            assert!(short < tall);
            assert_eq!(short, 4);
        }

        #[test]
        fn compute_rendered_height_shrinks_with_filtered_borderless_multicolumn_results() {
            let many = vec!["abcdefgh".to_string(); 9];
            let few = vec!["abcdefgh".to_string(); 3];

            let tall = compute_rendered_height(36, many.len(), &many, 10, Some(8), false);
            let short = compute_rendered_height(36, few.len(), &few, 10, Some(8), false);

            assert!(short < tall);
            assert_eq!(short, 3);
        }

        #[test]
        fn bounded_rendered_height_stays_within_reserved_height() {
            assert_eq!(bounded_rendered_height(8, 5), 5);
            assert_eq!(bounded_rendered_height(8, 8), 8);
            assert_eq!(bounded_rendered_height(8, 10), 8);
        }

        #[test]
        fn bounded_rendered_height_allows_reexpansion_back_to_reserved_height() {
            let reserved_height = 8;

            let shrunk_height = bounded_rendered_height(reserved_height, 3);
            let reexpanded_height = bounded_rendered_height(reserved_height, reserved_height);

            assert_eq!(shrunk_height, 3);
            assert_eq!(reexpanded_height, reserved_height);
        }

        #[test]
        fn rendered_menu_area_uses_current_height_within_reserved_area() {
            let reserved = Rect::new(0, 5, 80, 10);
            assert_eq!(rendered_menu_area(reserved, 4), Rect::new(0, 5, 80, 4));
            assert_eq!(rendered_menu_area(reserved, 12), reserved);
        }

        #[test]
        fn cleared_trailing_area_returns_vacated_rows_on_shrink() {
            let reserved = Rect::new(0, 5, 80, 10);
            assert_eq!(
                cleared_trailing_area(reserved, 8, 3),
                Some(Rect::new(0, 8, 80, 5))
            );
            assert_eq!(cleared_trailing_area(reserved, 3, 3), None);
            assert_eq!(cleared_trailing_area(reserved, 3, 5), None);
        }

        #[test]
        fn borderless_menu_adds_prompt_gap_row() {
            assert_eq!(prompt_gap_rows(true), 0);
            assert_eq!(prompt_gap_rows(false), 1);
            assert_eq!(required_rows_below(12, true), 12);
            assert_eq!(required_rows_below(12, false), 13);
        }

        #[test]
        fn menu_top_row_leaves_blank_line_when_borderless() {
            assert_eq!(menu_top_row(5, 24, 10, true), 6);
            assert_eq!(menu_top_row(5, 24, 10, false), 7);
        }

        #[test]
        fn scroll_rows_needed_only_when_not_enough_rows_below() {
            assert_eq!(scroll_rows_needed(5, 24, 10), 0);
            assert_eq!(scroll_rows_needed(20, 24, 10), 7);
            assert_eq!(scroll_rows_needed(23, 24, 2), 2);
        }

        #[test]
        fn key_event_mapping_j_is_filter_input_not_navigation() {
            let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
            assert_eq!(map_key_event(key, false), MenuKeyAction::InputChar('j'));
        }

        #[test]
        fn key_event_mapping_k_is_filter_input_not_navigation() {
            let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
            assert_eq!(map_key_event(key, false), MenuKeyAction::InputChar('k'));
        }

        #[test]
        fn key_event_mapping_arrows_remain_navigation() {
            let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
            let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
            assert_eq!(map_key_event(down, false), MenuKeyAction::MoveLinear(1));
            assert_eq!(map_key_event(up, false), MenuKeyAction::MoveLinear(-1));
        }

        #[test]
        fn key_event_mapping_tab_and_backtab_remain_navigation() {
            let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
            let backtab = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
            assert_eq!(map_key_event(tab, false), MenuKeyAction::MoveLinear(1));
            assert_eq!(map_key_event(backtab, false), MenuKeyAction::MoveLinear(-1));
        }

        #[test]
        fn filter_state_clamps_backspace_to_initial_query() {
            let mut filter = FilterState::new("Do");
            assert!(!filter.backspace());
            assert_eq!(filter.effective_query(), "Do");

            filter.push('w');
            assert_eq!(filter.effective_query(), "Dow");
            assert!(filter.backspace());
            assert_eq!(filter.effective_query(), "Do");
            assert!(!filter.changed_query());
        }

        #[test]
        fn filter_state_empty_seed_can_return_to_empty() {
            let mut filter = FilterState::new("");
            filter.push('a');
            assert_eq!(filter.effective_query(), "a");
            assert!(filter.changed_query());

            assert!(filter.backspace());
            assert_eq!(filter.effective_query(), "");
            assert!(!filter.changed_query());
            assert!(!filter.backspace());
        }

        #[test]
        fn cancel_result_is_noop_after_net_zero_edits() {
            let mut filter = FilterState::new("Do");
            filter.push('w');
            assert!(filter.backspace());

            assert_eq!(
                cancelled_result(&filter),
                MenuResult::Cancelled {
                    filter_query: "Do".to_string(),
                    changed_query: false,
                }
            );
        }

        #[test]
        fn apply_filter_edit_backspace_at_seed_does_not_requery() {
            let calls = RefCell::new(Vec::new());
            let query_fn: QueryFn<'_> = Box::new(|query| {
                calls.borrow_mut().push(query.to_string());
                CompletionCandidates {
                    paths: Vec::new(),
                    has_more: false,
                }
            });
            let mut filter = FilterState::new("Do");
            let mut completion = CompletionCandidates {
                paths: vec![PathBuf::from("/tmp/Do")],
                has_more: false,
            };
            let mut list_state = ListState::default();
            list_state.select(Some(0));

            apply_filter_edit(
                &mut filter,
                &mut completion,
                &mut list_state,
                MenuKeyAction::Backspace,
                &query_fn,
            );

            assert!(calls.borrow().is_empty());
            assert_eq!(filter.effective_query(), "Do");
            assert_eq!(completion.paths, vec![PathBuf::from("/tmp/Do")]);
            assert_eq!(list_state.selected(), Some(0));
        }

        #[test]
        fn apply_filter_edit_requeries_with_seed_plus_typed_refinement() {
            let calls = RefCell::new(Vec::new());
            let query_fn: QueryFn<'_> = Box::new(|query| {
                calls.borrow_mut().push(query.to_string());
                CompletionCandidates {
                    paths: if query == "Doz" {
                        Vec::new()
                    } else {
                        vec![PathBuf::from(format!("/tmp/{query}"))]
                    },
                    has_more: false,
                }
            });
            let mut filter = FilterState::new("Do");
            let mut completion = CompletionCandidates {
                paths: vec![PathBuf::from("/tmp/Do")],
                has_more: false,
            };
            let mut list_state = ListState::default();
            list_state.select(Some(0));

            apply_filter_edit(
                &mut filter,
                &mut completion,
                &mut list_state,
                MenuKeyAction::InputChar('z'),
                &query_fn,
            );

            assert_eq!(calls.borrow().as_slice(), ["Doz"]);
            assert_eq!(filter.effective_query(), "Doz");
            assert!(completion.paths.is_empty());
            assert_eq!(list_state.selected(), None);
        }

        #[test]
        fn selected_label_uses_precomputed_labels_and_falls_back_for_empty_selection() {
            let labels = vec!["./alpha".to_string(), "./beta".to_string()];

            assert_eq!(selected_label(&labels, Some(1)), "./beta");
            assert_eq!(selected_label(&labels, None), "(no matches)");
            assert_eq!(selected_label(&labels, Some(99)), "(no matches)");
        }
    }
}

#[cfg(not(unix))]
mod imp {
    use super::MenuResult;
    use std::path::Path;

    use crate::resolve::CompletionCandidates;

    pub type QueryFn<'a> = Box<dyn Fn(&str) -> CompletionCandidates + 'a>;

    pub fn select(
        _candidates: CompletionCandidates,
        initial_query: &str,
        _cwd: &Path,
        _prefer_relative_paths: bool,
        _prompt_row_override: Option<u16>,
        _max_rows: u16,
        _item_max_len: Option<usize>,
        _show_border: bool,
        _psreadline_mode: bool,
        _query_fn: QueryFn<'_>,
    ) -> Option<MenuResult> {
        Some(MenuResult::Cancelled {
            filter_query: initial_query.to_string(),
            changed_query: false,
        })
    }
}

pub use imp::{QueryFn, select};
