pub mod abbreviation;
pub mod precedence;
pub mod roots;
pub mod traversal;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;

use crate::{bookmarks, config::AppConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveMode {
    Default,
    List,
    Json,
}

#[derive(Debug, Clone)]
pub struct ResolveQuery<'a> {
    pub raw: &'a str,
    pub cwd: &'a Path,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveResult {
    pub path: PathBuf,
}

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("query was empty")]
    EmptyQuery,
    #[error("target path does not exist: {0}")]
    PathNotFound(String),
    #[error("query is ambiguous ({count} matches)")]
    Ambiguous {
        candidates: Vec<PathBuf>,
        count: usize,
    },
    #[error("unable to resolve query")]
    NotFound,
}

#[derive(Debug, Serialize)]
struct JsonOutput<'a> {
    status: &'a str,
    reason: Option<&'a str>,
    path: Option<String>,
    candidates: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Resolver {
    pub config: AppConfig,
    bookmark_lookup: fn(&str) -> Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct CompletionCandidates {
    pub paths: Vec<PathBuf>,
    pub has_more: bool,
}

impl Resolver {
    pub fn from_environment() -> Self {
        let config = AppConfig::load().unwrap_or_default();
        Self {
            config,
            bookmark_lookup: bookmarks::lookup,
        }
    }

    pub fn with_bookmark_lookup(
        config: AppConfig,
        bookmark_lookup: fn(&str) -> Option<PathBuf>,
    ) -> Self {
        Self {
            config,
            bookmark_lookup,
        }
    }

    pub fn execute(&self, raw_query: &str, mode: ResolveMode) -> i32 {
        let cwd = match std::env::current_dir() {
            Ok(path) => path,
            Err(err) => {
                eprintln!("dx resolve: failed to read current directory: {err}");
                return 1;
            }
        };

        let query = ResolveQuery {
            raw: raw_query,
            cwd: &cwd,
        };

        match self.resolve(query, mode) {
            Ok(result) => match mode {
                ResolveMode::Default => {
                    println!("{}", result.path.display());
                    0
                }
                ResolveMode::List => {
                    println!("{}", result.path.display());
                    0
                }
                ResolveMode::Json => {
                    let payload = JsonOutput {
                        status: "ok",
                        reason: None,
                        path: Some(result.path.display().to_string()),
                        candidates: None,
                    };
                    match serde_json::to_string(&payload) {
                        Ok(json) => {
                            println!("{json}");
                            0
                        }
                        Err(err) => {
                            eprintln!("dx resolve: failed to serialize json: {err}");
                            1
                        }
                    }
                }
            },
            Err(err) => self.emit_error(err, mode),
        }
    }

    pub fn resolve(
        &self,
        query: ResolveQuery<'_>,
        mode: ResolveMode,
    ) -> Result<ResolveResult, ResolveError> {
        let trimmed = query.raw.trim();
        if trimmed.is_empty() {
            return Err(ResolveError::EmptyQuery);
        }

        let mut resolution_query = trimmed;

        if let Some(path) = precedence::resolve_direct(query.cwd, trimmed) {
            if path.is_dir() {
                return Ok(ResolveResult { path });
            }

            if is_filesystem_prefix(trimmed) {
                let stripped = strip_filesystem_prefix_for_fallback(trimmed);
                if stripped.is_empty() {
                    return Err(ResolveError::PathNotFound(path.display().to_string()));
                }
                resolution_query = stripped;
            } else {
                return Err(ResolveError::PathNotFound(path.display().to_string()));
            }
        }

        if let Some(path) = traversal::resolve_step_up(query.cwd, resolution_query) {
            return Ok(ResolveResult { path });
        }

        let effective_roots = build_effective_roots(query.cwd, &self.config.search_roots);

        let mut candidates = abbreviation::resolve_abbreviation(
            &effective_roots,
            resolution_query,
            self.config.resolve.case_sensitive,
        );

        if candidates.is_empty() {
            candidates = roots::resolve_fallbacks(
                &effective_roots,
                resolution_query,
                self.config.resolve.case_sensitive,
            );
        }

        if candidates.is_empty() {
            if let Some(path) = (self.bookmark_lookup)(resolution_query) {
                return Ok(ResolveResult { path });
            }
            return Err(ResolveError::NotFound);
        }

        if candidates.len() == 1 {
            return Ok(ResolveResult {
                path: candidates.remove(0),
            });
        }

        prepare_candidates(&mut candidates, None);

        match mode {
            ResolveMode::List => Err(ResolveError::Ambiguous {
                count: candidates.len(),
                candidates,
            }),
            ResolveMode::Json => Err(ResolveError::Ambiguous {
                count: candidates.len(),
                candidates,
            }),
            ResolveMode::Default => Err(ResolveError::Ambiguous {
                count: candidates.len(),
                candidates,
            }),
        }
    }

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
            if completion_query.is_empty() {
                return apply_completion_limit(output, limit);
            }
        }

        let probe_limit = limit.map(|value| value.saturating_add(1));

        if let Some(path) = precedence::resolve_direct(&cwd, completion_query) {
            if path.is_dir() {
                push_unique(&mut output, &mut seen, path);
            }
        }

        if let Some(path) = traversal::resolve_step_up(&cwd, completion_query) {
            if path.is_dir() {
                push_unique(&mut output, &mut seen, path);
            }
        }

        let effective_roots = build_effective_roots(&cwd, &self.config.search_roots);

        let mut abbreviation_candidates = abbreviation::resolve_abbreviation(
            &effective_roots,
            completion_query,
            self.config.resolve.case_sensitive,
        );
        prepare_candidates(
            &mut abbreviation_candidates,
            probe_limit,
        );
        for candidate in abbreviation_candidates {
            push_unique(&mut output, &mut seen, candidate);
        }

        let mut fallback_candidates = roots::resolve_fallbacks(
            &effective_roots,
            completion_query,
            self.config.resolve.case_sensitive,
        );
        prepare_candidates(
            &mut fallback_candidates,
            probe_limit,
        );
        for candidate in fallback_candidates {
            push_unique(&mut output, &mut seen, candidate);
        }

        if let Some(path) = (self.bookmark_lookup)(completion_query) {
            push_unique(&mut output, &mut seen, path);
        }

        apply_completion_limit(output, limit)
    }

    fn emit_error(&self, err: ResolveError, mode: ResolveMode) -> i32 {
        match (mode, err) {
            (ResolveMode::Json, ResolveError::Ambiguous { candidates, .. }) => {
                let payload = JsonOutput {
                    status: "error",
                    reason: Some("ambiguous"),
                    path: None,
                    candidates: Some(
                        candidates
                            .into_iter()
                            .map(|path| path.display().to_string())
                            .collect(),
                    ),
                };
                match serde_json::to_string(&payload) {
                    Ok(json) => {
                        println!("{json}");
                        0
                    }
                    Err(serialization_error) => {
                        eprintln!("dx resolve: failed to serialize json: {serialization_error}");
                        1
                    }
                }
            }
            (ResolveMode::Json, ResolveError::NotFound) => {
                let payload = JsonOutput {
                    status: "error",
                    reason: Some("not_found"),
                    path: None,
                    candidates: None,
                };
                match serde_json::to_string(&payload) {
                    Ok(json) => {
                        println!("{json}");
                        1
                    }
                    Err(serialization_error) => {
                        eprintln!("dx resolve: failed to serialize json: {serialization_error}");
                        1
                    }
                }
            }
            (ResolveMode::List, ResolveError::Ambiguous { candidates, .. }) => {
                for candidate in candidates {
                    println!("{}", candidate.display());
                }
                0
            }
            (_, ResolveError::Ambiguous { candidates, .. }) => {
                eprintln!("dx resolve: ambiguous query; candidates:");
                for candidate in candidates {
                    eprintln!("- {}", candidate.display());
                }
                1
            }
            (_, other) => {
                eprintln!("dx resolve: {other}");
                1
            }
        }
    }
}


