//! Measuring and fitting label text to terminal cells.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub(super) fn effective_item_max_len(
    labels: &[String],
    item_max_len: Option<usize>,
) -> Option<usize> {
    let configured = item_max_len?;
    if configured < 1 {
        return None;
    }
    let actual = labels
        .iter()
        .map(|s| text_width(s))
        .max()
        .unwrap_or(1)
        .max(1);
    Some(std::cmp::min(configured, actual))
}

/// Keeps the tail behind a leading `…`. A budget a wide cluster cannot fill
/// exactly stays one cell short; overflowing would misalign later columns.
pub(super) fn truncate_for_cell(input: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if text_width(input) <= max {
        return input.to_string();
    }
    if max == 1 {
        return "…".to_string();
    }

    let budget = max - 1;
    let mut used = 0;
    let mut start = input.len();
    for (offset, cluster) in input.grapheme_indices(true).rev() {
        let width = cluster_width(cluster);
        if used + width > budget {
            break;
        }
        used += width;
        start = offset;
    }
    format!("…{}", &input[start..])
}

/// Width in terminal cells, counted as ratatui advances its cursor: one
/// grapheme cluster at a time, by that cluster's own width.
pub(super) fn text_width(input: &str) -> usize {
    if is_single_cell_ascii(input) {
        return input.len();
    }
    input.graphemes(true).map(cluster_width).sum()
}

pub(super) fn cluster_width(cluster: &str) -> usize {
    UnicodeWidthStr::width(cluster)
}

/// ASCII paths are one cell per byte, making segmentation pure overhead.
pub(super) fn is_single_cell_ascii(input: &str) -> bool {
    input
        .bytes()
        .all(|byte| byte.is_ascii_graphic() || byte == b' ')
}

pub(super) fn pad_to_width(input: &str, width: usize) -> String {
    let mut out = input.to_string();
    let len = text_width(&out);
    if width > len {
        out.push_str(&" ".repeat(width - len));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Widths must match ratatui's cursor advance: per grapheme cluster, by
    /// the cluster's own width.
    #[test]
    fn text_width_counts_terminal_cells_not_chars() {
        assert_eq!(text_width("abc"), 3);
        // CJK ideographs occupy two cells each.
        assert_eq!(text_width("日本語"), 6);
        // A combining mark adds no cell of its own.
        assert_eq!(text_width("e\u{0301}"), 1);
        // One cluster, one advance, regardless of how many chars it holds.
        assert_eq!(text_width("🇬🇧"), text_width_via_ratatui("🇬🇧"));
        assert_eq!(text_width("👩‍💻"), text_width_via_ratatui("👩‍💻"));
    }

    /// Independent oracle: sum the cursor advances ratatui itself would make.
    fn text_width_via_ratatui(input: &str) -> usize {
        use ratatui::style::Style;
        use ratatui::text::Span;
        use unicode_width::UnicodeWidthStr;

        Span::raw(input)
            .styled_graphemes(Style::default())
            .map(|g| UnicodeWidthStr::width(g.symbol))
            .sum()
    }

    #[test]
    fn padded_cell_occupies_exactly_the_column_width() {
        for label in [
            "abc",
            "日本語",
            "café",
            "e\u{0301}tude",
            "🇬🇧 flag",
            "日本語abc",
        ] {
            let cell = pad_to_width(&truncate_for_cell(label, 8), 10);
            assert_eq!(
                text_width(&cell),
                10,
                "label {label:?} produced a misaligned cell {cell:?}"
            );
        }
    }

    #[test]
    fn truncate_for_cell_never_exceeds_the_budget_or_splits_a_cluster() {
        // Two cells per ideograph: three fit in the six cells left after '…'.
        assert_eq!(truncate_for_cell("日本語日本語", 7), "…日本語");
        // 6 cells leave 5 after `…`, so only two ideographs fit.
        assert_eq!(truncate_for_cell("日本語日本語", 6), "…本語");
        for label in ["日本語日本語", "e\u{0301}tude", "🇬🇧🇬🇧🇬🇧", "👩‍💻👩‍💻"]
        {
            for max in 0..12 {
                let out = truncate_for_cell(label, max);
                assert!(
                    text_width(&out) <= max,
                    "truncate_for_cell({label:?}, {max}) = {out:?} overflows"
                );
                assert!(
                    label.contains(out.trim_start_matches('…')),
                    "truncate_for_cell({label:?}, {max}) = {out:?} split a cluster"
                );
            }
        }
    }

    #[test]
    fn effective_item_max_len_measures_cells() {
        let labels = vec!["日本語".to_string()];
        assert_eq!(effective_item_max_len(&labels, Some(10)), Some(6));
    }

    #[test]
    fn effective_item_max_len_uses_actual_max() {
        let labels = vec!["a".to_string(), "abc".to_string()];
        assert_eq!(effective_item_max_len(&labels, Some(10)), Some(3));
        assert_eq!(effective_item_max_len(&labels, Some(2)), Some(2));
        assert_eq!(effective_item_max_len(&labels, None), None);
    }

    /// The fast path must be indistinguishable from the segmentation it skips.
    #[test]
    fn ascii_fast_path_agrees_with_segmentation() {
        let ascii: String = (0x20u8..0x7f).map(char::from).collect();
        assert!(is_single_cell_ascii(&ascii));
        assert_eq!(text_width(&ascii), ascii.len());
        assert_eq!(
            ascii.graphemes(true).map(cluster_width).sum::<usize>(),
            ascii.len(),
            "some printable ASCII character is not one cell wide"
        );
        // A tab or newline is not a single cell, so it must take the slow path.
        assert!(!is_single_cell_ascii("a\tb"));
    }
}
