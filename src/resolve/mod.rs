pub mod abbreviation;
mod completion;
mod output;
mod pipeline;
pub mod precedence;
pub mod roots;
pub mod traversal;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone)]
pub struct Resolver {
    pub(crate) config: AppConfig,
    bookmark_lookup: fn(&str) -> Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct CompletionCandidates {
    pub paths: Vec<PathBuf>,
    pub has_more: bool,
}

impl CompletionCandidates {
    pub fn empty() -> Self {
        Self {
            paths: Vec::new(),
            has_more: false,
        }
    }
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
}

pub(super) fn normalized_root_key(path: &Path) -> String {
    let normalized =
        std::fs::canonicalize(path).unwrap_or_else(|_| traversal::normalize_path(path));
    normalized.display().to_string()
}

pub(super) fn build_effective_roots(cwd: &Path, configured_roots: &[PathBuf]) -> Vec<PathBuf> {
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

pub(super) fn prepare_candidates(candidates: &mut Vec<PathBuf>, max: Option<usize>) {
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

/// Returns true when the query is a filesystem path prefix that should be
/// expanded via readdir rather than the search-root / abbreviation pipeline.
/// Matches: absolute paths (/…), home-relative (~/…), and explicit relative
/// paths (./… or ../…).
pub(super) fn is_filesystem_prefix(query: &str) -> bool {
    query.starts_with('/')
        || query.starts_with("~/")
        || query == "~"
        || query.starts_with("./")
        || query.starts_with("../")
}

pub(super) fn strip_filesystem_prefix_for_fallback(query: &str) -> &str {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FilesystemPrefixFallback {
    DirectResolutionOnly,
    AlwaysForFilesystemPrefix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FallbackScope {
    Standard,
    RootAnchored,
}

#[derive(Debug, Clone)]
pub(super) struct FallbackPolicy {
    pub effective_roots: Vec<PathBuf>,
    pub allow_step_up: bool,
    pub allow_bookmark_lookup: bool,
}

impl FallbackPolicy {
    pub fn from_query_context(
        cwd: &Path,
        configured_roots: &[PathBuf],
        raw_query: &str,
        uses_prefix_fallback: bool,
    ) -> Self {
        let scope = if uses_prefix_fallback && raw_query.starts_with('/') {
            FallbackScope::RootAnchored
        } else {
            FallbackScope::Standard
        };

        match scope {
            FallbackScope::Standard => Self {
                effective_roots: build_effective_roots(cwd, configured_roots),
                allow_step_up: true,
                allow_bookmark_lookup: true,
            },
            FallbackScope::RootAnchored => Self {
                effective_roots: vec![PathBuf::from("/")],
                allow_step_up: false,
                allow_bookmark_lookup: false,
            },
        }
    }

    pub fn allow_direct_injection(&self) -> bool {
        self.allow_step_up
    }
}

#[derive(Debug, Clone)]
pub(super) struct PreparedQuery<'a> {
    pub effective_query: &'a str,
    pub direct_dir: Option<PathBuf>,
    pub fallback_policy: FallbackPolicy,
}

pub(super) fn prepare_search_query<'a>(
    cwd: &Path,
    configured_roots: &[PathBuf],
    raw_query: &'a str,
    prefix_fallback: FilesystemPrefixFallback,
) -> Result<PreparedQuery<'a>, ResolveError> {
    let trimmed = raw_query.trim();
    if trimmed.is_empty() {
        return Err(ResolveError::EmptyQuery);
    }

    let mut effective_query = trimmed;
    let mut uses_prefix_fallback = false;
    let mut direct_dir = None;

    if let Some(path) = precedence::resolve_direct(cwd, trimmed) {
        if path.is_dir() {
            direct_dir = Some(path);
        } else if is_filesystem_prefix(trimmed) {
            effective_query = strip_filesystem_prefix_for_fallback(trimmed);
            if effective_query.is_empty() {
                return Err(ResolveError::PathNotFound(path.display().to_string()));
            }
            uses_prefix_fallback = true;
        } else {
            return Err(ResolveError::PathNotFound(path.display().to_string()));
        }
    } else if prefix_fallback == FilesystemPrefixFallback::AlwaysForFilesystemPrefix
        && is_filesystem_prefix(trimmed)
    {
        effective_query = strip_filesystem_prefix_for_fallback(trimmed);
        if effective_query.is_empty() {
            return Err(ResolveError::PathNotFound(trimmed.to_string()));
        }
        uses_prefix_fallback = true;
    }

    let fallback_policy =
        FallbackPolicy::from_query_context(cwd, configured_roots, trimmed, uses_prefix_fallback);

    Ok(PreparedQuery {
        effective_query,
        direct_dir,
        fallback_policy,
    })
}

pub(super) fn resolve_search_candidates(
    effective_roots: &[PathBuf],
    query: &str,
    case_sensitive: bool,
) -> Vec<PathBuf> {
    let mut candidates = abbreviation::resolve_abbreviation(effective_roots, query, case_sensitive);
    if candidates.is_empty() {
        candidates = roots::resolve_fallbacks(effective_roots, query, case_sensitive);
    }
    candidates
}
