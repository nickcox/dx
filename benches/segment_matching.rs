//! Upper bound on what 6a could save: the whole cost of `matches_segment`,
//! which rebuilds a `ParsedSegment` for every directory entry scanned.

use std::time::Instant;

use dx::resolve::abbreviation::matches_segment;

fn names(count: usize) -> Vec<String> {
    (0..count).map(|index| format!("entry{index:04}")).collect()
}

fn time(label: &str, segment: &str, case_sensitive: bool, names: &[String], rounds: u32) {
    for name in names {
        std::hint::black_box(matches_segment(name, segment, case_sensitive));
    }

    let started = Instant::now();
    for _ in 0..rounds {
        for name in names {
            std::hint::black_box(matches_segment(name, segment, case_sensitive));
        }
    }
    let per_scan = started.elapsed().as_secs_f64() * 1000.0 / f64::from(rounds);
    println!(
        "  {label:<34} {per_scan:6.3} ms per {}-entry directory scan",
        names.len()
    );
}

fn main() {
    let names = names(2000);

    // No `.`/`_`/`-`, so no tokenizing: one String allocation per entry.
    time("plain prefix, case-sensitive", "ent", true, &names, 200);
    time("plain prefix, case-insensitive", "ent", false, &names, 200);
    // Operator segments also build a Vec<SegmentToken> of Strings per entry.
    time("delimiter operator (cd-e)", "cd-e", true, &names, 200);
    time("gap operator (p..shell)", "p..shell", false, &names, 200);
}
