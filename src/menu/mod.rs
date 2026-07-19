pub mod action;
pub mod buffer;
pub mod ls_colors;
pub mod mode;
pub mod tui;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::common;
use crate::complete::{
    self, CompletionMode, ancestors, recents as recents_mode, stack as stack_mode,
};
use crate::frecency::ZoxideProvider;
use crate::resolve::path_query::{PathQuery, QueryKind};
use crate::resolve::precedence;
use crate::resolve::{CompletionCandidates, Resolver};

pub use action::{MenuAction, TerminalGeometry, TerminalState};
pub use buffer::{
    ParsedBuffer, parse_buffer, parse_buffer_with_mode, parse_buffer_with_override_mode,
};
pub use mode::MenuMode;
pub use tui::MenuResult;

/// Source completion candidates for the given mode and query.
/// Built-in dx modes reuse the same pipelines as `dx complete`; mapped
/// filesystem modes use an explicit file-aware directory scan.
/// Duplicates are removed for all modes.
/// The cwd itself is filtered out for non-path-selection modes only.
pub fn source_candidates(
    resolver: &Resolver,
    mode: CompletionMode,
    query: Option<&str>,
    session: Option<&str>,
    cwd: Option<&std::path::Path>,
) -> Vec<PathBuf> {
    source_candidates_with_meta(
        resolver,
        MenuMode::Completion(mode),
        query,
        session,
        cwd,
        None,
    )
    .paths
}

pub fn source_candidates_with_meta(
    resolver: &Resolver,
    mode: MenuMode,
    query: Option<&str>,
    session: Option<&str>,
    cwd: Option<&std::path::Path>,
    limit: Option<usize>,
) -> CompletionCandidates {
    let raw_meta = match mode {
        MenuMode::Completion(CompletionMode::Paths) => resolver
            .collect_completion_candidates_with_limit_and_cwd(query.unwrap_or(""), limit, cwd),
        MenuMode::Completion(CompletionMode::Ancestors) => {
            apply_limit_with_has_more(ancestors::complete(query), limit)
        }
        MenuMode::Completion(CompletionMode::Frecents) => {
            let provider = ZoxideProvider::default();
            apply_limit_with_has_more(complete::complete_frecents(&provider, query), limit)
        }
        MenuMode::Completion(CompletionMode::Recents) => {
            apply_limit_with_has_more(recents_mode::complete(session, query), limit)
        }
        MenuMode::Completion(CompletionMode::Stack(direction)) => {
            apply_limit_with_has_more(stack_mode::complete(session, direction, query), limit)
        }
        MenuMode::Path | MenuMode::Directory | MenuMode::File => {
            source_mapped_filesystem_candidates(resolver, query, cwd, limit, mode)
        }
    };

    let canonical_cwd = match mode {
        MenuMode::Completion(CompletionMode::Paths)
        | MenuMode::Path
        | MenuMode::Directory
        | MenuMode::File => None,
        _ => cwd.and_then(|p| std::fs::canonicalize(p).ok()),
    };

    let mut seen = HashSet::new();
    let mut filtered = Vec::new();

    for p in raw_meta.paths {
        let canonical = std::fs::canonicalize(&p).unwrap_or_else(|_| p.clone());
        if let Some(ref ccwd) = canonical_cwd
            && &canonical == ccwd
        {
            continue;
        }
        if seen.insert(canonical) {
            filtered.push(p);
        }
    }

    CompletionCandidates {
        paths: filtered,
        has_more: raw_meta.has_more,
    }
}

fn source_mapped_filesystem_candidates(
    resolver: &Resolver,
    query: Option<&str>,
    cwd: Option<&Path>,
    limit: Option<usize>,
    mode: MenuMode,
) -> CompletionCandidates {
    let cwd = match cwd {
        Some(path) => path.to_path_buf(),
        None => match std::env::current_dir() {
            Ok(value) => value,
            Err(_) => return CompletionCandidates::empty(),
        },
    };

    let raw_query = query.unwrap_or("");
    let mut combined = Vec::new();
    let mut seen = HashSet::new();

    if !raw_query.is_empty()
        && matches!(mode, MenuMode::Path | MenuMode::Directory)
        && !PathQuery::new(raw_query).is_filesystem_prefix()
    {
        let smart_dirs = resolver.collect_completion_candidates_with_limit_and_cwd(
            raw_query,
            None,
            Some(cwd.as_path()),
        );

        for path in smart_dirs.paths {
            if mode == MenuMode::Directory && !path.is_dir() {
                continue;
            }
            push_unique_path(&mut combined, &mut seen, path);
        }
    }

    let (parents, leaf_prefix) = mapped_parent_directories(resolver, &cwd, raw_query);
    for parent in parents {
        for child in list_filesystem_children(&parent, &leaf_prefix, mode) {
            push_unique_path(&mut combined, &mut seen, child);
        }
    }

    apply_limit_with_has_more(combined, limit)
}

