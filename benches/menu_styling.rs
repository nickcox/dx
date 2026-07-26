//! Measures the per-candidate cost of LS_COLORS styling, which the menu pays
//! for every rendered row on every keystroke.

use std::path::PathBuf;
use std::time::Instant;

use dx::menu::ls_colors::{LsColorsConfig, parse_ls_colors};

/// A stand-in for a populated LS_COLORS: the usual file-type entries plus a
/// long tail of extensions, which is what makes the suffix scan expensive.
fn sample_ls_colors(extension_count: usize) -> String {
    let mut raw = String::from("di=01;34:ln=01;36:so=01;35:pi=33:ex=01;32:");
    for index in 0..extension_count {
        raw.push_str(&format!("*.ext{index}=01;31:"));
    }
    raw.push_str("*.rs=01;33");
    raw
}

fn sample_paths(limit: usize) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut stack = vec![PathBuf::from(".")];
    while let Some(dir) = stack.pop() {
        if paths.len() >= limit {
            break;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path.clone());
            }
            paths.push(path);
            if paths.len() >= limit {
                break;
            }
        }
    }
    paths
}

fn time_styling(config: &LsColorsConfig, paths: &[PathBuf], rounds: u32) -> f64 {
    // Warm the filesystem cache so the measurement reflects steady-state typing.
    for path in paths {
        let _ = config.style_for_path(path);
    }

    let started = Instant::now();
    for _ in 0..rounds {
        for path in paths {
            std::hint::black_box(config.style_for_path(path));
        }
    }
    started.elapsed().as_secs_f64() * 1000.0 / f64::from(rounds)
}

fn main() {
    let paths = sample_paths(2400);
    println!("candidates: {}", paths.len());

    // A 20-row menu draws at most 22 rows, so this is what a frame costs once
    // styling is limited to the visible window.
    let window: Vec<_> = paths.iter().take(22).cloned().collect();

    for extension_count in [0usize, 50, 315] {
        let config = parse_ls_colors(&sample_ls_colors(extension_count));
        let all = time_styling(&config, &paths, 20);
        let visible = time_styling(&config, &window, 200);
        println!(
            "  {extension_count:>3} extension entries: all {} candidates {all:7.2} ms  |  \
             visible window (22) {visible:6.3} ms",
            paths.len()
        );
    }
}
