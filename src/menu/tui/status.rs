//! The status row below the candidates: selected path, overflow note, and the
//! refinement typed during this menu session, fitted to the terminal width.

use super::width::{text_width, truncate_for_cell};

pub(super) fn build_status_text(
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

pub(super) fn refinement_cap(width: usize, refinement: &str) -> usize {
    let natural = text_width(refinement);
    let cap = (width / 3).clamp(4, 32);
    natural.min(cap)
}

pub(super) fn join_status_parts(width: usize, left: &str, right: &str) -> Option<String> {
    let left_width = text_width(left);
    let right_width = refinement_cap(width, right);
    if width < left_width + 1 + right_width {
        return None;
    }

    let right = truncate_for_cell(right, right_width);
    let gap = width - left_width - text_width(&right);
    Some(format!("{left}{}{right}", " ".repeat(gap)))
}

pub(super) fn overflow_note(displayed: usize, has_more: bool) -> String {
    if has_more {
        format!(" | showing first {displayed}")
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
