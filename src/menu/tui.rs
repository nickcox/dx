//! Interactive TUI selection for `dx menu`.
//!
//! Renders an inline list immediately below the prompt line.
//! stdout stays free for JSON output; the TUI is drawn to stderr.
//! crossterm is built with `use-dev-tty` so `event::read()` reads from
//! `/dev/tty` directly, working even when stdin is redirected by a shell
//! completion hook.

use std::path::Path;

use crate::menu::MenuMode;
use crate::menu::ls_colors::LsColorsConfig;
use crate::resolve::CompletionCandidates;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuResult {
    Selected {
        filter_query: String,
        changed_query: bool,
        value: std::path::PathBuf,
        terminal: crate::menu::action::TerminalState,
        geometry: Option<crate::menu::action::TerminalGeometry>,
    },
    Cancelled {
        filter_query: String,
        changed_query: bool,
        geometry: Option<crate::menu::action::TerminalGeometry>,
    },
}

/// Re-query callback: given a query string, returns fresh candidates.
pub type QueryFn<'a> = Box<dyn Fn(&str) -> CompletionCandidates + 'a>;

/// What a single menu session should show. Fixed for as long as the menu is
/// open — only the typed refinement varies, and that is held internally.
pub struct MenuRequest<'a> {
    pub candidates: CompletionCandidates,
    pub query: &'a str,
    pub mode: MenuMode,
    pub cwd: &'a Path,
    /// Prompt row supplied by shells that can report it. Measured from the
    /// terminal when absent.
    pub prompt_row: Option<u16>,
    pub query_fn: QueryFn<'a>,
}

/// Presentation settings, all sourced from `DX_MENU_*` environment variables.
#[derive(Debug, Clone, Default)]
pub struct MenuOptions {
    pub max_rows: u16,
    pub item_max_len: Option<usize>,
    pub show_border: bool,
    /// Draw to `/dev/tty` rather than stderr, as PSReadLine requires.
    pub use_tty_backend: bool,
    pub ls_colors: Option<LsColorsConfig>,
}

// The interactive menu TUI currently targets Unix TTY semantics (`/dev/tty`
// and explicit terminal scrolling). Non-Unix builds use the stub
// implementation below, which preserves the JSON/noop contract for shell
// fallback paths without enabling the inline TUI yet.
#[cfg(unix)]
mod imp {
    use std::fs::OpenOptions;
    use std::io::{BufWriter, Read, Write, stderr};
    use std::os::fd::AsFd;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    use crossterm::{
        cursor,
        event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
        execute, terminal,
    };
    use nix::sys::select::{FdSet, select as select_fds};
    use nix::sys::time::{TimeVal, TimeValLike};
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

    use crate::menu::QueryStyle;

    type CandidateLabelStyle = QueryStyle;
    use crate::menu::ls_colors::LsColorsConfig;
    use crate::resolve::CompletionCandidates;

    use super::{MenuOptions, MenuRequest, MenuResult, QueryFn};

    #[derive(Clone, Copy)]
    struct CleanupRegion {
        prompt_row: u16,
        area: Rect,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct SpaceReservation {
        prompt_row: u16,
        scroll_rows: u16,
    }

    trait TerminalOps {
        fn size(&self) -> std::io::Result<(u16, u16)>;
        fn cursor_row(&self) -> std::io::Result<u16>;
        fn enable_raw_mode(&self) -> std::io::Result<()>;
        fn disable_raw_mode(&self) -> std::io::Result<()>;
        fn open_output(&self, use_tty_backend: bool) -> std::io::Result<Box<dyn Write>>;
    }

    struct CrosstermTerminalOps;

    impl TerminalOps for CrosstermTerminalOps {
        fn size(&self) -> std::io::Result<(u16, u16)> {
            terminal::size()
        }

        fn cursor_row(&self) -> std::io::Result<u16> {
            cursor_row_via_tty(Duration::from_millis(250))
        }

        fn enable_raw_mode(&self) -> std::io::Result<()> {
            terminal::enable_raw_mode()
        }

        fn disable_raw_mode(&self) -> std::io::Result<()> {
            terminal::disable_raw_mode()
        }

        fn open_output(&self, use_tty_backend: bool) -> std::io::Result<Box<dyn Write>> {
            if use_tty_backend {
                Ok(Box::new(BufWriter::new(
                    OpenOptions::new().write(true).open("/dev/tty")?,
                )))
            } else {
                Ok(Box::new(stderr()))
            }
        }
    }

