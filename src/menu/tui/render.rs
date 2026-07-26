//! Drawing the candidates: single-column list, multicolumn grid, scrollbar and
//! status row.

use std::path::Path;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, ScrollbarState};

use crate::menu::ls_colors::LsColorsConfig;
use crate::resolve::CompletionCandidates;

use super::MenuOptions;
use super::layout::{MenuLayoutPlan, build_scrollbar, menu_scrollbar_render_area};
use super::selection::visible_window;
use super::status::build_status_text;
use super::width::{pad_to_width, truncate_for_cell};

pub(super) fn render_grid_items(
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

pub(super) fn selected_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

pub(super) fn candidate_span(
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

pub(super) fn render_grid(
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

pub(super) fn render_list(
    frame: &mut ratatui::Frame<'_>,
    completion: &CompletionCandidates,
    labels: &[String],
    options: &MenuOptions,
    layout: &MenuLayoutPlan,
    list_state: &mut ListState,
) {
    let show_border = options.show_border;
    let styled = visible_window(
        list_state.offset(),
        list_state.selected(),
        layout.visible_rows,
        labels.len(),
    );
    let items: Vec<ListItem> = labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let selected = list_state.selected() == Some(i);
            // Colouring a row costs a stat; rows outside the visible window
            // are never drawn, so only look those up.
            let ls_colors = if styled.contains(&i) {
                options.ls_colors.as_ref()
            } else {
                None
            };
            let line = Line::from(candidate_span(
                label.clone(),
                &completion.paths[i],
                selected,
                ls_colors,
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

pub(super) fn render_scrollbar(
    frame: &mut ratatui::Frame<'_>,
    layout: &MenuLayoutPlan,
    show_border: bool,
    state: &mut ScrollbarState,
) {
    let area = menu_scrollbar_render_area(layout.content_area, layout.scrollbar_area, show_border);
    frame.render_stateful_widget(build_scrollbar(show_border), area, state);
}

pub(super) fn render_status(
    frame: &mut ratatui::Frame<'_>,
    chunks: &[Rect],
    divider_area: Option<Rect>,
    selected_path: &str,
    overflow: &str,
    typed_refinement: &str,
) {
    let status_text = build_status_text(chunks[1].width, selected_path, overflow, typed_refinement);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use super::super::fixtures::*;
    use super::super::layout::build_menu_layout;
    use ratatui::layout::Rect;

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

    /// The real grid, measured by ratatui's own `Span::width`. Every row must
    /// break its columns at the same offsets or the grid staircases.
    #[test]
    fn render_grid_items_aligns_columns_across_rows_with_wide_labels() {
        let names = [
            "main.rs",
            "日本語のディレクトリ",
            "README.md",
            "café",
            "🇬🇧-flags",
            "e\u{0301}tude",
            "src",
            "ドキュメント",
        ];
        let completion = CompletionCandidates {
            paths: names
                .iter()
                .map(|n| PathBuf::from("/tmp").join(n))
                .collect(),
            has_more: false,
        };
        let labels: Vec<String> = names.iter().map(|n| (*n).to_string()).collect();
        let layout = build_menu_layout(
            Rect::new(0, 0, 60, 4),
            labels.len(),
            &labels,
            &borderless_options(Some(20)),
        );
        assert!(
            layout.metrics.columns > 1,
            "test needs a multi-column grid to prove alignment"
        );

        let lines = render_grid_items(&completion, &labels, None, &layout, 0);
        let mut checked = 0;
        for (row, line) in lines.iter().enumerate() {
            let mut x = 0;
            for (col, span) in line.spans.iter().enumerate() {
                assert_eq!(
                    span.width(),
                    layout.metrics.column_widths[col],
                    "row {row} column {col} ({:?}) at x={x} does not fill its column",
                    span.content
                );
                x += span.width();
                checked += 1;
            }
        }
        assert!(checked > 0, "no cells were rendered");
        assert!(
            lines
                .iter()
                .flat_map(|line| &line.spans)
                .any(|span| span.content.contains('日')),
            "no wide label reached the visible area, so nothing was proven"
        );
    }
}
