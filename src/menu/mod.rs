pub mod action;
pub mod buffer;
pub mod ls_colors;
pub mod mode;
pub mod tui;

use std::collections::HashSet;
use std::path::PathBuf;

use crate::complete::{
    self, CompletionMode, ancestors, recents as recents_mode, stack as stack_mode,
};
use crate::frecency::ZoxideProvider;
use crate::resolve::{CompletionCandidates, Resolver};

pub use action::{MenuAction, TerminalGeometry, TerminalState};
pub use buffer::{
    ParsedBuffer, parse_buffer, parse_buffer_with_mode, parse_buffer_with_override_mode,
};
pub use mode::{MenuMode, QueryStyle};
pub use tui::{MenuOptions, MenuRequest, MenuResult};

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
            CompletionCandidates::limited(ancestors::complete(query), limit)
        }
        MenuMode::Completion(CompletionMode::Frecents) => {
            let provider = ZoxideProvider::default();
            CompletionCandidates::limited(complete::complete_frecents(&provider, query), limit)
        }
        MenuMode::Completion(CompletionMode::Recents) => {
            CompletionCandidates::limited(recents_mode::complete(session, query), limit)
        }
        MenuMode::Completion(CompletionMode::Stack(direction)) => {
            CompletionCandidates::limited(stack_mode::complete(session, direction, query), limit)
        }
        MenuMode::Path | MenuMode::Directory | MenuMode::File => complete::filesystem::complete(
            resolver,
            query,
            cwd,
            limit,
            mode.filesystem_kind()
                .expect("mapped filesystem modes always carry a kind"),
        ),
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
        // Canonicalising costs ~12us per candidate. Only the cwd filter needs
        // it, and the modes that use it yield few candidates; the high-volume
        // filesystem modes leave `canonical_cwd` unset and dedup lexically.
        if let Some(ref ccwd) = canonical_cwd {
            let canonical = std::fs::canonicalize(&p).unwrap_or_else(|_| p.clone());
            if &canonical == ccwd {
                continue;
            }
        }
        if seen.insert(p.clone()) {
            filtered.push(p);
        }
    }

    CompletionCandidates {
        paths: filtered,
        has_more: raw_meta.has_more,
    }
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
