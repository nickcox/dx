use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::common;

use super::{
    CompletionCandidates, FilesystemPrefixFallback, Resolver, prepare_candidates,
    prepare_search_query, resolve_search_candidates, traversal,
};

impl Resolver {
    pub fn collect_completion_candidates(&self, raw_query: &str) -> Vec<PathBuf> {
        self.collect_completion_candidates_with_meta(raw_query).paths
    }

    pub fn collect_completion_candidates_with_limit_and_cwd(
        &self,
        raw_query: &str,
        limit: Option<usize>,
        cwd: Option<&Path>,
    ) -> CompletionCandidates {
        self.collect_completion_candidates_impl(raw_query, limit, cwd)
    }

    pub fn collect_completion_candidates_with_limit(
        &self,
        raw_query: &str,
        limit: Option<usize>,
    ) -> CompletionCandidates {
        self.collect_completion_candidates_impl(raw_query, limit, None)
    }

    pub fn collect_completion_candidates_with_meta(
        &self,
        raw_query: &str,
    ) -> CompletionCandidates {
        self.collect_completion_candidates_impl(raw_query, None, None)
    }

    fn collect_completion_candidates_impl(
        &self,
        raw_query: &str,
        limit: Option<usize>,
        cwd: Option<&Path>,
    ) -> CompletionCandidates {
        let trimmed = raw_query.trim();
        if trimmed.is_empty() {
            return CompletionCandidates::empty();
        }
        let explicit_filesystem_prefix = super::is_filesystem_prefix(trimmed);

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
        if super::is_filesystem_prefix(trimmed) {
            let candidates = expand_filesystem_prefix(&effective_cwd, trimmed);
            for path in candidates {
                push_unique(&mut output, &mut seen, path);
            }

            // For explicit filesystem-prefix queries, prefer filesystem-derived
            // results when present.
            if !output.is_empty() {
                return apply_completion_limit(output, limit);
            }
        }

        let prepared = match prepare_search_query(
            &effective_cwd,
            &self.config.search_roots,
            raw_query,
            FilesystemPrefixFallback::AlwaysForFilesystemPrefix,
        ) {
            Ok(value) => value,
            Err( super::ResolveError::EmptyQuery | super::ResolveError::PathNotFound(_)) => {
                return apply_completion_limit(output, limit);
            }
            Err(super::ResolveError::Ambiguous { .. } | super::ResolveError::NotFound) => {
                return apply_completion_limit(output, limit);
            }
        };

        if prepared.effective_query.is_empty() {
            return apply_completion_limit(output, limit);
        }

        let probe_limit = limit.map(|value| value.saturating_add(1));

        if !explicit_filesystem_prefix
            && let Some(path) = prepared.direct_dir
            && prepared.fallback_policy.allow_direct_injection()
            && path.is_dir()
        {
            push_unique(&mut output, &mut seen, path);
        }

        if prepared.fallback_policy.allow_step_up
            && let Some(path) = traversal::resolve_step_up(&effective_cwd, prepared.effective_query)
            && path.is_dir()
        {
            push_unique(&mut output, &mut seen, path);
        }

        let mut search_candidates = resolve_search_candidates(
            &prepared.fallback_policy.effective_roots,
            prepared.effective_query,
            self.config.resolve.case_sensitive,
        );
        prepare_candidates(&mut search_candidates, probe_limit);
        for candidate in search_candidates {
            push_unique(&mut output, &mut seen, candidate);
        }

        if prepared.fallback_policy.allow_bookmark_lookup
            && let Some(path) = (self.bookmark_lookup)(prepared.effective_query)
        {
            push_unique(&mut output, &mut seen, path);
        }

        apply_completion_limit(output, limit)
    }
}

fn push_unique(output: &mut Vec<PathBuf>, seen: &mut HashSet<PathBuf>, candidate: PathBuf) {
    if seen.insert(candidate.clone()) {
        output.push(candidate);
    }
}

