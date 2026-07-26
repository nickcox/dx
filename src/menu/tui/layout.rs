//! Menu geometry: how many rows and columns the candidates need, how wide each
//! column is, and which rectangles the content, divider and scrollbar occupy.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Scrollbar, ScrollbarOrientation};

use super::MenuOptions;
use super::width::effective_item_max_len;

pub(super) fn compute_list_rows(rows_total: usize, max_rows: u16) -> u16 {
    let cap = max_rows.max(1);
    cap.min(u16::try_from(rows_total.max(1)).unwrap_or(u16::MAX))
}

pub(super) fn compute_rendered_height(
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

pub(super) fn menu_chrome_height(show_border: bool) -> u16 {
    if show_border { 3 } else { 2 }
}

pub(super) fn bounded_rendered_height(reserved_height: u16, target_height: u16) -> u16 {
    reserved_height.min(target_height)
}

pub(super) fn rendered_menu_area(area: Rect, rendered_height: u16) -> Rect {
    Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: rendered_height.min(area.height),
    }
}

pub(super) fn cleared_trailing_area(
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

#[derive(Debug, Clone)]
pub(super) struct LayoutMetrics {
    pub(super) columns: usize,
    pub(super) rows_total: usize,
    pub(super) use_grid: bool,
    pub(super) column_widths: Vec<usize>,
}

#[derive(Debug, Clone)]
pub(super) struct MenuLayoutPlan {
    pub(super) content_area: Rect,
    pub(super) divider_area: Option<Rect>,
    pub(super) scrollbar_area: Option<Rect>,
    pub(super) visible_rows: usize,
    pub(super) scrollbar_needed: bool,
    pub(super) metrics: LayoutMetrics,
}

/// Builds the final menu layout with a two-pass calculation.
///
/// Borderless mode can reserve a dedicated scrollbar column, but whether that
/// column is needed depends on the row/column layout, which itself depends on
/// the available width. We therefore do a provisional width probe first,
/// decide whether a scrollbar column is needed, and then recompute the final
/// layout using the true content width.
pub(super) fn build_menu_layout(
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
    let final_metrics = compute_layout_metrics(final_inner_width, item_count, labels, item_max_len);
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

pub(super) fn compute_layout_metrics(
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

pub(super) fn menu_content_area(list_area: Rect, show_border: bool) -> Rect {
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

pub(super) fn menu_divider_area(list_area: Rect, show_border: bool) -> Option<Rect> {
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

pub(super) fn build_scrollbar(show_border: bool) -> Scrollbar<'static> {
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

pub(super) fn menu_scrollbar_render_area(
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

pub(super) fn split_menu_areas(
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

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::*;

    #[test]
    fn list_rows_saturate_instead_of_wrapping_past_u16() {
        // `rows_total as u16` turned 65536 rows into 0, rendering an empty
        // menu. Reachable with a large DX_MAX_MENU_RESULTS.
        assert_eq!(compute_list_rows(65_536, 20), 20);
        assert_eq!(compute_list_rows(usize::MAX, 20), 20);
        assert_eq!(compute_list_rows(5, 20), 5);
        assert_eq!(compute_list_rows(0, 20), 1);
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
        let area =
            menu_scrollbar_render_area(Rect::new(2, 3, 19, 8), Some(Rect::new(21, 3, 1, 8)), false);
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
        let short = compute_rendered_height(20, smaller.len(), &smaller, &bordered_options(None));

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
}
