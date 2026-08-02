//! Collecting every candidate a query could mean, rather than insisting on one.
//! Shares the resolution pipeline with `dx resolve`, but tolerates ambiguity.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::common;

use super::{
    CompletionCandidates, FilesystemPrefixFallback, Resolver, path_query::PathQuery, precedence,
    prepare_candidates, prepare_search_query, resolve_search_candidates, traversal,
};

impl Resolver {
    pub fn collect_completion_candidates(&self, raw_query: &str) -> Vec<PathBuf> {
        self.collect_completion_candidates_with_meta(raw_query)
            .paths
    }

    pub fn collect_completion_candidates_with_limit_and_cwd(
        &self,
        raw_query: &str,
        limit: Option<usize>,
        cwd: Option<&Path>,
    ) -> CompletionCandidates {
        self.collect_completion_candidates_impl(raw_query, limit, cwd)
    }

    pub fn collect_completion_candidates_with_meta(&self, raw_query: &str) -> CompletionCandidates {
        self.collect_completion_candidates_impl(raw_query, None, None)
    }

    fn collect_completion_candidates_impl(
        &self,
        raw_query: &str,
        limit: Option<usize>,
        cwd: Option<&Path>,
    ) -> CompletionCandidates {
        if raw_query.is_empty() {
            return CompletionCandidates::empty();
        }
        let query = PathQuery::new(raw_query);
        let explicit_filesystem_prefix = query.is_filesystem_prefix();

        let effective_cwd = match cwd {
            Some(path) => path.to_path_buf(),
            None => match std::env::current_dir() {
                Ok(value) => value,
                Err(_) => return CompletionCandidates::empty(),
            },
        };

        let mut output = Vec::new();
        let mut seen = HashSet::new();

        // Filesystem prefix expansion: when the query looks like a rooted or
        // relative filesystem path prefix, readdir the parent and return
        // matching children. Covers: /abs/pre, ~/pre, ./pre, ../pre.
        if query.is_filesystem_prefix() {
            let candidates = expand_filesystem_prefix(&effective_cwd, query);
            for path in candidates {
                common::push_unique(&mut output, &mut seen, path);
            }

            // For explicit filesystem-prefix queries, prefer filesystem-derived
            // results when present.
            if !output.is_empty() {
                return CompletionCandidates::limited(output, limit);
            }
        }

        let prepared = match prepare_search_query(
            &effective_cwd,
            &self.search_roots,
            raw_query,
            FilesystemPrefixFallback::AlwaysForFilesystemPrefix,
        ) {
            Ok(value) => value,
            // Every failure yields what was collected so far. Listed exhaustively
            // rather than `Err(_)` so a new variant has to be considered here.
            Err(
                super::ResolveError::EmptyQuery
                | super::ResolveError::PathNotFound(_)
                | super::ResolveError::Ambiguous { .. }
                | super::ResolveError::NotFound
                | super::ResolveError::DriveRelativePath(_)
                | super::ResolveError::Filesystem { .. },
            ) => {
                return CompletionCandidates::limited(output, limit);
            }
        };

        if prepared.effective_query.is_empty() {
            return CompletionCandidates::limited(output, limit);
        }

        let probe_limit = limit.map(|value| value.saturating_add(1));

        if !explicit_filesystem_prefix
            && let Some(path) = prepared.direct_dir
            && prepared.fallback_policy.allow_direct_injection()
            && path.is_dir()
        {
            common::push_unique(&mut output, &mut seen, path);
        }

        if prepared.fallback_policy.allow_step_up
            && let Some(path) =
                traversal::resolve_step_up(&effective_cwd, &prepared.effective_query)
            && path.is_dir()
        {
            common::push_unique(&mut output, &mut seen, path);
        }

        // Completion would rather show a partial list than nothing, so an
        // unreadable directory is skipped instead of failing the whole query.
        // `Skip` never returns `Err`, hence the default.
        let mut search_candidates = resolve_search_candidates(
            &prepared.fallback_policy.effective_roots,
            &prepared.effective_query,
            self.case_sensitive,
            traversal::OnIoError::Skip,
        )
        .unwrap_or_default();
        prepare_candidates(&mut search_candidates, probe_limit);
        for candidate in search_candidates {
            common::push_unique(&mut output, &mut seen, candidate);
        }

        // Bookmarks come last: a directory the user can see beats a name only
        // they remember. Prefix matching is confined to plain queries, so an
        // explicit filesystem prefix (`./x`, `~/x`) keeps the exact-match
        // behaviour it has always had; a root-anchored query never gets here at
        // all, because its policy forbids bookmark lookup.
        if prepared.fallback_policy.allow_bookmark_lookup {
            if explicit_filesystem_prefix {
                if let Some(path) = self.bookmarks.get(&prepared.effective_query) {
                    common::push_unique(&mut output, &mut seen, path);
                }
            } else {
                for path in self
                    .bookmarks
                    .prefix_matches(&prepared.effective_query, self.case_sensitive)
                {
                    common::push_unique(&mut output, &mut seen, path);
                }
            }
        }

        CompletionCandidates::limited(output, limit)
    }
}