    fn cursor_row_via_tty(timeout: Duration) -> std::io::Result<u16> {
        let mut tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
        tty.write_all(b"\x1b[6n")?;
        tty.flush()?;

        let deadline = Instant::now() + timeout;
        let mut response = Vec::with_capacity(16);
        while response.len() < 32 {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }

            let mut read_fds = FdSet::new();
            read_fds.insert(tty.as_fd());
            let timeout_ms = i64::try_from(remaining.as_millis().max(1)).unwrap_or(i64::MAX);
            let mut select_timeout = TimeVal::milliseconds(timeout_ms);
            if select_fds(
                None,
                Some(&mut read_fds),
                None,
                None,
                Some(&mut select_timeout),
            )
            .map_err(std::io::Error::other)?
                == 0
            {
                break;
            }

            let mut byte = [0u8; 1];
            tty.read_exact(&mut byte)?;
            response.push(byte[0]);
            if byte[0] == b'R' {
                return parse_cursor_row_response(&response).ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "terminal returned an invalid cursor position",
                    )
                });
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "terminal did not report its cursor position",
        ))
    }

    fn parse_cursor_row_response(response: &[u8]) -> Option<u16> {
        let response = std::str::from_utf8(response).ok()?;
        let position = response.rsplit_once("\x1b[")?.1.strip_suffix('R')?;
        let (row, _column) = position.split_once(';')?;
        row.parse::<u16>().ok()?.checked_sub(1)
    }

    struct TerminalSession<'a> {
        terminal_ops: &'a dyn TerminalOps,
        use_tty_backend: bool,
        output: Option<Box<dyn Write>>,
        cursor_hidden: bool,
        reservation: Option<SpaceReservation>,
        cleanup_region: Option<CleanupRegion>,
    }

    impl<'a> TerminalSession<'a> {
        fn start(
            terminal_ops: &'a dyn TerminalOps,
            use_tty_backend: bool,
        ) -> std::io::Result<Self> {
            terminal_ops.enable_raw_mode()?;
            Ok(Self {
                terminal_ops,
                use_tty_backend,
                output: None,
                cursor_hidden: false,
                reservation: None,
                cleanup_region: None,
            })
        }

        fn output(&mut self) -> std::io::Result<&mut Box<dyn Write>> {
            if self.output.is_none() {
                self.output = Some(self.terminal_ops.open_output(self.use_tty_backend)?);
            }

            Ok(self.output.as_mut().expect("output initialized"))
        }

        fn reserve_space(
            &mut self,
            prompt_row: u16,
            terminal_rows: u16,
            needed_height: u16,
        ) -> std::io::Result<SpaceReservation> {
            let scroll_needed = scroll_rows_needed(prompt_row, terminal_rows, needed_height);
            if scroll_needed == 0 {
                return Ok(SpaceReservation {
                    prompt_row,
                    scroll_rows: 0,
                });
            }

            let reservation =
                scroll_terminal(self.output()?, prompt_row, terminal_rows, scroll_needed)?;
            self.reservation = Some(reservation);
            Ok(reservation)
        }

        fn set_cleanup_region(&mut self, prompt_row: u16, area: Rect) {
            self.cleanup_region = Some(CleanupRegion { prompt_row, area });
        }

        fn hide_cursor(&mut self) -> std::io::Result<()> {
            execute!(self.output()?, cursor::Hide)?;
            self.cursor_hidden = true;
            self.output()?.flush()?;
            Ok(())
        }
    }

    impl Drop for TerminalSession<'_> {
        fn drop(&mut self) {
            let has_acquired_output_state =
                self.cleanup_region.is_some() || self.reservation.is_some() || self.cursor_hidden;
            if has_acquired_output_state && let Some(output) = self.output.as_mut() {
                restore_terminal_output(
                    output,
                    self.cleanup_region,
                    self.reservation,
                    self.cursor_hidden,
                );
            }

            let _ = self.terminal_ops.disable_raw_mode();
        }
    }

    fn scroll_terminal<W: Write>(
        output: &mut W,
        prompt_row: u16,
        terminal_rows: u16,
        scroll_needed: u16,
    ) -> std::io::Result<SpaceReservation> {
        let next_prompt_row = prompt_row.saturating_sub(scroll_needed);
        execute!(
            output,
            cursor::MoveTo(0, terminal_rows.saturating_sub(1)),
            terminal::ScrollUp(scroll_needed),
            cursor::MoveTo(0, next_prompt_row)
        )?;
        output.flush()?;
        Ok(SpaceReservation {
            prompt_row: next_prompt_row,
            scroll_rows: scroll_needed,
        })
    }

    fn restore_terminal_output<W: Write>(
        output: &mut W,
        cleanup_region: Option<CleanupRegion>,
        reservation: Option<SpaceReservation>,
        cursor_hidden: bool,
    ) {
        if let Some(region) = cleanup_region {
            let _ = execute!(output, cursor::MoveTo(0, region.prompt_row));
            for row in region.prompt_row.saturating_add(1)..region.area.bottom() {
                let _ = execute!(
                    output,
                    cursor::MoveTo(0, row),
                    terminal::Clear(terminal::ClearType::CurrentLine)
                );
            }
            let _ = execute!(output, cursor::MoveTo(0, region.prompt_row));
        } else if let Some(reservation) = reservation {
            let _ = execute!(output, cursor::MoveTo(0, reservation.prompt_row));
        }

        if cursor_hidden {
            let _ = execute!(output, cursor::Show);
        }
        let _ = output.flush();
    }

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
        MovePage(isize),
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
            (KeyCode::PageDown, _) => MenuKeyAction::MovePage(1),
            (KeyCode::PageUp, _) => MenuKeyAction::MovePage(-1),
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

    fn selected_result(
        filter_state: &FilterState,
        value: PathBuf,
        geometry: crate::menu::action::TerminalGeometry,
    ) -> MenuResult {
        MenuResult::Selected {
            value,
            filter_query: filter_state.effective_query(),
            changed_query: filter_state.changed_query(),
            terminal: crate::menu::action::TerminalState::Dirty,
            geometry: Some(geometry),
        }
    }

    fn cancelled_result(
        filter_state: &FilterState,
        geometry: crate::menu::action::TerminalGeometry,
    ) -> MenuResult {
        MenuResult::Cancelled {
            filter_query: filter_state.effective_query(),
            changed_query: filter_state.changed_query(),
            geometry: Some(geometry),
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

    fn selected_status_path(paths: &[PathBuf], selected: Option<usize>) -> String {
        selected
            .and_then(|idx| paths.get(idx))
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "(no matches)".to_string())
    }

    fn display_label(
        path: &Path,
        cwd: &Path,
        home: Option<&Path>,
        prefer_relative_paths: bool,
    ) -> String {
        if prefer_relative_paths && let Some(rel) = relative_path_for_display(path, cwd) {
            use std::path::Component;

            if rel.as_os_str().is_empty() {
                "./".to_string()
            } else {
                let starts_with_parent = rel
                    .components()
                    .next()
                    .is_some_and(|component| matches!(component, Component::ParentDir));
                if starts_with_parent {
                    rel.display().to_string()
                } else {
                    format!("./{}", rel.display())
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

    fn relative_path_for_display(path: &Path, cwd: &Path) -> Option<PathBuf> {
        path.strip_prefix(cwd)
            .ok()
            .map(crate::complete::sanitize_relative_components)
            .or_else(|| {
                let path = std::fs::canonicalize(path).ok()?;
                let cwd = std::fs::canonicalize(cwd).ok()?;
                path.strip_prefix(cwd)
                    .ok()
                    .map(crate::complete::sanitize_relative_components)
            })
    }

    fn cwd_relative_label_for_display(path: &Path, cwd: &Path, dot_prefix: bool) -> Option<String> {
        crate::complete::cwd_relative_label(path, cwd, dot_prefix).or_else(|| {
            let path = std::fs::canonicalize(path).ok()?;
            let cwd = std::fs::canonicalize(cwd).ok()?;
            crate::complete::cwd_relative_label(&path, &cwd, dot_prefix)
        })
    }

    fn display_label_for_style(
        path: &Path,
        cwd: &Path,
        home: Option<&Path>,
        style: QueryStyle,
    ) -> String {
        match style {
            QueryStyle::Compact => display_label(path, cwd, home, true),
            QueryStyle::BareRelative => cwd_relative_label_for_display(path, cwd, false)
                .unwrap_or_else(|| display_label(path, cwd, home, false)),
            QueryStyle::DotRelative => cwd_relative_label_for_display(path, cwd, true)
                .unwrap_or_else(|| display_label(path, cwd, home, false)),
            QueryStyle::ParentRelative => crate::complete::parent_relative_path_from(cwd, path)
                .map(|relative| relative.display().to_string())
                .unwrap_or_else(|| display_label(path, cwd, home, false)),
            QueryStyle::HomeRelative => crate::complete::home_relative_label(path, home)
                .unwrap_or_else(|| display_label(path, cwd, home, false)),
            QueryStyle::Absolute => path.display().to_string(),
        }
    }

    pub fn select(request: MenuRequest<'_>, options: &MenuOptions) -> Option<MenuResult> {
        select_with_terminal_ops(request, options, &CrosstermTerminalOps)
    }

    fn select_with_terminal_ops(
        request: MenuRequest<'_>,
        options: &MenuOptions,
        terminal_ops: &dyn TerminalOps,
    ) -> Option<MenuResult> {
        if request.candidates.paths.is_empty() {
            return Some(MenuResult::Cancelled {
                filter_query: request.query.to_string(),
                changed_query: false,
                geometry: None,
            });
        }

        if request.candidates.paths.len() == 1 && !request.candidates.has_more {
            return Some(MenuResult::Selected {
                value: request.candidates.paths.into_iter().next()?,
                filter_query: request.query.to_string(),
                changed_query: false,
                terminal: crate::menu::action::TerminalState::Clean,
                geometry: None,
            });
        }

        let (cols, rows) = terminal_ops.size().ok()?;

        let home = dirs::home_dir();
        let initial_label_style = QueryStyle::from_query(request.mode, request.query);
        let initial_labels: Vec<String> = request
            .candidates
            .paths
            .iter()
            .map(|p| display_label_for_style(p, request.cwd, home.as_deref(), initial_label_style))
            .collect();
        let height = compute_rendered_height(
            cols,
            request.candidates.paths.len(),
            &initial_labels,
            options,
        );
        let required_rows = required_rows_below(height, options.show_border);
        let mut session = TerminalSession::start(terminal_ops, options.use_tty_backend).ok()?;

        let measured_prompt_row = match request.prompt_row {
            Some(row) => row,
            None => terminal_ops.cursor_row().ok()?,
        };
        let prompt_row = clamp_prompt_row(measured_prompt_row, rows);
        let reservation = session
            .reserve_space(prompt_row, rows, required_rows)
            .ok()?;
        let prompt_row = reservation.prompt_row;

        let menu_top = menu_top_row(prompt_row, rows, height, options.show_border);
        let area = Rect::new(0, menu_top, cols, height);
        session.hide_cursor().ok()?;
        session.set_cleanup_region(prompt_row, area);

        run_loop(
            request,
            options,
            area,
            terminal_ops,
            crate::menu::action::TerminalGeometry {
                redraw_row: reservation.prompt_row,
                scroll_rows: reservation.scroll_rows,
            },
        )
    }

    fn render_grid_items(
        completion: &CompletionCandidates,
        labels: &[String],
        ls_colors: Option<&LsColorsConfig>,
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
                let span = candidate_span(text, &completion.paths[idx], idx == selected, ls_colors);
                spans.push(span);
            }
            lines.push(Line::from(spans));
        }

        lines
    }

    fn selected_style() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    fn candidate_span(
        text: String,
        path: &Path,
        selected: bool,
        ls_colors: Option<&LsColorsConfig>,
    ) -> Span<'static> {
        if selected {
            return Span::styled(text, selected_style());
        }

        if let Some(style) = ls_colors.and_then(|lc| lc.style_for_path(path)) {
            Span::styled(text, style)
        } else {
            Span::raw(text)
        }
    }

    fn render_grid(
        frame: &mut ratatui::Frame<'_>,
        completion: &CompletionCandidates,
        labels: &[String],
        options: &MenuOptions,
        layout: &MenuLayoutPlan,
        selected: usize,
    ) {
        let show_border = options.show_border;
        let lines = render_grid_items(
            completion,
            labels,
            options.ls_colors.as_ref(),
            layout,
            selected,
        );
        let mut grid = Paragraph::new(lines);
        if show_border {
            grid = grid.block(Block::bordered());
        }
        frame.render_widget(grid, layout.content_area);

        if layout.scrollbar_needed {
            let selected_row = selected / layout.metrics.columns;
            let mut scrollbar_state =
                ScrollbarState::new(layout.metrics.rows_total).position(selected_row);
            render_scrollbar(frame, layout, show_border, &mut scrollbar_state);
        }
    }

    fn render_list(
        frame: &mut ratatui::Frame<'_>,
        completion: &CompletionCandidates,
        labels: &[String],
        options: &MenuOptions,
        layout: &MenuLayoutPlan,
        list_state: &mut ListState,
    ) {
        let show_border = options.show_border;
        let items: Vec<ListItem> = labels
            .iter()
            .enumerate()
            .map(|(i, label)| {
                let selected = list_state.selected() == Some(i);
                let line = Line::from(candidate_span(
                    label.clone(),
                    &completion.paths[i],
                    selected,
                    options.ls_colors.as_ref(),
                ));
                ListItem::new(line)
            })
            .collect();

        let mut list = List::new(items)
            .highlight_style(selected_style())
            .highlight_symbol("▸ ");
        if show_border {
            list = list.block(Block::bordered());
        }

        frame.render_stateful_widget(list, layout.content_area, list_state);

        if layout.scrollbar_needed {
            let selected = list_state.selected().unwrap_or(0);
            let mut scrollbar_state = ScrollbarState::new(labels.len()).position(selected);
            render_scrollbar(frame, layout, show_border, &mut scrollbar_state);
        }
    }

    fn render_scrollbar(
        frame: &mut ratatui::Frame<'_>,
        layout: &MenuLayoutPlan,
        show_border: bool,
        state: &mut ScrollbarState,
    ) {
        let area =
            menu_scrollbar_render_area(layout.content_area, layout.scrollbar_area, show_border);
        frame.render_stateful_widget(build_scrollbar(show_border), area, state);
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
        request: MenuRequest<'_>,
        options: &MenuOptions,
        area: Rect,
        terminal_ops: &dyn TerminalOps,
        geometry: crate::menu::action::TerminalGeometry,
    ) -> Option<MenuResult> {
        let MenuRequest {
            candidates: initial_candidates,
            query: initial_query,
            mode,
            cwd,
            query_fn,
            ..
        } = request;

        let writer = terminal_ops.open_output(options.use_tty_backend).ok()?;

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
            let effective_query = filter_state.effective_query();
            let label_style = CandidateLabelStyle::from_query(mode, &effective_query);
            let labels: Vec<String> = completion
                .paths
                .iter()
                .map(|p| display_label_for_style(p, cwd, home.as_deref(), label_style))
                .collect();
            let target_height =
                compute_rendered_height(area.width, completion.paths.len(), &labels, options);
            let current_height = bounded_rendered_height(area.height, target_height);
            let current_area = rendered_menu_area(area, current_height);
            let list_area = Rect {
                x: current_area.x,
                y: current_area.y,
                width: current_area.width,
                height: current_area.height.saturating_sub(1),
            };
            let layout = build_menu_layout(list_area, completion.paths.len(), &labels, options);

            let selected_path = selected_status_path(&completion.paths, list_state.selected());

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

                    if layout.metrics.use_grid {
                        render_grid(frame, &completion, &labels, options, &layout, selected);
                    } else {
                        render_list(
                            frame,
                            &completion,
                            &labels,
                            options,
                            &layout,
                            &mut list_state,
                        );
                    }

                    render_status(
                        frame,
                        &chunks,
                        layout.divider_area,
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
                            return Some(selected_result(&filter_state, value, geometry));
                        }
                    }
                    MenuKeyAction::Cancel => {
                        return Some(cancelled_result(&filter_state, geometry));
                    }
                    MenuKeyAction::MoveLinear(delta) => {
                        move_selection(&mut list_state, len, delta);
                    }
                    MenuKeyAction::MoveGridVertical(direction) => {
                        move_selection_grid_vertical(&mut list_state, len, columns, direction);
                    }
                    MenuKeyAction::MovePage(direction) => {
                        let page_size = page_selection_step(layout.visible_rows, columns, use_grid);
                        move_selection_page(&mut list_state, len, page_size, direction);
                    }
                    MenuKeyAction::Backspace | MenuKeyAction::InputChar(_) => apply_filter_edit(
                        &mut filter_state,
                        &mut completion,
                        &mut list_state,
                        action,
                        &query_fn,
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
        options: &MenuOptions,
    ) -> u16 {
        let frame_inset = if options.show_border { 2 } else { 0 };
        let metrics = compute_layout_metrics(
            width.saturating_sub(frame_inset) as usize,
            item_count,
            labels,
            options.item_max_len,
        );
        let list_rows = compute_list_rows(metrics.rows_total, options.max_rows);
        list_rows + menu_chrome_height(options.show_border)
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

    fn clamp_prompt_row(prompt_row: u16, terminal_rows: u16) -> u16 {
        prompt_row.min(terminal_rows.saturating_sub(1))
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
        item_count: usize,
        labels: &[String],
        options: &MenuOptions,
    ) -> MenuLayoutPlan {
        let show_border = options.show_border;
        let item_max_len = options.item_max_len;
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

        let Some(refinement) = refinement else {
            if text_width(&selected_with_overflow) <= width {
                return selected_with_overflow;
            }
            return truncate_for_cell(selected_path, width);
        };

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
        let cap = (width / 3).clamp(4, 32);
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

    fn page_selection_step(visible_rows: usize, columns: usize, use_grid: bool) -> usize {
        if use_grid {
            visible_rows.saturating_mul(columns).max(1)
        } else {
            visible_rows.max(1)
        }
    }

    fn move_selection_page(state: &mut ListState, len: usize, page_size: usize, direction: isize) {
        if len == 0 {
            state.select(None);
            return;
        }

        let current = state.selected().unwrap_or(0);
        let step = page_size.max(1);
        let next = if direction >= 0 {
            current.saturating_add(step).min(len - 1)
        } else {
            current.saturating_sub(step)
        };
        state.select(Some(next));
    }

    #[cfg(test)]
    mod tests {
        use std::cell::{Cell, RefCell};
        use std::fs;
        use std::os::unix::fs::symlink;
        use std::rc::Rc;

        use super::*;

        #[derive(Default)]
        struct WriterState {
            bytes: Vec<u8>,
            fail_write: bool,
            fail_flush: bool,
        }

        struct RecordingWriter {
            state: Rc<RefCell<WriterState>>,
        }

        impl Write for RecordingWriter {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                let mut state = self.state.borrow_mut();
                if state.fail_write {
                    return Err(std::io::Error::other("injected write failure"));
                }
                state.bytes.extend_from_slice(bytes);
                Ok(bytes.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                if self.state.borrow().fail_flush {
                    Err(std::io::Error::other("injected flush failure"))
                } else {
                    Ok(())
                }
            }
        }

        struct MockTerminalOps {
            size: (u16, u16),
            cursor_row: u16,
            fail_enable: bool,
            fail_cursor_row: bool,
            fail_open_at: Option<usize>,
            size_calls: Cell<usize>,
            cursor_row_calls: Cell<usize>,
            enable_calls: Cell<usize>,
            disable_calls: Cell<usize>,
            open_calls: Cell<usize>,
            writer_state: Rc<RefCell<WriterState>>,
        }

        impl MockTerminalOps {
            fn new() -> Self {
                Self {
                    size: (80, 24),
                    cursor_row: 23,
                    fail_enable: false,
                    fail_cursor_row: false,
                    fail_open_at: None,
                    size_calls: Cell::new(0),
                    cursor_row_calls: Cell::new(0),
                    enable_calls: Cell::new(0),
                    disable_calls: Cell::new(0),
                    open_calls: Cell::new(0),
                    writer_state: Rc::new(RefCell::new(WriterState::default())),
                }
            }

            fn output_contains(&self, sequence: &[u8]) -> bool {
                self.writer_state
                    .borrow()
                    .bytes
                    .windows(sequence.len())
                    .any(|bytes| bytes == sequence)
            }

            fn output_contains_scroll_up(&self) -> bool {
                let state = self.writer_state.borrow();
                let bytes = &state.bytes;
                bytes.iter().enumerate().any(|(index, byte)| {
                    *byte == b'S'
                        && bytes[..index]
                            .iter()
                            .rev()
                            .take_while(|byte| byte.is_ascii_digit())
                            .count()
                            > 0
                })
            }
        }

        impl TerminalOps for MockTerminalOps {
            fn size(&self) -> std::io::Result<(u16, u16)> {
                self.size_calls.set(self.size_calls.get() + 1);
                Ok(self.size)
            }

            fn cursor_row(&self) -> std::io::Result<u16> {
                self.cursor_row_calls.set(self.cursor_row_calls.get() + 1);
                if self.fail_cursor_row {
                    Err(std::io::Error::other("injected cursor row failure"))
                } else {
                    Ok(self.cursor_row)
                }
            }

            fn enable_raw_mode(&self) -> std::io::Result<()> {
                self.enable_calls.set(self.enable_calls.get() + 1);
                if self.fail_enable {
                    Err(std::io::Error::other("injected raw mode failure"))
                } else {
                    Ok(())
                }
            }

            fn disable_raw_mode(&self) -> std::io::Result<()> {
                self.disable_calls.set(self.disable_calls.get() + 1);
                Ok(())
            }

            fn open_output(&self, _use_tty_backend: bool) -> std::io::Result<Box<dyn Write>> {
                let call = self.open_calls.get() + 1;
                self.open_calls.set(call);
                if self.fail_open_at == Some(call) {
                    return Err(std::io::Error::other("injected output open failure"));
                }
                Ok(Box::new(RecordingWriter {
                    state: Rc::clone(&self.writer_state),
                }))
            }
        }

        fn candidates(count: usize) -> CompletionCandidates {
            CompletionCandidates {
                paths: (0..count)
                    .map(|index| PathBuf::from(format!("/tmp/candidate-{index}")))
                    .collect(),
                has_more: false,
            }
        }

        /// Presentation defaults used by the layout and terminal-setup tests:
        /// 10 rows, no truncation, no border, stderr backend, no colours.
        fn test_options() -> MenuOptions {
            MenuOptions {
                max_rows: 10,
                ..MenuOptions::default()
            }
        }

        fn borderless_options(item_max_len: Option<usize>) -> MenuOptions {
            MenuOptions {
                max_rows: 10,
                item_max_len,
                ..MenuOptions::default()
            }
        }

        fn bordered_options(item_max_len: Option<usize>) -> MenuOptions {
            MenuOptions {
                max_rows: 10,
                item_max_len,
                show_border: true,
                ..MenuOptions::default()
            }
        }

        fn select_with_mock(
            terminal_ops: &MockTerminalOps,
            candidate_count: usize,
            prompt_row_override: Option<u16>,
        ) -> Option<MenuResult> {
            select_with_terminal_ops(
                MenuRequest {
                    candidates: candidates(candidate_count),
                    query: "",
                    mode: crate::menu::MenuMode::Path,
                    cwd: Path::new("/tmp"),
                    prompt_row: prompt_row_override,
                    query_fn: Box::new(|_| panic!("setup failure must not re-query")),
                },
                &test_options(),
                terminal_ops,
            )
        }

        struct FlushFailure;

        impl Write for FlushFailure {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                Ok(bytes.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::other("injected flush failure"))
            }
        }

        #[test]
        fn successful_scroll_reports_reserved_prompt_geometry() {
            let mut output = Vec::new();
            let reservation =
                scroll_terminal(&mut output, 23, 24, 10).expect("scroll should succeed");

            assert_eq!(
                reservation,
                SpaceReservation {
                    prompt_row: 13,
                    scroll_rows: 10,
                }
            );
            assert!(!output.is_empty());
        }

        #[test]
        fn reservation_without_scroll_reports_zero_scroll_rows() {
            let terminal_ops = MockTerminalOps::new();
            let mut session = TerminalSession::start(&terminal_ops, false).expect("start session");

            let reservation = session
                .reserve_space(5, 24, 10)
                .expect("reserve existing rows");

            assert_eq!(
                reservation,
                SpaceReservation {
                    prompt_row: 5,
                    scroll_rows: 0,
                }
            );
            assert_eq!(terminal_ops.open_calls.get(), 0);
        }

        #[test]
        fn failed_scroll_flush_does_not_produce_a_reservation() {
            let error = scroll_terminal(&mut FlushFailure, 23, 24, 10)
                .expect_err("flush failure should fail reservation");

            assert_eq!(error.kind(), std::io::ErrorKind::Other);
        }

        #[test]
        fn restoration_without_acquired_state_emits_nothing() {
            let mut output = Vec::new();
            restore_terminal_output(&mut output, None, None, false);
            assert!(output.is_empty());
        }

        #[test]
        fn raw_mode_enable_failure_needs_no_cleanup() {
            let mut terminal_ops = MockTerminalOps::new();
            terminal_ops.fail_enable = true;

            assert!(select_with_mock(&terminal_ops, 2, Some(0)).is_none());
            assert_eq!(terminal_ops.enable_calls.get(), 1);
            assert_eq!(terminal_ops.disable_calls.get(), 0);
            assert_eq!(terminal_ops.open_calls.get(), 0);
        }

        #[test]
        fn cursor_row_failure_aborts_without_assuming_terminal_bottom() {
            let mut terminal_ops = MockTerminalOps::new();
            terminal_ops.fail_cursor_row = true;

            assert!(select_with_mock(&terminal_ops, 2, None).is_none());
            assert_eq!(terminal_ops.cursor_row_calls.get(), 1);
            assert_eq!(terminal_ops.disable_calls.get(), 1);
            assert_eq!(terminal_ops.open_calls.get(), 0);
        }

        #[test]
        fn measured_cursor_near_top_does_not_scroll_during_startup() {
            let mut terminal_ops = MockTerminalOps::new();
            terminal_ops.cursor_row = 5;
            terminal_ops.fail_open_at = Some(2);

            assert!(select_with_mock(&terminal_ops, 20, None).is_none());
            assert_eq!(terminal_ops.cursor_row_calls.get(), 1);
            assert!(!terminal_ops.output_contains_scroll_up());
        }

        #[test]
        fn explicit_prompt_row_bypasses_terminal_cursor_query() {
            let mut terminal_ops = MockTerminalOps::new();
            terminal_ops.fail_cursor_row = true;
            terminal_ops.fail_open_at = Some(2);

            assert!(select_with_mock(&terminal_ops, 20, Some(5)).is_none());
            assert_eq!(terminal_ops.cursor_row_calls.get(), 0);
            assert!(!terminal_ops.output_contains_scroll_up());
        }

        #[test]
        fn output_open_failure_disables_raw_mode_without_cursor_restore() {
            let mut terminal_ops = MockTerminalOps::new();
            terminal_ops.fail_open_at = Some(1);

            assert!(select_with_mock(&terminal_ops, 2, Some(0)).is_none());
            assert_eq!(terminal_ops.disable_calls.get(), 1);
            assert_eq!(terminal_ops.open_calls.get(), 1);
            assert!(!terminal_ops.output_contains(b"\x1b[?25h"));
        }

        #[test]
        fn reservation_failure_does_not_clear_rows_or_show_cursor() {
            let terminal_ops = MockTerminalOps::new();
            terminal_ops.writer_state.borrow_mut().fail_flush = true;

            assert!(select_with_mock(&terminal_ops, 2, None).is_none());
            assert_eq!(terminal_ops.disable_calls.get(), 1);
            assert!(!terminal_ops.output_contains(b"\x1b[2K"));
            assert!(!terminal_ops.output_contains(b"\x1b[?25h"));
        }

        #[test]
        fn cursor_hide_failure_disables_raw_mode_without_showing_cursor() {
            let terminal_ops = MockTerminalOps::new();
            terminal_ops.writer_state.borrow_mut().fail_write = true;

            assert!(select_with_mock(&terminal_ops, 2, Some(0)).is_none());
            assert_eq!(terminal_ops.disable_calls.get(), 1);
            assert!(!terminal_ops.output_contains(b"\x1b[?25h"));
        }

        #[test]
        fn loop_startup_failure_restores_reserved_rows_and_cursor() {
            let mut terminal_ops = MockTerminalOps::new();
            terminal_ops.fail_open_at = Some(2);

            assert!(select_with_mock(&terminal_ops, 2, None).is_none());
            assert_eq!(terminal_ops.open_calls.get(), 2);
            assert_eq!(terminal_ops.disable_calls.get(), 1);
            assert!(terminal_ops.output_contains(b"\x1b[2K"));
            assert!(terminal_ops.output_contains(b"\x1b[?25h"));
        }

        #[test]
        fn restoration_only_shows_cursor_when_session_hid_it() {
            let region = CleanupRegion {
                prompt_row: 2,
                area: Rect::new(0, 3, 80, 3),
            };
            let mut hidden_output = Vec::new();
            restore_terminal_output(&mut hidden_output, Some(region), None, true);
            assert!(hidden_output.windows(6).any(|bytes| bytes == b"\x1b[?25h"));

            let mut visible_output = Vec::new();
            restore_terminal_output(&mut visible_output, Some(region), None, false);
            assert!(!visible_output.windows(6).any(|bytes| bytes == b"\x1b[?25h"));
            assert!(visible_output.windows(4).any(|bytes| bytes == b"\x1b[2K"));
        }

        #[test]
        fn single_candidate_selection_does_not_require_terminal_setup() {
            let only_path = PathBuf::from("/tmp/candidate-0");
            let terminal_ops = MockTerminalOps::new();
            let result = select_with_mock(&terminal_ops, 1, None);

            assert!(matches!(
                result,
                Some(MenuResult::Selected {
                    value,
                    terminal: crate::menu::action::TerminalState::Clean,
                    ..
                }) if value == only_path
            ));
            assert_eq!(terminal_ops.size_calls.get(), 0);
            assert_eq!(terminal_ops.enable_calls.get(), 0);
            assert_eq!(terminal_ops.open_calls.get(), 0);
            assert_eq!(terminal_ops.disable_calls.get(), 0);
        }

        #[test]
        fn display_label_relative_under_cwd() {
            let cwd = Path::new("/Users/nick");
            let path = Path::new("/Users/nick/Desktop");
            assert_eq!(display_label(path, cwd, None, true), "./Desktop");
        }

        #[test]
        fn display_label_is_relative_for_equivalent_symlinked_cwd() {
            let temp = crate::test_support::temp_dir("menu-display-symlink-cwd");
            let real_cwd = temp.path().join("real");
            let linked_cwd = temp.path().join("linked");
            let path = real_cwd.join("documentation");
            fs::create_dir_all(&path).expect("create candidate directory");
            symlink("real", &linked_cwd).expect("create cwd symlink");

            assert_eq!(
                display_label(&path, &linked_cwd, None, true),
                "./documentation"
            );
            assert_eq!(
                display_label_for_style(
                    &path,
                    &linked_cwd,
                    None,
                    CandidateLabelStyle::BareRelative,
                ),
                "documentation"
            );
        }

        #[test]
        fn candidate_label_style_from_query_only_applies_to_filesystem_modes() {
            assert_eq!(
                CandidateLabelStyle::from_query(
                    crate::menu::MenuMode::Completion(crate::complete::CompletionMode::Paths),
                    "",
                ),
                CandidateLabelStyle::BareRelative
            );
            assert_eq!(
                CandidateLabelStyle::from_query(
                    crate::menu::MenuMode::Completion(crate::complete::CompletionMode::Frecents),
                    "",
                ),
                CandidateLabelStyle::Compact
            );
        }

        #[test]
        fn candidate_label_style_from_query_detects_explicit_styles() {
            let mode = crate::menu::MenuMode::Completion(crate::complete::CompletionMode::Paths);

            assert_eq!(
                CandidateLabelStyle::from_query(mode, "src"),
                CandidateLabelStyle::BareRelative
            );
            assert_eq!(
                CandidateLabelStyle::from_query(mode, "./src"),
                CandidateLabelStyle::DotRelative
            );
            assert_eq!(
                CandidateLabelStyle::from_query(mode, "../src"),
                CandidateLabelStyle::ParentRelative
            );
            assert_eq!(
                CandidateLabelStyle::from_query(mode, "~/src"),
                CandidateLabelStyle::HomeRelative
            );
            assert_eq!(
                CandidateLabelStyle::from_query(mode, "/tmp/src"),
                CandidateLabelStyle::Absolute
            );
        }

        #[test]
        fn display_label_for_empty_query_uses_bare_cwd_relative_label() {
            let cwd = Path::new("/Users/nick/project");
            let path = Path::new("/Users/nick/project/src");

            assert_eq!(
                display_label_for_style(path, cwd, None, CandidateLabelStyle::BareRelative),
                "src"
            );
        }

        #[test]
        fn display_label_for_bare_query_uses_bare_cwd_relative_label() {
            let cwd = Path::new("/Users/nick/project");
            let path = Path::new("/Users/nick/project/src");

            assert_eq!(
                display_label_for_style(path, cwd, None, CandidateLabelStyle::BareRelative),
                "src"
            );
        }

        #[test]
        fn display_label_for_dot_query_preserves_dot_prefix() {
            let cwd = Path::new("/Users/nick/project");
            let path = Path::new("/Users/nick/project/src");

            assert_eq!(
                display_label_for_style(path, cwd, None, CandidateLabelStyle::DotRelative),
                "./src"
            );
        }

        #[test]
        fn display_label_for_parent_query_preserves_parent_prefix() {
            let cwd = Path::new("/Users/nick/project");
            let path = Path::new("/Users/nick/sibling");

            assert_eq!(
                display_label_for_style(path, cwd, None, CandidateLabelStyle::ParentRelative),
                "../sibling"
            );
        }

        #[test]
        fn display_label_for_parent_query_keeps_anchor_for_cwd_candidate() {
            let cwd = Path::new("/Users/nick/project");

            assert_eq!(
                display_label_for_style(cwd, cwd, None, CandidateLabelStyle::ParentRelative),
                "../project"
            );
        }

        #[test]
        fn display_label_for_parent_query_normalizes_candidate_parent_components() {
            let cwd = Path::new("/Users/nick/code/personal/dx");
            let path = Path::new("/Users/nick/code/personal/dx/../sibling");

            assert_eq!(
                display_label_for_style(path, cwd, None, CandidateLabelStyle::ParentRelative),
                "../sibling"
            );
        }

        #[test]
        fn display_label_for_multi_parent_query_preserves_parent_prefix() {
            let cwd = Path::new("/Users/nick/project/deep");
            let path = Path::new("/Users/nick/outer");

            assert_eq!(
                display_label_for_style(path, cwd, None, CandidateLabelStyle::ParentRelative),
                "../../outer"
            );
        }

        #[test]
        fn display_label_for_home_query_preserves_home_prefix() {
            let cwd = Path::new("/tmp");
            let home = Path::new("/Users/nick");
            let path = Path::new("/Users/nick/code");

            assert_eq!(
                display_label_for_style(path, cwd, Some(home), CandidateLabelStyle::HomeRelative),
                "~/code"
            );
        }

        #[test]
        fn display_label_for_absolute_query_preserves_absolute_path() {
            let cwd = Path::new("/tmp");
            let path = Path::new("/Users/nick/code");

            assert_eq!(
                display_label_for_style(path, cwd, None, CandidateLabelStyle::Absolute),
                "/Users/nick/code"
            );
        }

        #[test]
        fn compact_label_style_preserves_non_filesystem_mode_behavior() {
            let cwd = Path::new("/Users/nick/project");
            let path = Path::new("/Users/nick/project/src");

            assert_eq!(
                display_label_for_style(path, cwd, None, CandidateLabelStyle::Compact),
                "./src"
            );
        }

        #[test]
        fn status_path_remains_full_when_item_label_is_query_style_relative() {
            let cwd = Path::new("/Users/nick/project");
            let path = PathBuf::from("/Users/nick/project/src");

            assert_eq!(
                display_label_for_style(&path, cwd, None, CandidateLabelStyle::BareRelative),
                "src"
            );
            assert_eq!(
                selected_status_path(&[path], Some(0)),
                "/Users/nick/project/src"
            );
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
            let layout = build_menu_layout(
                Rect::new(0, 0, 20, 8),
                50,
                &labels,
                &borderless_options(Some(8)),
            );
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
        fn selected_status_path_uses_resolved_path_not_compact_label() {
            let paths = vec![PathBuf::from("/Users/nick/code/personal/dx/src")];
            assert_eq!(
                selected_status_path(&paths, Some(0)),
                "/Users/nick/code/personal/dx/src"
            );
        }

        #[test]
        fn selected_status_path_falls_back_for_empty_selection() {
            let paths = vec![PathBuf::from("/tmp/alpha")];

            assert_eq!(selected_status_path(&paths, None), "(no matches)");
            assert_eq!(selected_status_path(&paths, Some(99)), "(no matches)");
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
            assert_eq!(compute_rendered_height(80, 0, &[], &test_options()), 3);
            assert_eq!(
                compute_rendered_height(80, 0, &[], &bordered_options(None)),
                4
            );
        }

        #[test]
        fn compute_rendered_height_shrinks_with_filtered_single_column_results() {
            let labels = vec!["alpha".to_string(); 8];
            let smaller = vec!["alpha".to_string(); 1];

            let tall = compute_rendered_height(80, labels.len(), &labels, &test_options());
            let short = compute_rendered_height(80, smaller.len(), &smaller, &test_options());

            assert!(short < tall);
            assert_eq!(short, 3);
        }

        #[test]
        fn compute_rendered_height_shrinks_with_filtered_bordered_single_column_results() {
            let labels = vec!["alpha".to_string(); 8];
            let smaller = vec!["alpha".to_string(); 1];

            let tall = compute_rendered_height(20, labels.len(), &labels, &bordered_options(None));
            let short =
                compute_rendered_height(20, smaller.len(), &smaller, &bordered_options(None));

            assert!(short < tall);
            assert_eq!(short, 4);
        }

        #[test]
        fn compute_rendered_height_shrinks_with_filtered_multicolumn_results() {
            let many = vec!["abcdefgh".to_string(); 9];
            let few = vec!["abcdefgh".to_string(); 3];

            let tall = compute_rendered_height(36, many.len(), &many, &bordered_options(Some(8)));
            let short = compute_rendered_height(36, few.len(), &few, &bordered_options(Some(8)));

            assert!(short < tall);
            assert_eq!(short, 4);
        }

        #[test]
        fn compute_rendered_height_shrinks_with_filtered_borderless_multicolumn_results() {
            let many = vec!["abcdefgh".to_string(); 9];
            let few = vec!["abcdefgh".to_string(); 3];

            let tall = compute_rendered_height(36, many.len(), &many, &borderless_options(Some(8)));
            let short = compute_rendered_height(36, few.len(), &few, &borderless_options(Some(8)));

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
        fn prompt_row_is_clamped_to_terminal_bounds() {
            assert_eq!(clamp_prompt_row(5, 24), 5);
            assert_eq!(clamp_prompt_row(30, 24), 23);
            assert_eq!(clamp_prompt_row(0, 0), 0);
        }

        #[test]
        fn cursor_position_response_parses_zero_based_row() {
            assert_eq!(parse_cursor_row_response(b"\x1b[6;42R"), Some(5));
            assert_eq!(parse_cursor_row_response(b"noise\x1b[24;1R"), Some(23));
            assert_eq!(parse_cursor_row_response(b"\x1b[0;1R"), None);
            assert_eq!(parse_cursor_row_response(b"invalid"), None);
        }

        #[test]
        fn measured_prompt_near_top_does_not_scroll_when_menu_fits() {
            let prompt_row = clamp_prompt_row(5, 24);
            let scroll_needed = scroll_rows_needed(prompt_row, 24, 10);

            assert_eq!(scroll_needed, 0);
            assert_eq!(menu_top_row(prompt_row, 24, 10, true), 6);
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
        fn key_event_mapping_page_keys_move_by_page() {
            let page_down = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);
            let page_up = KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE);
            assert_eq!(map_key_event(page_down, false), MenuKeyAction::MovePage(1));
            assert_eq!(map_key_event(page_up, false), MenuKeyAction::MovePage(-1));
            assert_eq!(map_key_event(page_down, true), MenuKeyAction::MovePage(1));
            assert_eq!(map_key_event(page_up, true), MenuKeyAction::MovePage(-1));
        }

        #[test]
        fn key_event_mapping_tab_and_backtab_remain_navigation() {
            let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
            let backtab = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
            assert_eq!(map_key_event(tab, false), MenuKeyAction::MoveLinear(1));
            assert_eq!(map_key_event(backtab, false), MenuKeyAction::MoveLinear(-1));
        }

        #[test]
        fn page_selection_step_uses_visible_rows_for_single_column() {
            assert_eq!(page_selection_step(10, 1, false), 10);
            assert_eq!(page_selection_step(0, 1, false), 1);
        }

        #[test]
        fn page_selection_step_uses_visible_grid_capacity_for_multicolumn() {
            assert_eq!(page_selection_step(4, 3, true), 12);
            assert_eq!(page_selection_step(0, 3, true), 1);
            assert_eq!(page_selection_step(4, 0, true), 1);
        }

        #[test]
        fn move_selection_page_moves_forward_and_backward_with_clamping() {
            let mut state = ListState::default();
            state.select(Some(0));

            move_selection_page(&mut state, 30, 10, 1);
            assert_eq!(state.selected(), Some(10));

            move_selection_page(&mut state, 30, 10, -1);
            assert_eq!(state.selected(), Some(0));

            state.select(Some(25));
            move_selection_page(&mut state, 30, 10, 1);
            assert_eq!(state.selected(), Some(29));

            state.select(Some(3));
            move_selection_page(&mut state, 30, 10, -1);
            assert_eq!(state.selected(), Some(0));
        }

        #[test]
        fn move_selection_page_uses_minimum_step_and_handles_empty_lists() {
            let mut state = ListState::default();
            state.select(Some(0));

            move_selection_page(&mut state, 5, 0, 1);
            assert_eq!(state.selected(), Some(1));

            move_selection_page(&mut state, 0, 10, 1);
            assert_eq!(state.selected(), None);
        }

        #[test]
        fn move_selection_page_supports_multicolumn_grid_capacity() {
            let mut state = ListState::default();
            let page_size = page_selection_step(4, 3, true);

            state.select(Some(0));
            move_selection_page(&mut state, 30, page_size, 1);
            assert_eq!(state.selected(), Some(12));

            state.select(Some(14));
            move_selection_page(&mut state, 30, page_size, -1);
            assert_eq!(state.selected(), Some(2));

            state.select(Some(25));
            move_selection_page(&mut state, 30, page_size, 1);
            assert_eq!(state.selected(), Some(29));
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

            let geometry = crate::menu::action::TerminalGeometry {
                redraw_row: 13,
                scroll_rows: 10,
            };

            assert_eq!(
                cancelled_result(&filter, geometry),
                MenuResult::Cancelled {
                    filter_query: "Do".to_string(),
                    changed_query: false,
                    geometry: Some(geometry),
                }
            );
        }

        #[test]
        fn interactive_selection_preserves_terminal_geometry() {
            let filter = FilterState::new("Do");
            let geometry = crate::menu::action::TerminalGeometry {
                redraw_row: 13,
                scroll_rows: 10,
            };

            assert_eq!(
                selected_result(&filter, PathBuf::from("/tmp/Documents"), geometry),
                MenuResult::Selected {
                    filter_query: "Do".to_string(),
                    changed_query: false,
                    value: PathBuf::from("/tmp/Documents"),
                    terminal: crate::menu::action::TerminalState::Dirty,
                    geometry: Some(geometry),
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
        fn candidate_span_uses_ls_colors_for_non_selected_item() {
            let ls_colors = crate::menu::ls_colors::parse_ls_colors("*.rs=01;31");
            let span = candidate_span(
                "main.rs".to_string(),
                Path::new("/tmp/main.rs"),
                false,
                Some(&ls_colors),
            );

            assert_eq!(span.content.as_ref(), "main.rs");
            assert_eq!(span.style.fg, Some(Color::Red));
            assert!(span.style.add_modifier & Modifier::BOLD != Modifier::empty());
        }

        #[test]
        fn candidate_span_stays_plain_when_ls_colors_disabled() {
            let span = candidate_span(
                "main.rs".to_string(),
                Path::new("/tmp/main.rs"),
                false,
                None,
            );

            assert_eq!(span.content.as_ref(), "main.rs");
            assert_eq!(span.style, Style::default());
        }

        #[test]
        fn candidate_span_selected_item_overrides_ls_colors() {
            let ls_colors = crate::menu::ls_colors::parse_ls_colors("*.rs=01;31");
            let span = candidate_span(
                "main.rs".to_string(),
                Path::new("/tmp/main.rs"),
                true,
                Some(&ls_colors),
            );

            assert_eq!(span.content.as_ref(), "main.rs");
            assert_eq!(span.style, selected_style());
        }

        #[test]
        fn render_grid_items_styles_non_selected_cells_and_preserves_selected_highlight() {
            let ls_colors = crate::menu::ls_colors::parse_ls_colors("*.rs=01;31:*.md=01;32");
            let completion = CompletionCandidates {
                paths: vec![
                    PathBuf::from("/tmp/main.rs"),
                    PathBuf::from("/tmp/README.md"),
                ],
                has_more: false,
            };
            let labels = vec!["main.rs".to_string(), "README.md".to_string()];
            let layout = build_menu_layout(
                Rect::new(0, 0, 40, 3),
                2,
                &labels,
                &borderless_options(Some(20)),
            );

            let lines = render_grid_items(&completion, &labels, Some(&ls_colors), &layout, 1);
            let spans = &lines[0].spans;

            assert_eq!(spans[0].style.fg, Some(Color::Red));
            assert_eq!(spans[1].style, selected_style());
        }
    }
}

#[cfg(not(unix))]
mod imp {
    use super::{MenuOptions, MenuRequest, MenuResult};

    pub fn select(request: MenuRequest<'_>, _options: &MenuOptions) -> Option<MenuResult> {
        Some(MenuResult::Cancelled {
            filter_query: request.query.to_string(),
            changed_query: false,
            geometry: None,
        })
    }
}

pub use imp::select;
