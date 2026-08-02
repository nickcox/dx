//! Runs one menu session: sets the terminal up, draws, reads events, and
//! returns what the user chose.

use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event};
use ratatui::{
    Terminal, TerminalOptions, Viewport,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Clear, ListState},
};

use super::input::{FilterState, MenuKeyAction, apply_filter_edit, map_key_event, map_mouse_event};
use super::labels::LabelContext;
use super::layout::{
    bounded_rendered_height, build_menu_layout, cleared_trailing_area, compute_rendered_height,
    rendered_menu_area,
};
use super::render::{render_grid, render_list, render_status};
use super::selection::{
    move_selection, move_selection_grid_vertical, move_selection_page, page_selection_step,
};
use super::status::overflow_note;
use super::terminal::{
    CrosstermTerminalOps, TerminalOps, TerminalSession, clamp_prompt_row, menu_top_row,
    required_rows_below,
};
use super::{MenuOptions, MenuRequest, MenuResult};
use crate::menu::{MenuMode, QueryStyle};
use crate::resolve::CompletionCandidates;

/// How long to wait for the next event when folding a run of scrolls together.
///
/// Not [`Duration::ZERO`]: with crossterm's `use-dev-tty` backend a zero-length
/// poll reports nothing available even when the terminal has queued a hundred
/// events, which silently disables the folding. Measured over ~93 queued scroll
/// events on a 4000-entry list, a zero poll redrew for every one of them
/// (~52 KB of output) where this window redraws once (~1 KB).
///
/// The cost is up to one millisecond of added latency on an isolated scroll,
/// which is well under a frame.
const SCROLL_COALESCE_WINDOW: Duration = Duration::from_millis(1);

pub(super) fn selected_result(
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

pub(super) fn cancelled_result(
    filter_state: &FilterState,
    geometry: crate::menu::action::TerminalGeometry,
) -> MenuResult {
    MenuResult::Cancelled {
        filter_query: filter_state.effective_query(),
        changed_query: filter_state.changed_query(),
        geometry: Some(geometry),
    }
}

pub(super) fn selected_status_path(paths: &[PathBuf], selected: Option<usize>) -> String {
    selected
        .and_then(|idx| paths.get(idx))
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "(no matches)".to_string())
}

pub fn select(request: MenuRequest<'_>, options: &MenuOptions) -> Option<MenuResult> {
    select_with_terminal_ops(request, options, &CrosstermTerminalOps)
}

pub(super) fn select_with_terminal_ops(
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
    let mut label_context = LabelContext::new(request.cwd, home.as_deref());
    let initial_labels: Vec<String> = request
        .candidates
        .paths
        .iter()
        .map(|p| label_context.label(p, initial_label_style))
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
    // Scrolling is a convenience; a terminal that refuses capture still gets a menu.
    let _ = session.capture_mouse();
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
    let mut label_context = LabelContext::new(cwd, home.as_deref());
    let mut previous_height = area.height;

    let mut plan = build_render_plan(
        &completion,
        &filter_state,
        &mut label_context,
        mode,
        area,
        options,
    );
    let mut needs_redraw = true;
    // Holds a non-scroll event read while draining a scroll run, so it is acted
    // on in the order the user produced it.
    let mut pending_action: Option<MenuKeyAction> = None;

    loop {
        if needs_redraw {
            let selected_path = selected_status_path(&completion.paths, list_state.selected());
            let overflow = overflow_note(completion.paths.len(), completion.has_more);
            let RenderPlan {
                labels,
                current_height,
                current_area,
                layout,
            } = &plan;
            let (current_height, current_area) = (*current_height, *current_area);

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
                        render_grid(frame, &completion, labels, options, layout, selected);
                    } else {
                        render_list(frame, &completion, labels, options, layout, &mut list_state);
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
        }

        let len = completion.paths.len();
        let columns = plan.layout.metrics.columns;
        let use_grid = plan.layout.metrics.use_grid;

        let action = match pending_action.take() {
            Some(action) => action,
            None => match event::read().ok()? {
                Event::Key(key) => map_key_event(key, use_grid),
                Event::Mouse(mouse) => map_mouse_event(mouse),
                _ => MenuKeyAction::Ignore,
            },
        };

        // A flick delivers far more scroll events than there are useful frames,
        // and only the final position is ever seen. Fold the ones already queued
        // into this move so a long list redraws once instead of once per event.
        let action = match action {
            MenuKeyAction::ScrollRow(first) => {
                let mut rows = first;
                while pending_action.is_none()
                    && event::poll(SCROLL_COALESCE_WINDOW).unwrap_or(false)
                {
                    match event::read().ok()? {
                        Event::Mouse(mouse) => match map_mouse_event(mouse) {
                            MenuKeyAction::ScrollRow(next) => rows = rows.saturating_add(next),
                            MenuKeyAction::Ignore => {}
                            other => pending_action = Some(other),
                        },
                        // A key press ends the run: it must be handled in order,
                        // so hold it for the next iteration rather than dropping
                        // it or acting on it out of sequence.
                        Event::Key(key) => pending_action = Some(map_key_event(key, use_grid)),
                        _ => {}
                    }
                }
                MenuKeyAction::ScrollRow(rows)
            }
            other => other,
        };

        // Nothing on screen depends on an event the menu discards, and a
        // trackpad can deliver them faster than a long list redraws.
        needs_redraw = !matches!(action, MenuKeyAction::Ignore | MenuKeyAction::ScrollRow(0));

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
                let page_size = page_selection_step(plan.layout.visible_rows, columns, use_grid);
                move_selection_page(&mut list_state, len, page_size, direction);
            }
            // Zero means a run of scrolls cancelled out, so the selection stays put.
            MenuKeyAction::ScrollRow(0) => {}
            MenuKeyAction::ScrollRow(rows) => {
                // One row is a whole grid line; `move_selection_page` clamps for us.
                let step = if use_grid { columns.max(1) } else { 1 };
                let distance = step.saturating_mul(rows.unsigned_abs());
                move_selection_page(&mut list_state, len, distance, rows);
            }
            // Only a filter edit can change the candidates or the label style,
            // so this is the one branch that has to rebuild the plan.
            MenuKeyAction::Backspace | MenuKeyAction::InputChar(_) => {
                apply_filter_edit(
                    &mut filter_state,
                    &mut completion,
                    &mut list_state,
                    action,
                    &query_fn,
                );
                plan = build_render_plan(
                    &completion,
                    &filter_state,
                    &mut label_context,
                    mode,
                    area,
                    options,
                );
            }
            MenuKeyAction::Ignore => {}
        }
    }
}

/// Everything a frame needs that is derived from the candidates and the active
/// query.
///
/// Building it is O(candidates) — a label per path, then two passes of layout —
/// so it is held across iterations. Moving the selection cannot invalidate it;
/// only a filter edit can, because only that changes the candidate list or the
/// label style.
struct RenderPlan {
    labels: Vec<String>,
    current_height: u16,
    current_area: Rect,
    layout: super::layout::MenuLayoutPlan,
}

fn build_render_plan(
    completion: &CompletionCandidates,
    filter_state: &FilterState,
    label_context: &mut LabelContext,
    mode: MenuMode,
    area: Rect,
    options: &MenuOptions,
) -> RenderPlan {
    let effective_query = filter_state.effective_query();
    let label_style = QueryStyle::from_query(mode, &effective_query);
    let labels: Vec<String> = completion
        .paths
        .iter()
        .map(|p| label_context.label(p, label_style))
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

    RenderPlan {
        labels,
        current_height,
        current_area,
        layout,
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::*;

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
}
