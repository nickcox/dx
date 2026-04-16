pub mod abbreviation;
pub mod precedence;
pub mod roots;
pub mod traversal;
mod completion;
mod output;
mod pipeline;

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
    let normalized = std::fs::canonicalize(path)
        .unwrap_or_else(|_| traversal::normalize_path(path));
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