/// Expand a filesystem path prefix by reading the parent directory and
/// returning all subdirectories whose name starts with the final component.
fn expand_filesystem_prefix(cwd: &Path, query: PathQuery<'_>) -> Vec<PathBuf> {
    let Some(path) = expand_query_path(cwd, query) else {
        return Vec::new();
    };

    if let Some(results) = exact_directory_candidates(&path, query.has_trailing_separator()) {
        return results;
    }

    prefix_directory_candidates(&path)
}

fn expand_query_path(cwd: &Path, query: PathQuery<'_>) -> Option<PathBuf> {
    precedence::resolve_direct(cwd, query).ok().flatten()
}

fn exact_directory_candidates(path: &Path, has_trailing_separator: bool) -> Option<Vec<PathBuf>> {
    if !path.is_dir() {
        return None;
    }

    if !has_trailing_separator {
        return Some(vec![path.to_path_buf()]);
    }

    let mut results = child_directory_candidates(path);
    common::sort_by_basename(&mut results);
    Some(results)
}

fn child_directory_candidates(path: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.metadata().map(|m| m.is_dir()).unwrap_or(false) {
                results.push(entry.path());
            }
        }
    }
    results
}

fn prefix_directory_candidates(path: &Path) -> Vec<PathBuf> {
    let parent = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => return Vec::new(),
    };
    let prefix = match path.file_name() {
        Some(name) => name.to_string_lossy().to_lowercase(),
        None => return Vec::new(),
    };

    let mut results = Vec::new();
    let entries = match std::fs::read_dir(parent) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    for entry in entries.flatten() {
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !meta.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.starts_with(&prefix) {
            results.push(entry.path());
        }
    }
    common::sort_by_basename(&mut results);
    results
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{config::AppConfig, test_support};

    use super::*;

    fn create_resolver_with_roots_and_bookmarks(roots: Vec<PathBuf>) -> Resolver {
        create_resolver_with_roots_and_bookmarks_and_case_sensitivity(roots, true)
    }

    fn create_resolver_with_roots_and_bookmarks_and_case_sensitivity(
        roots: Vec<PathBuf>,
        case_sensitive: bool,
    ) -> Resolver {
        Resolver::from_config(&AppConfig {
            search_roots: roots,
            resolve: crate::config::ResolveOptions { case_sensitive },
            ..AppConfig::default()
        })
    }

    /// Writes a bookmark store and points `DX_BOOKMARKS_FILE` at it, returning
    /// each canonical target in the order given.
    fn write_bookmarks(
        process: &mut test_support::ScopedProcess,
        dir: &Path,
        entries: &[(&str, &Path)],
    ) -> Vec<PathBuf> {
        let mut body = String::from("[bookmarks]\n");
        let mut targets = Vec::new();
        for (name, path) in entries {
            let canonical = fs::canonicalize(path).expect("canonical bookmark target");
            body.push_str(&format!(
                "{name} = \"{}\"\n",
                canonical.display().to_string().replace('\\', "\\\\")
            ));
            targets.push(canonical);
        }

        let store_file = dir.join("bookmarks.toml");
        fs::write(&store_file, body).expect("write bookmarks file");
        process.set("DX_BOOKMARKS_FILE", &store_file);
        targets
    }

    #[test]
    fn completion_offers_bookmark_names_by_prefix() {
        let temp = test_support::temp_dir("complete-bookmark-prefix");
        let mut process = test_support::ScopedProcess::new();
        let cwd = temp.path().join("cwd");
        let target = temp.path().join("acme");
        fs::create_dir_all(&cwd).expect("create cwd");
        fs::create_dir_all(&target).expect("create target");
        let targets = write_bookmarks(&mut process, temp.path(), &[("work", &target)]);

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let out = resolver.collect_completion_candidates_with_limit_and_cwd("wo", None, Some(&cwd));

        assert_eq!(out.paths, targets);
    }

    #[test]
    fn completion_excludes_stale_bookmarks_from_prefix_matches() {
        let temp = test_support::temp_dir("complete-bookmark-stale");
        let mut process = test_support::ScopedProcess::new();
        let cwd = temp.path().join("cwd");
        let target = temp.path().join("acme");
        fs::create_dir_all(&cwd).expect("create cwd");
        fs::create_dir_all(&target).expect("create target");
        let _ = write_bookmarks(&mut process, temp.path(), &[("work", &target)]);
        fs::remove_dir_all(&target).expect("remove bookmark target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let out = resolver.collect_completion_candidates_with_limit_and_cwd("wo", None, Some(&cwd));

        assert!(out.paths.is_empty());
    }

    #[test]
    fn completion_ranks_bookmarks_after_filesystem_candidates() {
        let temp = test_support::temp_dir("complete-bookmark-rank");
        let mut process = test_support::ScopedProcess::new();
        let cwd = temp.path().join("cwd");
        let on_disk = cwd.join("workspace");
        let bookmarked = temp.path().join("acme");
        fs::create_dir_all(&on_disk).expect("create on-disk match");
        fs::create_dir_all(&bookmarked).expect("create bookmark target");
        let targets = write_bookmarks(&mut process, temp.path(), &[("work", &bookmarked)]);

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let out = resolver.collect_completion_candidates_with_limit_and_cwd("wo", None, Some(&cwd));

        // A directory the user can see beats a name only they remember.
        assert_eq!(out.paths.len(), 2);
        assert!(out.paths[0].ends_with("workspace"));
        assert_eq!(out.paths[1], targets[0]);
    }

    #[test]
    fn filesystem_prefix_query_does_not_prefix_match_bookmarks() {
        let temp = test_support::temp_dir("complete-bookmark-fs-prefix");
        let mut process = test_support::ScopedProcess::new();
        let cwd = temp.path().join("cwd");
        let target = temp.path().join("acme");
        fs::create_dir_all(&cwd).expect("create cwd");
        fs::create_dir_all(&target).expect("create target");
        let targets = write_bookmarks(&mut process, temp.path(), &[("work", &target)]);

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);

        // An explicit filesystem prefix keeps the exact-match behaviour it has
        // always had, rather than gaining prefix matching.
        let prefix =
            resolver.collect_completion_candidates_with_limit_and_cwd("./wo", None, Some(&cwd));
        assert!(prefix.paths.is_empty());

        let exact =
            resolver.collect_completion_candidates_with_limit_and_cwd("./work", None, Some(&cwd));
        assert_eq!(exact.paths, targets);
    }

    #[test]
    fn root_anchored_query_offers_no_bookmark_candidates() {
        let temp = test_support::temp_dir("complete-bookmark-root-anchored");
        let mut process = test_support::ScopedProcess::new();
        let cwd = temp.path().join("cwd");
        let target = temp.path().join("acme");
        fs::create_dir_all(&cwd).expect("create cwd");
        fs::create_dir_all(&target).expect("create target");
        let _ = write_bookmarks(&mut process, temp.path(), &[("work", &target)]);

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = format!("/dx-absent-{}/wo", std::process::id());
        let out =
            resolver.collect_completion_candidates_with_limit_and_cwd(&query, None, Some(&cwd));

        assert!(out.paths.is_empty());
    }

    #[test]
    fn completion_dot_slash_lists_children_when_present() {
        let temp = test_support::temp_dir("complete-dot-slash-children");
        let mut process = test_support::ScopedProcess::new();
        let child = temp.path().join("alpha");
        fs::create_dir_all(&child).expect("create child");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        process.set_current_dir(temp.path());

        let out = resolver.collect_completion_candidates("./");

        assert!(out.iter().any(|p| p.ends_with("alpha")));
    }

    #[test]
    fn completion_dot_slash_empty_dir_returns_empty() {
        let temp = test_support::temp_dir("complete-dot-slash-empty");
        let mut process = test_support::ScopedProcess::new();
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);

        process.set_current_dir(temp.path());
        let out = resolver.collect_completion_candidates("./");

        assert!(out.is_empty());
    }

    #[test]
    fn completion_leading_slash_empty_filesystem_falls_back_from_filesystem_root() {
        let temp = test_support::temp_dir("complete-leading-slash-root");
        let _process = test_support::ScopedProcess::new();
        let canonical_temp = fs::canonicalize(temp.path()).expect("canonical temp dir");
        let missing_prefix = format!("dx-miss-{}", std::process::id());
        let target = canonical_temp.join(&missing_prefix).join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = format!("{}/{}/pro", canonical_temp.display(), missing_prefix);
        let out = resolver.collect_completion_candidates(&query);

        assert!(out.contains(&target));
    }

    #[test]
    fn completion_dot_slash_empty_filesystem_falls_back_to_abbreviation() {
        let temp = test_support::temp_dir("complete-dot-slash-fallback");
        let mut process = test_support::ScopedProcess::new();
        let root = temp.path().join("root");
        let missing_prefix = "no-local-hit";
        let target = root.join(missing_prefix).join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        process.set_current_dir(temp.path());

        let out = resolver.collect_completion_candidates("./no-local-hit/pro");

        assert!(out.contains(&target));
    }

    #[test]
    fn completion_tilde_slash_empty_filesystem_falls_back_to_abbreviation() {
        let temp = test_support::temp_dir("complete-tilde-slash-fallback");
        let mut process = test_support::ScopedProcess::new();
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("create home");

        let root = temp.path().join("root");
        let missing_prefix = "no-home-hit";
        let target = root.join(missing_prefix).join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        process.set("HOME", &home);

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        let out = resolver.collect_completion_candidates("~/no-home-hit/pro");

        assert!(out.contains(&target));
    }

    #[test]
    fn completion_matches_resolve_for_dot_slash_prefix_fallback_target() {
        let temp = test_support::temp_dir("complete-resolve-prefix-parity");
        let _process = test_support::ScopedProcess::new();
        let root = temp.path().join("root");
        let target = root.join("no-local-hit").join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);

        let resolved = resolver
            .resolve(super::super::ResolveQuery {
                raw: "./no-local-hit/pro",
                cwd: temp.path(),
            })
            .expect("resolve should succeed");
        let completed = resolver.collect_completion_candidates_with_limit_and_cwd(
            "./no-local-hit/pro",
            None,
            Some(temp.path()),
        );

        assert!(completed.paths.contains(&resolved.path));
    }

    #[test]
    fn mixed_case_path_order_for_filesystem_prefix_and_filtered_siblings() {
        let temp = test_support::temp_dir("complete-mixed-case-order");
        let mut process = test_support::ScopedProcess::new();
        let cwd = temp.path().join("work");
        fs::create_dir_all(&cwd).expect("create cwd");

        let code = temp.path().join("Code");
        let cobalt = temp.path().join("cobalt");
        let cbravo = temp.path().join("cbravo");
        fs::create_dir_all(&code).expect("create Code");
        fs::create_dir_all(&cobalt).expect("create cobalt");
        fs::create_dir_all(&cbravo).expect("create cbravo");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        process.set_current_dir(&cwd);

        let siblings = resolver.collect_completion_candidates("../");
        let filtered = resolver.collect_completion_candidates("../c");

        let sibling_names = siblings
            .iter()
            .filter_map(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().to_string())
            })
            .collect::<Vec<_>>();
        let filtered_names = filtered
            .iter()
            .filter_map(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().to_string())
            })
            .collect::<Vec<_>>();

        let expected = [
            "cbravo".to_string(),
            "cobalt".to_string(),
            "Code".to_string(),
        ];
        let expected_prefix = &expected[..];

        assert!(
            sibling_names.len() >= expected_prefix.len(),
            "expected at least {} sibling entries, got {:?}",
            expected_prefix.len(),
            sibling_names
        );
        assert_eq!(&sibling_names[..expected_prefix.len()], expected_prefix);
        assert_eq!(filtered_names, expected_prefix);
    }

    #[test]
    fn mixed_case_path_order_tie_breaks_are_deterministic() {
        let mut results = vec![
            PathBuf::from("/tmp/cAlpha"),
            PathBuf::from("/tmp/Calpha"),
            PathBuf::from("/tmp/cbravo"),
        ];

        common::sort_by_basename(&mut results);

        let ordered = results
            .iter()
            .map(|path| {
                path.file_name()
                    .expect("basename")
                    .to_string_lossy()
                    .to_string()
            })
            .collect::<Vec<_>>();

        assert_eq!(ordered, vec!["Calpha", "cAlpha", "cbravo"]);
    }

    #[test]
    fn completion_returns_delimiter_aware_matches() {
        let temp = test_support::temp_dir("complete-delimiter-aware");
        let root = temp.path().join("root");
        let target = root.join("cd-extras");
        fs::create_dir_all(&target).expect("create target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        let out = resolver.collect_completion_candidates("cd-e");

        assert_eq!(out, vec![target]);
    }

    #[test]
    fn completion_returns_doubled_period_matches() {
        let temp = test_support::temp_dir("complete-gap-aware");
        let root = temp.path().join("root");
        let target = root.join("PowerShell");
        fs::create_dir_all(&target).expect("create target");

        let resolver =
            create_resolver_with_roots_and_bookmarks_and_case_sensitivity(vec![root], false);
        let out = resolver.collect_completion_candidates("p..shell");

        assert_eq!(out, vec![target]);
    }

    #[test]
    fn completion_preserves_whitespace_in_filesystem_prefixes() {
        let temp = test_support::temp_dir("complete-whitespace-prefix");
        let target = temp.path().join(" project ");
        fs::create_dir_all(&target).expect("create target");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);

        let out = resolver.collect_completion_candidates_with_limit_and_cwd(
            target.to_str().expect("UTF-8 temp path"),
            None,
            Some(temp.path()),
        );

        assert_eq!(out.paths, vec![target]);
    }

    #[cfg(unix)]
    #[test]
    fn completion_deduplication_keeps_distinct_non_utf_paths() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let first = PathBuf::from(OsString::from_vec(b"/tmp/path-\x80".to_vec()));
        let second = PathBuf::from(OsString::from_vec(b"/tmp/path-\x81".to_vec()));
        let mut output = Vec::new();
        let mut seen = HashSet::new();

        common::push_unique(&mut output, &mut seen, first.clone());
        common::push_unique(&mut output, &mut seen, second.clone());

        assert_eq!(output, vec![first, second]);
    }

    #[cfg(unix)]
    #[test]
    fn completion_skips_invalid_sibling_and_keeps_available_match() {
        use std::os::unix::fs::symlink;

        let temp = test_support::temp_dir("complete-invalid-sibling");
        let available = temp.path().join("available");
        fs::create_dir_all(&available).expect("create available sibling");
        symlink(temp.path().join("missing"), temp.path().join("absent"))
            .expect("create dangling sibling");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = format!("{}{}a", temp.path().display(), std::path::MAIN_SEPARATOR);

        let out = resolver.collect_completion_candidates_with_limit_and_cwd(
            &query,
            None,
            Some(temp.path()),
        );

        assert_eq!(out.paths, vec![available]);
    }

    #[cfg(windows)]
    #[test]
    fn windows_trailing_separator_lists_children() {
        let temp = test_support::temp_dir("complete-windows-trailing-separator");
        let parent = temp.path().join("parent");
        let child = parent.join("child");
        fs::create_dir_all(&child).expect("create child");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = format!("{}\\", parent.display());

        let out = resolver.collect_completion_candidates_with_limit_and_cwd(
            &query,
            None,
            Some(temp.path()),
        );

        assert_eq!(out.paths, vec![child]);
    }
}