fn normalized_root_key(path: &Path) -> String {
    let normalized = std::fs::canonicalize(path)
        .unwrap_or_else(|_| traversal::normalize_path(path));
    normalized.display().to_string()
}

fn build_effective_roots(cwd: &Path, configured_roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut roots = Vec::new();

    for root in configured_roots {
        let key = normalized_root_key(root);
        if seen.insert(key) {
            roots.push(root.clone());
        }
    }

    let cwd_key = normalized_root_key(cwd);
    if seen.insert(cwd_key) {
        roots.push(cwd.to_path_buf());
    }

    roots
}


fn push_unique(output: &mut Vec<PathBuf>, seen: &mut HashSet<String>, candidate: PathBuf) {
    let key = candidate.display().to_string();
    if seen.insert(key) {
        output.push(candidate);
    }
}

fn prepare_candidates(candidates: &mut Vec<PathBuf>, max: Option<usize>) {
    candidates.sort_by(|left, right| {
        left.components()
            .count()
            .cmp(&right.components().count())
            .then_with(|| left.as_os_str().cmp(right.as_os_str()))
    });
    candidates.dedup();
    if let Some(max) = max {
        candidates.truncate(max);
    }
}

fn apply_completion_limit(mut paths: Vec<PathBuf>, limit: Option<usize>) -> CompletionCandidates {
    let mut has_more = false;
    if let Some(max) = limit
        && paths.len() > max
    {
        paths.truncate(max);
        has_more = true;
    }

    CompletionCandidates { paths, has_more }
}