fn mapped_parent_directories(
    resolver: &Resolver,
    cwd: &Path,
    query: &str,
) -> (Vec<PathBuf>, String) {
    if query.is_empty() {
        return (vec![cwd.to_path_buf()], String::new());
    }

    let path_query = PathQuery::new(query);
    if path_query.kind == QueryKind::DriveRelative {
        return (Vec::new(), String::new());
    }
    let query_path = Path::new(query);
    let (parent_query, leaf_prefix) = if path_query.has_trailing_separator() {
        (Some(query_path), String::new())
    } else {
        let parent = query_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty());
        let leaf = query_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        (parent, leaf)
    };
    let Some(parent_query) = parent_query else {
        return (vec![cwd.to_path_buf()], leaf_prefix);
    };
    let Some(parent_query) = parent_query.to_str() else {
        return (Vec::new(), leaf_prefix);
    };

    if !PathQuery::new(parent_query).is_filesystem_prefix() {
        let smart_dirs = resolver.collect_completion_candidates_with_limit_and_cwd(
            parent_query,
            None,
            Some(cwd),
        );
        if !smart_dirs.paths.is_empty() {
            return (smart_dirs.paths, leaf_prefix);
        }
    }

    let fallback = expand_query_path(cwd, PathQuery::new(parent_query))
        .filter(|path| path.is_dir())
        .into_iter()
        .collect::<Vec<_>>();

    (fallback, leaf_prefix)
}

fn list_filesystem_children(parent: &Path, leaf_prefix: &str, mode: MenuMode) -> Vec<PathBuf> {
    let entries = match std::fs::read_dir(parent) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let prefix_lower = leaf_prefix.to_ascii_lowercase();
    let mut results = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !leaf_prefix.is_empty() && !name.to_ascii_lowercase().starts_with(&prefix_lower) {
            continue;
        }

        let path = entry.path();
        let keep = match mode {
            MenuMode::Path => true,
            MenuMode::Directory => path.is_dir(),
            MenuMode::File => path.is_file(),
            MenuMode::Completion(_) => true,
        };
        if keep {
            results.push(path);
        }
    }

    sort_filesystem_candidates_by_basename(&mut results);
    results
}

fn expand_query_path(cwd: &Path, query: PathQuery<'_>) -> Option<PathBuf> {
    precedence::resolve_direct(cwd, query).ok().flatten()
}

fn sort_filesystem_candidates_by_basename(results: &mut [PathBuf]) {
    results.sort_by(|left, right| {
        let left_name = left
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_else(|| left.as_os_str().to_string_lossy());
        let right_name = right
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_else(|| right.as_os_str().to_string_lossy());

        left_name
            .to_ascii_lowercase()
            .cmp(&right_name.to_ascii_lowercase())
            .then_with(|| left_name.cmp(&right_name))
            .then_with(|| left.as_os_str().cmp(right.as_os_str()))
    });
}

fn push_unique_path(output: &mut Vec<PathBuf>, seen: &mut HashSet<PathBuf>, path: PathBuf) {
    let canonical = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
    if seen.insert(canonical) {
        output.push(path);
    }
}

