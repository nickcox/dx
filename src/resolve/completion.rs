use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::common;

use super::{
    CompletionCandidates, FallbackPolicy, Resolver, abbreviation, is_filesystem_prefix,
    prepare_candidates, roots, strip_filesystem_prefix_for_fallback, traversal,
};

impl Resolver {
    pub fn collect_completion_candidates(&self, raw_query: &str) -> Vec<PathBuf> {
        self.collect_completion_candidates_with_meta(raw_query).paths
    }

    pub fn collect_completion_candidates_with_limit(
        &self,
        raw_query: &str,
        limit: Option<usize>,
    ) -> CompletionCandidates {
        self.collect_completion_candidates_impl(raw_query, limit)
    }

    pub fn collect_completion_candidates_with_meta(
        &self,
        raw_query: &str,
    ) -> CompletionCandidates {
        self.collect_completion_candidates_impl(raw_query, None)
    }

    fn collect_completion_candidates_impl(
        &self,
        raw_query: &str,
        limit: Option<usize>,
    ) -> CompletionCandidates {
        let trimmed = raw_query.trim();
        if trimmed.is_empty() {
            return CompletionCandidates {
                paths: Vec::new(),
                has_more: false,
            };
        }

        let cwd = match std::env::current_dir() {
            Ok(value) => value,
            Err(_) => {
                return CompletionCandidates {
                    paths: Vec::new(),
                    has_more: false,
                }
            }
        };

        let mut output = Vec::new();
        let mut seen = HashSet::new();

        let mut completion_query = trimmed;
        let mut uses_prefix_fallback = false;

        // Filesystem prefix expansion: when the query looks like a rooted or
        // relative filesystem path prefix, readdir the parent and return
        // matching children. Covers: /abs/pre, ~/pre, ./pre, ../pre.
        if is_filesystem_prefix(trimmed) {
            let candidates = expand_filesystem_prefix(&cwd, trimmed);
            for path in candidates {
                push_unique(&mut output, &mut seen, path);
            }

            // For explicit filesystem-prefix queries, prefer filesystem-derived
            // results when present.
            if !output.is_empty() {
                return apply_completion_limit(output, limit);
            }

            // When expansion yields no filesystem matches, continue into
            // abbreviation/fallback completion using a prefix-stripped query.
            completion_query = strip_filesystem_prefix_for_fallback(trimmed);
            uses_prefix_fallback = true;
            if completion_query.is_empty() {
                return apply_completion_limit(output, limit);
            }
        }

        let fallback_policy = FallbackPolicy::from_query_context(
            &cwd,
            &self.config.search_roots,
            trimmed,
            uses_prefix_fallback,
        );

        let probe_limit = limit.map(|value| value.saturating_add(1));

        if fallback_policy.allow_direct_injection()
            && let Some(path) = super::precedence::resolve_direct(&cwd, completion_query)
            && path.is_dir()
        {
            push_unique(&mut output, &mut seen, path);
        }

        if fallback_policy.allow_step_up
            && let Some(path) = traversal::resolve_step_up(&cwd, completion_query)
            && path.is_dir()
        {
            push_unique(&mut output, &mut seen, path);
        }

        let mut abbreviation_candidates = abbreviation::resolve_abbreviation(
            &fallback_policy.effective_roots,
            completion_query,
            self.config.resolve.case_sensitive,
        );
        prepare_candidates(&mut abbreviation_candidates, probe_limit);
        for candidate in abbreviation_candidates {
            push_unique(&mut output, &mut seen, candidate);
        }

        let mut fallback_candidates = roots::resolve_fallbacks(
            &fallback_policy.effective_roots,
            completion_query,
            self.config.resolve.case_sensitive,
        );
        prepare_candidates(&mut fallback_candidates, probe_limit);
        for candidate in fallback_candidates {
            push_unique(&mut output, &mut seen, candidate);
        }

        if fallback_policy.allow_bookmark_lookup
            && let Some(path) = (self.bookmark_lookup)(completion_query)
        {
            push_unique(&mut output, &mut seen, path);
        }

        apply_completion_limit(output, limit)
    }
}

fn push_unique(output: &mut Vec<PathBuf>, seen: &mut HashSet<String>, candidate: PathBuf) {
    let key = candidate.display().to_string();
    if seen.insert(key) {
        output.push(candidate);
    }
}

fn apply_completion_limit(paths: Vec<PathBuf>, limit: Option<usize>) -> CompletionCandidates {
    let (paths, has_more) = common::truncate_with_has_more(paths, limit);
    CompletionCandidates { paths, has_more }
}

/// Expand a filesystem path prefix by reading the parent directory and
/// returning all subdirectories whose name starts with the final component.
fn expand_filesystem_prefix(cwd: &Path, query: &str) -> Vec<PathBuf> {
    use std::env;

    // Resolve home prefix first.
    let expanded: std::borrow::Cow<str> = if query == "~" {
        match env::var("HOME") {
            Ok(home) => std::borrow::Cow::Owned(home),
            Err(_) => return Vec::new(),
        }
    } else if let Some(rest) = query.strip_prefix("~/") {
        match env::var("HOME") {
            Ok(home) => std::borrow::Cow::Owned(format!("{home}/{rest}")),
            Err(_) => return Vec::new(),
        }
    } else {
        std::borrow::Cow::Borrowed(query)
    };

    let path = if expanded.starts_with('/') {
        PathBuf::from(expanded.as_ref())
    } else {
        cwd.join(expanded.as_ref())
    };

    // If the path itself is an existing directory, return it directly.
    if path.is_dir() && !query.ends_with('/') {
        return vec![path];
    }

    // If path is a directory and query ends with '/', list its children.
    if path.is_dir() && query.ends_with('/') {
        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                if entry.metadata().map(|m| m.is_dir()).unwrap_or(false) {
                    results.push(entry.path());
                }
            }
        }
        results.sort();
        return results;
    }

    // Otherwise treat it as a prefix: parent dir + prefix filter.
    let parent = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => return Vec::new(),
    };
    let prefix = match path.file_name() {
        Some(name) => name.to_string_lossy().to_lowercase(),
        None => return Vec::new(),
    };

    let mut results = Vec::new();
    let entries = match std::fs::read_dir(&parent) {
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
        if name.starts_with(&*prefix) {
            results.push(entry.path());
        }
    }
    results.sort();
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
        Resolver::with_bookmark_lookup(
            AppConfig {
                search_roots: roots,
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
}