/// Returns true when the query is a filesystem path prefix that should be
/// expanded via readdir rather than the search-root / abbreviation pipeline.
/// Matches: absolute paths (/…), home-relative (~/…), and explicit relative
/// paths (./… or ../…).
fn is_filesystem_prefix(query: &str) -> bool {
    query.starts_with('/')
        || query.starts_with("~/")
        || query == "~"
        || query.starts_with("./")
        || query.starts_with("../")
}

fn strip_filesystem_prefix_for_fallback(query: &str) -> &str {
    if let Some(stripped) = query.strip_prefix("~/") {
        stripped
    } else if query == "~" {
        ""
    } else if let Some(stripped) = query.strip_prefix("./") {
        stripped
    } else if let Some(stripped) = query.strip_prefix("../") {
        stripped
    } else if let Some(stripped) = query.strip_prefix('/') {
        stripped
    } else {
        query
    }
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
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::test_support;

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

    fn create_resolver_with_roots(roots: Vec<PathBuf>) -> Resolver {
        Resolver::with_bookmark_lookup(
            AppConfig {
                search_roots: roots,
                ..AppConfig::default()
            },
            |_| None,
        )
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

    fn set_bookmark_env(file: Option<String>) {
        if let Some(path) = file {
            unsafe { env::set_var("DX_BOOKMARKS_FILE", path) };
        } else {
            unsafe { env::remove_var("DX_BOOKMARKS_FILE") };
        }
    }

    #[test]
    fn resolves_absolute_existing_path() {
        let temp = make_temp_dir("resolve-abs");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: temp.to_str().expect("utf8 path"),
            cwd: &temp,
        };

        let result = resolver
            .resolve(query, ResolveMode::Default)
            .expect("resolve");
        assert_eq!(result.path, temp);
    }

    #[test]
    fn resolves_relative_existing_path() {
        let temp = make_temp_dir("resolve-rel");
        let child = temp.join("src");
        fs::create_dir_all(&child).expect("create dir");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: "./src",
            cwd: &temp,
        };

        let result = resolver
            .resolve(query, ResolveMode::Default)
            .expect("resolve");
        assert_eq!(result.path, child);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn errors_on_nonexistent_path() {
        let temp = make_temp_dir("resolve-miss");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);

        let query = ResolveQuery {
            raw: "./does-not-exist",
            cwd: &temp,
        };

        let err = resolver
            .resolve(query, ResolveMode::Default)
            .expect_err("should error");
        assert!(matches!(err, ResolveError::NotFound));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn resolve_leading_slash_direct_miss_falls_back_to_abbreviation() {
        let temp = make_temp_dir("resolve-leading-slash-fallback");
        let root = temp.join("root");
        let missing_prefix = format!("dx-miss-{}", std::process::id());
        let target = root.join(&missing_prefix).join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        let query = ResolveQuery {
            raw: &format!("/{missing_prefix}/pro"),
            cwd: &temp,
        };

        let result = resolver
            .resolve(query, ResolveMode::Default)
            .expect("fallback should resolve");
        assert_eq!(result.path, target);

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn resolve_dot_slash_direct_miss_falls_back_to_abbreviation() {
        let temp = make_temp_dir("resolve-dot-slash-fallback");
        let root = temp.join("root");
        let target = root.join("no-local-hit").join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        let query = ResolveQuery {
            raw: "./no-local-hit/pro",
            cwd: &temp,
        };

        let result = resolver
            .resolve(query, ResolveMode::Default)
            .expect("fallback should resolve");
        assert_eq!(result.path, target);

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn resolve_tilde_slash_direct_miss_falls_back_to_abbreviation() {
        let _guard = env_lock();
        let temp = make_temp_dir("resolve-tilde-slash-fallback");
        let home = temp.join("home");
        fs::create_dir_all(&home).expect("create home");

        let root = temp.join("root");
        let target = root.join("no-home-hit").join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let prev_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", &home) };

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        let query = ResolveQuery {
            raw: "~/no-home-hit/pro",
            cwd: &temp,
        };

        let result = resolver
            .resolve(query, ResolveMode::Default)
            .expect("fallback should resolve");
        assert_eq!(result.path, target);

        if let Some(value) = prev_home {
            unsafe { std::env::set_var("HOME", value) };
        } else {
            unsafe { std::env::remove_var("HOME") };
        }

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn resolve_prefixed_empty_fallback_query_preserves_path_not_found() {
        let _guard = env_lock();
        let temp = make_temp_dir("resolve-empty-prefixed-fallback");
        let missing_home = temp.join("missing-home");

        let prev_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", &missing_home) };

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: "~/",
            cwd: &temp,
        };

        let err = resolver
            .resolve(query, ResolveMode::Default)
            .expect_err("missing home directory should keep path-not-found");
        assert!(matches!(err, ResolveError::PathNotFound(_)));

        if let Some(value) = prev_home {
            unsafe { std::env::set_var("HOME", value) };
        } else {
            unsafe { std::env::remove_var("HOME") };
        }

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn returns_ambiguous_error_for_multiple_candidates() {
        let temp = make_temp_dir("resolve-ambiguous");
        let root = temp.join("root");
        fs::create_dir_all(root.join("proj/alpha")).expect("create proj alpha");
        fs::create_dir_all(root.join("prod/alpha")).expect("create prod alpha");

        let resolver = create_resolver_with_roots(vec![root]);
        let query = ResolveQuery {
            raw: "pro/al",
            cwd: &temp,
        };

        let err = resolver
            .resolve(query, ResolveMode::Default)
            .expect_err("should be ambiguous");
        assert!(matches!(
            err,
            ResolveError::Ambiguous {
                count: 2,
                candidates: _
            }
        ));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn direct_resolution_wins_over_fallback_search_root() {
        let temp = make_temp_dir("resolve-precedence");
        let local = temp.join("src");
        fs::create_dir_all(&local).expect("create local src");

        let root = temp.join("root");
        fs::create_dir_all(root.join("src")).expect("create fallback src");

        let resolver = create_resolver_with_roots(vec![root]);
        let query = ResolveQuery {
            raw: "src",
            cwd: &temp,
        };

        let result = resolver
            .resolve(query, ResolveMode::Default)
            .expect("should resolve local");
        assert_eq!(result.path, local);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn bookmark_resolves_when_no_filesystem_match_exists() {
        let _guard = env_lock();
        let temp = make_temp_dir("resolve-bookmark");
        let bookmarks_file = temp.join("bookmarks.toml");
        let bookmark_target = temp.join("target");
        fs::create_dir_all(&bookmark_target).expect("create bookmark target");

        let canonical_target = fs::canonicalize(&bookmark_target).expect("canonical target");
        let toml = format!(
            "[bookmarks]\nproj = \"{}\"\n",
            canonical_target.display().to_string().replace('\\', "\\\\")
        );
        fs::write(&bookmarks_file, toml).expect("write bookmarks file");
        set_bookmark_env(Some(bookmarks_file.display().to_string()));

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: "proj",
            cwd: &temp,
        };

        let result = resolver
            .resolve(query, ResolveMode::Default)
            .expect("bookmark should resolve");
        assert_eq!(result.path, canonical_target);

        set_bookmark_env(None);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn fallback_root_takes_precedence_over_bookmark() {
        let _guard = env_lock();
        let temp = make_temp_dir("resolve-fallback-over-bookmark");
        let bookmarks_file = temp.join("bookmarks.toml");

        let fallback_root = temp.join("root");
        let fallback_match = fallback_root.join("proj");
        fs::create_dir_all(&fallback_match).expect("create fallback match");

        let bookmark_target = temp.join("bookmark-target");
        fs::create_dir_all(&bookmark_target).expect("create bookmark target");
        let canonical_bookmark = fs::canonicalize(&bookmark_target).expect("canonical bookmark");
        let toml = format!(
            "[bookmarks]\nproj = \"{}\"\n",
            canonical_bookmark
                .display()
                .to_string()
                .replace('\\', "\\\\")
        );
        fs::write(&bookmarks_file, toml).expect("write bookmarks file");
        set_bookmark_env(Some(bookmarks_file.display().to_string()));

        let resolver = create_resolver_with_roots_and_bookmarks(vec![fallback_root]);
        let query = ResolveQuery {
            raw: "proj",
            cwd: &temp,
        };

        let result = resolver
            .resolve(query, ResolveMode::Default)
            .expect("fallback should resolve");
        assert_eq!(result.path, fallback_match);

        set_bookmark_env(None);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn stale_bookmark_returns_no_match_and_resolution_fails() {
        let _guard = env_lock();
        let temp = make_temp_dir("resolve-stale-bookmark");
        let bookmarks_file = temp.join("bookmarks.toml");
        let missing_target = temp.join("missing-target");

        let toml = format!(
            "[bookmarks]\nproj = \"{}\"\n",
            missing_target.display().to_string().replace('\\', "\\\\")
        );
        fs::write(&bookmarks_file, toml).expect("write bookmarks file");
        set_bookmark_env(Some(bookmarks_file.display().to_string()));

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: "proj",
            cwd: &temp,
        };

        let err = resolver
            .resolve(query, ResolveMode::Default)
            .expect_err("stale bookmark should fail");
        assert!(matches!(err, ResolveError::NotFound));

        set_bookmark_env(None);
        let _ = fs::remove_dir_all(temp);
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
    fn completion_leading_slash_empty_filesystem_falls_back_to_abbreviation() {
        let _guard = env_lock();
        let temp = make_temp_dir("complete-leading-slash-fallback");
        let root = temp.join("root");
        let missing_prefix = format!("dx-miss-{}", std::process::id());
        let target = root.join(&missing_prefix).join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        let query = format!("/{missing_prefix}/pro");
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
    fn effective_roots_include_cwd_when_no_roots_configured() {
        let temp = make_temp_dir("effective-roots-cwd");
        let roots = build_effective_roots(&temp, &[]);
        assert_eq!(roots, vec![temp.clone()]);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn effective_roots_dedup_when_cwd_already_configured() {
        let temp = make_temp_dir("effective-roots-dedup");
        let roots = build_effective_roots(&temp, std::slice::from_ref(&temp));
        assert_eq!(roots, vec![temp.clone()]);
        let _ = fs::remove_dir_all(temp);
    }

}
