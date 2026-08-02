//! Moving the selection. Arrow keys wrap; paging and scrolling clamp.

use ratatui::widgets::ListState;

/// The rows a `List` will actually draw, mirroring ratatui's scroll-to-
/// selected behaviour and padded by a row either side so an adjustment
/// inside ratatui cannot leave a drawn row unstyled.
pub(super) fn visible_window(
    offset: usize,
    selected: Option<usize>,
    height: usize,
    len: usize,
) -> std::ops::Range<usize> {
    if height == 0 || len == 0 {
        return 0..0;
    }

    let mut start = offset.min(len.saturating_sub(1));
    if let Some(selected) = selected.filter(|selected| *selected < len) {
        if selected < start {
            start = selected;
        } else if selected >= start + height {
            start = selected + 1 - height;
        }
    }

    let start = start.saturating_sub(1);
    start..(start + height + 2).min(len)
}

pub(super) fn move_selection_grid_vertical(
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

pub(super) fn reset_selection(state: &mut ListState, len: usize) {
    if len == 0 {
        state.select(None);
    } else {
        state.select(Some(0));
    }
}

pub(super) fn move_selection(state: &mut ListState, len: usize, delta: isize) {
    if len == 0 {
        state.select(None);
        return;
    }
    let current = state.selected().unwrap_or(0) as isize;
    let next = (current + delta).rem_euclid(len as isize) as usize;
    state.select(Some(next));
}

pub(super) fn page_selection_step(visible_rows: usize, columns: usize, use_grid: bool) -> usize {
    if use_grid {
        visible_rows.saturating_mul(columns).max(1)
    } else {
        visible_rows.max(1)
    }
}

pub(super) fn move_selection_page(
    state: &mut ListState,
    len: usize,
    page_size: usize,
    direction: isize,
) {
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
    use super::*;

    /// A flick of the wheel must not spin past the last candidate and back to
    /// the first; only the arrow keys wrap.
    #[test]
    fn wheel_stops_at_both_ends_where_the_arrows_wrap() {
        let mut state = ListState::default();

        state.select(Some(0));
        move_selection_page(&mut state, 5, 1, -1);
        assert_eq!(
            state.selected(),
            Some(0),
            "scrolling up past the top wrapped"
        );

        state.select(Some(4));
        move_selection_page(&mut state, 5, 1, 1);
        assert_eq!(
            state.selected(),
            Some(4),
            "scrolling down past the bottom wrapped"
        );

        // The arrows still wrap, which is what makes the two behaviours distinct.
        state.select(Some(4));
        move_selection(&mut state, 5, 1);
        assert_eq!(state.selected(), Some(0));
        state.select(Some(0));
        move_selection(&mut state, 5, -1);
        assert_eq!(state.selected(), Some(4));
    }

    #[test]
    fn visible_window_covers_the_selection_when_scrolled() {
        // selection above the current offset scrolls up to it
        let window = visible_window(40, Some(10), 20, 100);
        assert!(window.contains(&10), "{window:?}");

        // selection below the window scrolls down so it is the last row
        let window = visible_window(0, Some(50), 20, 100);
        assert!(window.contains(&50), "{window:?}");

        // selection already inside keeps the offset
        let window = visible_window(30, Some(35), 20, 100);
        assert!(window.contains(&35), "{window:?}");
    }

    #[test]
    fn visible_window_is_bounded_and_padded() {
        let window = visible_window(0, Some(0), 20, 2400);
        assert_eq!(window.start, 0);
        assert!(
            window.len() <= 22,
            "a 20-row list should style at most 22 of 2400 candidates, got {}",
            window.len()
        );

        // never runs past the candidate list
        let window = visible_window(0, Some(99), 20, 100);
        assert!(window.end <= 100, "{window:?}");
    }

    #[test]
    fn visible_window_handles_degenerate_inputs() {
        assert!(visible_window(0, None, 0, 100).is_empty());
        assert!(visible_window(0, Some(0), 20, 0).is_empty());
        // an offset past the end must not panic or invert the range
        let window = visible_window(500, Some(1), 20, 3);
        assert!(window.start <= window.end && window.end <= 3, "{window:?}");
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

    /// The menu folds a queued run of wheel events into one move by multiplying
    /// the row step. That is only sound if it lands where the events would have
    /// one at a time, including when the run clamps at an end.
    #[test]
    fn one_folded_scroll_lands_where_the_separate_scrolls_would() {
        for (len, step, count) in [(30, 1, 5), (30, 7, 3), (30, 7, 20), (5, 3, 4)] {
            for direction in [1_isize, -1] {
                let mut folded = ListState::default();
                folded.select(Some(if direction > 0 { 0 } else { len - 1 }));
                let mut separate = folded;

                move_selection_page(&mut folded, len, step * count, direction);
                for _ in 0..count {
                    move_selection_page(&mut separate, len, step, direction);
                }

                assert_eq!(
                    folded.selected(),
                    separate.selected(),
                    "len={len} step={step} count={count} direction={direction}"
                );
            }
        }
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
}