fn apply_completion_limit(paths: Vec<PathBuf>, limit: Option<usize>) -> CompletionCandidates {
    let (paths, has_more) = common::truncate_with_has_more(paths, limit);
    CompletionCandidates { paths, has_more }
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

/// Expand a filesystem path prefix by reading the parent directory and
/// returning all subdirectories whose name starts with the final component.
fn expand_filesystem_prefix(cwd: &Path, query: &str) -> Vec<PathBuf> {
    let Some(path) = expand_query_path(cwd, query) else {
        return Vec::new();
    };

    if let Some(results) = exact_directory_candidates(&path, query) {
        return results;
    }

    prefix_directory_candidates(&path)
}

fn expand_query_path(cwd: &Path, query: &str) -> Option<PathBuf> {
    let expanded = expand_home_prefix(query)?;
    Some(if expanded.starts_with('/') {
        PathBuf::from(expanded.as_ref())
    } else {
        cwd.join(expanded.as_ref())
    })
}

fn expand_home_prefix(query: &str) -> Option<std::borrow::Cow<'_, str>> {
    use std::env;

    if query == "~" {
        env::var("HOME").ok().map(std::borrow::Cow::Owned)
    } else if let Some(rest) = query.strip_prefix("~/") {
        env::var("HOME")
            .ok()
            .map(|home| std::borrow::Cow::Owned(format!("{home}/{rest}")))
    } else {
        Some(std::borrow::Cow::Borrowed(query))
    }
}

