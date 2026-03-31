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

        if let Some(path) = precedence::resolve_direct(query.cwd, trimmed) {
            if path.is_dir() {
                return Ok(ResolveResult { path });
            }
            return Err(ResolveError::PathNotFound(path.display().to_string()));
        }

        if let Some(path) = traversal::resolve_step_up(query.cwd, trimmed) {
            return Ok(ResolveResult { path });
        }

        let mut candidates = abbreviation::resolve_abbreviation(
            &self.config.search_roots,
            trimmed,
            self.config.resolve.case_sensitive,
        );

        if candidates.is_empty() {
            candidates = roots::resolve_fallbacks(
                &self.config.search_roots,
                trimmed,
                self.config.resolve.case_sensitive,
            );
        }

        if candidates.is_empty() {
            if let Some(path) = (self.bookmark_lookup)(trimmed) {
                return Ok(ResolveResult { path });
            }
            return Err(ResolveError::NotFound);
        }

        if candidates.len() == 1 {
            return Ok(ResolveResult {
                path: candidates.remove(0),
            });
        }

        prepare_candidates(&mut candidates, self.config.resolve.max_list_results);

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
        let trimmed = raw_query.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let cwd = match std::env::current_dir() {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };

        let mut output = Vec::new();
        let mut seen = HashSet::new();

        if let Some(path) = precedence::resolve_direct(&cwd, trimmed) {
            if path.is_dir() {
                push_unique(&mut output, &mut seen, path);
            }
        }

        if let Some(path) = traversal::resolve_step_up(&cwd, trimmed) {
            if path.is_dir() {
                push_unique(&mut output, &mut seen, path);
            }
        }

        let mut abbreviation_candidates = abbreviation::resolve_abbreviation(
            &self.config.search_roots,
            trimmed,
            self.config.resolve.case_sensitive,
        );
        prepare_candidates(
            &mut abbreviation_candidates,
            self.config.resolve.max_list_results,
        );
        for candidate in abbreviation_candidates {
            push_unique(&mut output, &mut seen, candidate);
        }

        let mut fallback_candidates = roots::resolve_fallbacks(
            &self.config.search_roots,
            trimmed,
            self.config.resolve.case_sensitive,
        );
        prepare_candidates(
            &mut fallback_candidates,
            self.config.resolve.max_list_results,
        );
        for candidate in fallback_candidates {
            push_unique(&mut output, &mut seen, candidate);
        }

        if let Some(path) = (self.bookmark_lookup)(trimmed) {
            push_unique(&mut output, &mut seen, path);
        }

        if let Some(max) = self.config.resolve.max_list_results {
            output.truncate(max);
        }

        output
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
        assert!(matches!(err, ResolveError::PathNotFound(_)));
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
}
