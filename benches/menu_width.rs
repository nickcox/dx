//! Every keystroke measures every candidate label: `effective_item_max_len`
//! takes the widest of up to `max_results` labels, and the render pass measures
//! each visible cell again.
//!
//! Sizes what measuring grapheme-cluster widths costs against counting `chars`,
//! and whether the ASCII fast path earns its place.

use std::time::Instant;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// The old, wrong measurement, for scale.
fn width_chars(input: &str) -> usize {
    input.chars().count()
}

/// Correct, with no shortcut.
fn width_graphemes(input: &str) -> usize {
    input
        .graphemes(true)
        .map(UnicodeWidthStr::width)
        .sum::<usize>()
}

/// Correct, skipping segmentation when every byte is one cell — what ships.
fn width_fast_ascii(input: &str) -> usize {
    if input
        .bytes()
        .all(|byte| byte.is_ascii_graphic() || byte == b' ')
    {
        return input.len();
    }
    width_graphemes(input)
}

fn time(label: &str, rounds: u32, mut body: impl FnMut()) -> f64 {
    body();
    let mut best = f64::MAX;
    for _ in 0..5 {
        let started = Instant::now();
        for _ in 0..rounds {
            body();
        }
        let per_round = started.elapsed().as_secs_f64() * 1000.0 / f64::from(rounds);
        best = best.min(per_round);
    }
    println!("  {label:<38} {best:7.4} ms per keystroke");
    best
}

fn report(name: &str, labels: &[String]) {
    let bytes: usize = labels.iter().map(String::len).sum();
    println!("\n{name}: {} labels, {bytes} bytes", labels.len());

    let chars = time("chars().count() (wrong)", 200, || {
        for label in labels {
            std::hint::black_box(width_chars(label));
        }
    });
    let graphemes = time("grapheme clusters", 200, || {
        for label in labels {
            std::hint::black_box(width_graphemes(label));
        }
    });
    let fast = time("grapheme clusters + ASCII fast path", 200, || {
        for label in labels {
            std::hint::black_box(width_fast_ascii(label));
        }
    });

    println!(
        "  correct measurement costs {:.1}x the wrong one",
        graphemes / chars
    );
    println!("  fast path recovers {:.1}x of that", graphemes / fast);
}

fn main() {
    // Realistic worst case: a full `max_results` page of paths.
    let ascii: Vec<String> = (0..1000)
        .map(|i| format!("crates/some-crate-{i}/src/module/submodule/file_name_{i}.rs"))
        .collect();
    report("plain ASCII paths", &ascii);

    let wide: Vec<String> = (0..1000)
        .map(|i| format!("プロジェクト/ソースコード/モジュール-{i}/ファイル名.rs"))
        .collect();
    report("CJK paths", &wide);

    let mixed: Vec<String> = ascii
        .iter()
        .zip(&wide)
        .enumerate()
        .map(|(i, (a, w))| if i % 20 == 0 { w.clone() } else { a.clone() })
        .collect();
    report("mixed (1 in 20 wide)", &mixed);
}
