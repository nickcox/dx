//! The menu recomputes a label for every candidate on every keystroke.
//!
//! `cwd_relative_label_for_display` falls back to canonicalising when the
//! candidate is not under the cwd — which is the normal case for search-root
//! hits. This replicates that fallback with the same public helpers the menu
//! uses, to size the cost.

use std::path::{Path, PathBuf};
use std::time::Instant;

use dx::complete::cwd_relative_label;

fn under_cwd(paths: &[PathBuf], cwd: &Path) -> usize {
    paths
        .iter()
        .filter(|path| path.strip_prefix(cwd).is_ok())
        .count()
}

/// Mirrors `menu::tui::cwd_relative_label_for_display`.
fn label_with_fallback(path: &Path, cwd: &Path) -> Option<String> {
    cwd_relative_label(path, cwd, false).or_else(|| {
        let path = std::fs::canonicalize(path).ok()?;
        let cwd = std::fs::canonicalize(cwd).ok()?;
        cwd_relative_label(&path, &cwd, false)
    })
}

/// The same, but with the cwd canonicalised once rather than per candidate.
fn label_with_hoisted_cwd(path: &Path, cwd: &Path, canonical_cwd: Option<&Path>) -> Option<String> {
    cwd_relative_label(path, cwd, false).or_else(|| {
        let canonical_cwd = canonical_cwd?;
        let path = std::fs::canonicalize(path).ok()?;
        cwd_relative_label(&path, canonical_cwd, false)
    })
}

/// Parent-cached: canonicalise each distinct parent once and reuse it for
/// siblings, which is what `LabelContext` now does.
fn label_with_parent_cache(
    path: &Path,
    cwd: &Path,
    canonical_cwd: Option<&Path>,
    cache: &mut std::collections::HashMap<PathBuf, Option<PathBuf>>,
) -> Option<String> {
    if let Some(label) = cwd_relative_label(path, cwd, false) {
        return Some(label);
    }
    let parent = path.parent()?;
    let name = path.file_name()?;
    let canonical_parent = match cache.get(parent) {
        Some(cached) => cached.clone(),
        None => {
            let resolved = std::fs::canonicalize(parent).ok();
            cache.insert(parent.to_path_buf(), resolved.clone());
            resolved
        }
    };
    cwd_relative_label(&canonical_parent?.join(name), canonical_cwd?, false)
}

/// Lexical only: compare against the canonical cwd instead of canonicalising
/// every candidate. Loses the case where a candidate reaches the cwd through a
/// symlinked component, which then renders absolute rather than relative.
fn label_lexical_only(path: &Path, cwd: &Path, canonical_cwd: Option<&Path>) -> Option<String> {
    cwd_relative_label(path, cwd, false).or_else(|| cwd_relative_label(path, canonical_cwd?, false))
}

fn time(label: &str, rounds: u32, mut body: impl FnMut()) {
    body();
    let started = Instant::now();
    for _ in 0..rounds {
        body();
    }
    println!(
        "  {label:<44} {:7.2} ms per keystroke",
        started.elapsed().as_secs_f64() * 1000.0 / f64::from(rounds)
    );
}

fn main() {
    let root = std::env::current_dir().expect("cwd");
    let mut paths = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        if paths.len() >= 2000 {
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
            if paths.len() >= 2000 {
                break;
            }
        }
    }

    // A cwd elsewhere on the filesystem, so candidates are not under it and the
    // fallback fires — the normal case when candidates come from search roots.
    let cwd = std::env::temp_dir();
    println!(
        "candidates: {}  ({} under cwd, {} take the fallback)",
        paths.len(),
        under_cwd(&paths, &cwd),
        paths.len() - under_cwd(&paths, &cwd)
    );

    time("current: canonicalise path + cwd each", 20, || {
        for path in &paths {
            std::hint::black_box(label_with_fallback(path, &cwd));
        }
    });

    let canonical_cwd = std::fs::canonicalize(&cwd).ok();
    time("cwd canonicalised once", 20, || {
        for path in &paths {
            std::hint::black_box(label_with_hoisted_cwd(path, &cwd, canonical_cwd.as_deref()));
        }
    });

    let mut cache = std::collections::HashMap::new();
    time("parent cache, cold (first keystroke)", 1, || {
        cache.clear();
        for path in &paths {
            std::hint::black_box(label_with_parent_cache(
                path,
                &cwd,
                canonical_cwd.as_deref(),
                &mut cache,
            ));
        }
    });
    time("parent cache, warm (later keystrokes)", 200, || {
        for path in &paths {
            std::hint::black_box(label_with_parent_cache(
                path,
                &cwd,
                canonical_cwd.as_deref(),
                &mut cache,
            ));
        }
    });
    println!("  distinct parents cached: {}", cache.len());

    time("lexical against canonical cwd only", 200, || {
        for path in &paths {
            std::hint::black_box(label_lexical_only(path, &cwd, canonical_cwd.as_deref()));
        }
    });
}