fn apply_limit_with_has_more(paths: Vec<PathBuf>, limit: Option<usize>) -> CompletionCandidates {
    let (paths, has_more) = common::truncate_with_has_more(paths, limit);

    CompletionCandidates { paths, has_more }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::complete::CompletionMode;
    use crate::config::AppConfig;
    use crate::resolve::Resolver;
    use crate::test_support;

    use super::{MenuMode, source_candidates, source_candidates_with_meta};

    #[test]
    fn mixed_case_path_order_menu_paths_matches_completion_order() {
        let temp = test_support::temp_dir("menu-source-order-mixed-case-path-order");
        let cwd = temp.path().join("work");
        fs::create_dir_all(&cwd).expect("create cwd");

        fs::create_dir_all(temp.path().join("Calpha")).expect("create Calpha");
        fs::create_dir_all(temp.path().join("cAlpha")).expect("create cAlpha");
        fs::create_dir_all(temp.path().join("cbravo")).expect("create cbravo");

        let resolver = Resolver::with_bookmark_lookup(AppConfig::default(), |_| None);

        let completion = resolver.collect_completion_candidates_with_limit_and_cwd(
            "../c",
            None,
            Some(cwd.as_path()),
        );
        let menu = source_candidates(
            &resolver,
            CompletionMode::Paths,
            Some("../c"),
            None,
            Some(cwd.as_path()),
        );

        assert_eq!(menu, completion.paths);
    }

    #[cfg(unix)]
    #[test]
    fn mapped_path_root_slash_lists_root_without_cwd_children() {
        let temp = test_support::temp_dir("menu-source-order-mapped-root-slash");
        let cwd = temp.path().join("work");
        fs::create_dir_all(&cwd).expect("create cwd");
        let cwd_only = cwd.join("cwd-only-marker");
        fs::write(&cwd_only, "marker").expect("create cwd marker");

        let resolver = Resolver::with_bookmark_lookup(AppConfig::default(), |_| None);
        let candidates = source_candidates_with_meta(
            &resolver,
            MenuMode::Path,
            Some("/"),
            None,
            Some(cwd.as_path()),
            None,
        );

        assert!(
            candidates
                .paths
                .iter()
                .all(|path| path.parent() == Some(PathBuf::from("/").as_path()))
        );
        assert!(!candidates.paths.contains(&cwd_only));
        assert!(!candidates.paths.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn mapped_path_rooted_prefix_filters_root_without_cwd_children() {
        let temp = test_support::temp_dir("menu-source-order-mapped-root-prefix");
        let cwd = temp.path().join("work");
        fs::create_dir_all(&cwd).expect("create cwd");
        let cwd_only = cwd.join("Users-local-marker");
        fs::write(&cwd_only, "marker").expect("create cwd marker");

        let resolver = Resolver::with_bookmark_lookup(AppConfig::default(), |_| None);
        let candidates = source_candidates_with_meta(
            &resolver,
            MenuMode::Path,
            Some("/U"),
            None,
            Some(cwd.as_path()),
            None,
        );

        assert!(
            candidates
                .paths
                .iter()
                .all(|path| path.parent() == Some(PathBuf::from("/").as_path()))
        );
        assert!(candidates.paths.iter().all(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_ascii_lowercase().starts_with('u'))
                .unwrap_or(false)
        }));
        assert!(!candidates.paths.contains(&cwd_only));
    }

    #[test]
    fn mapped_path_empty_query_still_lists_cwd_children() {
        let temp = test_support::temp_dir("menu-source-order-mapped-empty-cwd");
        let cwd = temp.path().join("work");
        fs::create_dir_all(&cwd).expect("create cwd");
        let cwd_only = cwd.join("cwd-only-marker");
        fs::write(&cwd_only, "marker").expect("create cwd marker");

        let resolver = Resolver::with_bookmark_lookup(AppConfig::default(), |_| None);
        let candidates = source_candidates_with_meta(
            &resolver,
            MenuMode::Path,
            Some(""),
            None,
            Some(cwd.as_path()),
            None,
        );

        assert!(candidates.paths.contains(&cwd_only));
    }

    #[test]
    fn mapped_path_bare_query_still_filters_cwd_children() {
        let temp = test_support::temp_dir("menu-source-order-mapped-bare-cwd");
        let cwd = temp.path().join("work");
        fs::create_dir_all(&cwd).expect("create cwd");
        let matching = cwd.join("src-local-marker");
        let nonmatching = cwd.join("other-marker");
        fs::write(&matching, "marker").expect("create matching marker");
        fs::write(&nonmatching, "marker").expect("create nonmatching marker");

        let resolver = Resolver::with_bookmark_lookup(AppConfig::default(), |_| None);
        let candidates = source_candidates_with_meta(
            &resolver,
            MenuMode::Path,
            Some("src"),
            None,
            Some(cwd.as_path()),
            None,
        );

        assert!(candidates.paths.contains(&matching));
        assert!(!candidates.paths.contains(&nonmatching));
    }

    #[cfg(unix)]
    #[test]
    fn mapped_path_query_preserves_whitespace() {
        let temp = test_support::temp_dir("menu-mapped-whitespace");
        let cwd = temp.path().join("work");
        let child = cwd.join(" project ").join("source");
        fs::create_dir_all(&child).expect("create whitespace path");

        let resolver = Resolver::with_bookmark_lookup(AppConfig::default(), |_| None);
        let candidates = source_candidates_with_meta(
            &resolver,
            MenuMode::Path,
            Some(" project /s"),
            None,
            Some(cwd.as_path()),
            None,
        );

        assert!(candidates.paths.contains(&child));
    }

    #[cfg(windows)]
    #[test]
    fn mapped_path_query_accepts_backslash_separator() {
        let temp = test_support::temp_dir("menu-mapped-backslash");
        let cwd = temp.path().join("work");
        let child = cwd.join("project").join("source");
        fs::create_dir_all(&child).expect("create nested path");

        let resolver = Resolver::with_bookmark_lookup(AppConfig::default(), |_| None);
        let candidates = source_candidates_with_meta(
            &resolver,
            MenuMode::Path,
            Some(r"project\s"),
            None,
            Some(cwd.as_path()),
            None,
        );

        assert!(candidates.paths.contains(&child));
    }
}