fn exact_directory_candidates(path: &Path, query: &str) -> Option<Vec<PathBuf>> {
    if !path.is_dir() {
        return None;
    }

    if !query.ends_with('/') {
        return Some(vec![path.to_path_buf()]);
    }

    let mut results = child_directory_candidates(path);
    sort_filesystem_candidates_by_basename(&mut results);
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
    sort_filesystem_candidates_by_basename(&mut results);
    results
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{bookmarks, config::AppConfig, test_support};

    use super::*;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        test_support::env_lock()
    }

    fn make_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dx-{label}-{nonce}-{}", std::process::id()));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn create_resolver_with_roots_and_bookmarks(roots: Vec<PathBuf>) -> Resolver {
        create_resolver_with_roots_and_bookmarks_and_case_sensitivity(roots, true)
    }

    fn create_resolver_with_roots_and_bookmarks_and_case_sensitivity(
        roots: Vec<PathBuf>,
        case_sensitive: bool,
    ) -> Resolver {
        Resolver::with_bookmark_lookup(
            AppConfig {
                search_roots: roots,
                resolve: crate::config::ResolveOptions { case_sensitive },
                ..AppConfig::default()
            },
            bookmarks::lookup,
        )
    }

    #[test]
    fn completion_dot_slash_lists_children_when_present() {
        let _guard = env_lock();
        let temp = make_temp_dir("complete-dot-slash-children");
        let child = temp.join("alpha");
        fs::create_dir_all(&child).expect("create child");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let prev = std::env::current_dir().expect("read cwd");
        std::env::set_current_dir(&temp).expect("set cwd");

        let out = resolver.collect_completion_candidates("./");

        std::env::set_current_dir(prev).expect("restore cwd");
        assert!(out.iter().any(|p| p.ends_with("alpha")));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn completion_dot_slash_empty_dir_returns_empty() {
        let _guard = env_lock();
        let temp = make_temp_dir("complete-dot-slash-empty");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);

        let prev = std::env::current_dir().expect("read cwd");
        std::env::set_current_dir(&temp).expect("set cwd");
        let out = resolver.collect_completion_candidates("./");
        std::env::set_current_dir(prev).expect("restore cwd");

        assert!(out.is_empty());
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn completion_leading_slash_empty_filesystem_falls_back_from_filesystem_root() {
        let _guard = env_lock();
        let temp = make_temp_dir("complete-leading-slash-root");
        let canonical_temp = fs::canonicalize(&temp).expect("canonical temp dir");
        let missing_prefix = format!("dx-miss-{}", std::process::id());
        let target = canonical_temp.join(&missing_prefix).join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = format!("{}/{}/pro", canonical_temp.display(), missing_prefix);
        let out = resolver.collect_completion_candidates(&query);

        assert!(out.contains(&target));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn completion_dot_slash_empty_filesystem_falls_back_to_abbreviation() {
        let _guard = env_lock();
        let temp = make_temp_dir("complete-dot-slash-fallback");
        let root = temp.join("root");
        let missing_prefix = "no-local-hit";
        let target = root.join(missing_prefix).join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        let prev = std::env::current_dir().expect("read cwd");
        std::env::set_current_dir(&temp).expect("set cwd");

        let out = resolver.collect_completion_candidates("./no-local-hit/pro");

        std::env::set_current_dir(prev).expect("restore cwd");
        assert!(out.contains(&target));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn completion_tilde_slash_empty_filesystem_falls_back_to_abbreviation() {
        let _guard = env_lock();
        let temp = make_temp_dir("complete-tilde-slash-fallback");
        let home = temp.join("home");
        fs::create_dir_all(&home).expect("create home");

        let root = temp.join("root");
        let missing_prefix = "no-home-hit";
        let target = root.join(missing_prefix).join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let prev_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", &home) };

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        let out = resolver.collect_completion_candidates("~/no-home-hit/pro");

        if let Some(value) = prev_home {
            unsafe { std::env::set_var("HOME", value) };
        } else {
            unsafe { std::env::remove_var("HOME") };
        }

        assert!(out.contains(&target));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn completion_matches_resolve_for_dot_slash_prefix_fallback_target() {
        let _guard = env_lock();
        let temp = make_temp_dir("complete-resolve-prefix-parity");
        let root = temp.join("root");
        let target = root.join("no-local-hit").join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);

        let resolved = resolver
            .resolve(super::super::ResolveQuery {
                raw: "./no-local-hit/pro",
                cwd: &temp,
            })
            .expect("resolve should succeed");
        let completed = resolver.collect_completion_candidates_with_limit_and_cwd(
            "./no-local-hit/pro",
            None,
            Some(&temp),
        );

        assert!(completed.paths.contains(&resolved.path));

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn mixed_case_path_order_for_filesystem_prefix_and_filtered_siblings() {
        let _guard = env_lock();
        let temp = make_temp_dir("complete-mixed-case-order");
        let cwd = temp.join("work");
        fs::create_dir_all(&cwd).expect("create cwd");

        let code = temp.join("Code");
        let cobalt = temp.join("cobalt");
        let cbravo = temp.join("cbravo");
        fs::create_dir_all(&code).expect("create Code");
        fs::create_dir_all(&cobalt).expect("create cobalt");
        fs::create_dir_all(&cbravo).expect("create cbravo");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let prev = std::env::current_dir().expect("read cwd");
        std::env::set_current_dir(&cwd).expect("set cwd");

        let siblings = resolver.collect_completion_candidates("../");
        let filtered = resolver.collect_completion_candidates("../c");

        std::env::set_current_dir(prev).expect("restore cwd");

        let sibling_names = siblings
            .iter()
            .filter_map(|path| path.file_name().map(|name| name.to_string_lossy().to_string()))
            .collect::<Vec<_>>();
        let filtered_names = filtered
            .iter()
            .filter_map(|path| path.file_name().map(|name| name.to_string_lossy().to_string()))
            .collect::<Vec<_>>();

        let expected = vec!["cbravo".to_string(), "cobalt".to_string(), "Code".to_string()];
        let expected_prefix = &expected[..];

        assert!(
            sibling_names.len() >= expected_prefix.len(),
            "expected at least {} sibling entries, got {:?}",
            expected_prefix.len(),
            sibling_names
        );
        assert_eq!(&sibling_names[..expected_prefix.len()], expected_prefix);
        assert_eq!(filtered_names, expected_prefix);

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn mixed_case_path_order_tie_breaks_are_deterministic() {
        let mut results = vec![
            PathBuf::from("/tmp/cAlpha"),
            PathBuf::from("/tmp/Calpha"),
            PathBuf::from("/tmp/cbravo"),
        ];

        sort_filesystem_candidates_by_basename(&mut results);

        let ordered = results
            .iter()
            .map(|path| path.file_name().expect("basename").to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert_eq!(ordered, vec!["Calpha", "cAlpha", "cbravo"]);
    }

    #[test]
    fn completion_returns_delimiter_aware_matches() {
        let temp = make_temp_dir("complete-delimiter-aware");
        let root = temp.join("root");
        let target = root.join("cd-extras");
        fs::create_dir_all(&target).expect("create target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        let out = resolver.collect_completion_candidates("cd-e");

        assert_eq!(out, vec![target]);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn completion_returns_doubled_period_matches() {
        let temp = make_temp_dir("complete-gap-aware");
        let root = temp.join("root");
        let target = root.join("PowerShell");
        fs::create_dir_all(&target).expect("create target");

        let resolver = create_resolver_with_roots_and_bookmarks_and_case_sensitivity(vec![root], false);
        let out = resolver.collect_completion_candidates("p..shell");

        assert_eq!(out, vec![target]);
        let _ = fs::remove_dir_all(temp);
    }
}
