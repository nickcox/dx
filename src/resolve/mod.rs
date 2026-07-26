//! Turns an abbreviated query into one directory. Bookmarks, the session stack
//! and search roots are consulted in a fixed precedence order, and an ambiguous
//! query is an error rather than a guess.
pub mod abbreviation;
mod completion;
pub(crate) mod path_query;
mod pipeline;
pub mod precedence;
pub mod roots;
pub mod traversal;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{
    bookmarks,
    config::{AppConfig, ConfigError},
};

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
    #[error("unsupported drive-relative query: {0}")]
    DriveRelativePath(String),
    #[error("failed to access {path}: {source}")]
    Filesystem {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("query is ambiguous ({} matches)", .candidates.len())]
    Ambiguous { candidates: Vec<PathBuf> },
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
    /// Truncates to `limit`, recording whether anything was dropped.
    pub fn limited(paths: Vec<PathBuf>, limit: Option<usize>) -> Self {
        let (paths, has_more) = crate::common::truncate_with_has_more(paths, limit);
        Self { paths, has_more }
    }

    pub fn empty() -> Self {
        Self {
            paths: Vec::new(),
            has_more: false,
        }
    }
}

impl Resolver {
    pub fn from_environment() -> Result<Self, ConfigError> {
        let config = AppConfig::load()?;
        Ok(Self {
            config,
            bookmark_lookup: bookmarks::lookup,
        })
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
        query: path_query::PathQuery<'_>,
        uses_prefix_fallback: bool,
    ) -> Self {
        let scope = if uses_prefix_fallback && query.root_anchor(cwd).is_some() {
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
                effective_roots: vec![query.root_anchor(cwd).expect("root-anchored query")],
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
pub(super) struct PreparedQuery {
    pub effective_query: String,
    pub direct_dir: Option<PathBuf>,
    pub fallback_policy: FallbackPolicy,
}

pub(super) fn prepare_search_query(
    cwd: &Path,
    configured_roots: &[PathBuf],
    raw_query: &str,
    prefix_fallback: FilesystemPrefixFallback,
) -> Result<PreparedQuery, ResolveError> {
    if raw_query.is_empty() {
        return Err(ResolveError::EmptyQuery);
    }

    let query = path_query::PathQuery::new(raw_query);
    if query.kind == path_query::QueryKind::DriveRelative {
        return Err(ResolveError::DriveRelativePath(raw_query.to_string()));
    }

    let mut effective_query = raw_query.to_string();
    let mut uses_prefix_fallback = false;
    let mut direct_dir = None;

    #[cfg(windows)]
    let step_up_alias = traversal::resolve_step_up(cwd, raw_query).is_some();
    #[cfg(not(windows))]
    let step_up_alias = false;

    if !step_up_alias
        && let Some(path) =
            precedence::resolve_direct(cwd, query).map_err(|source| ResolveError::Filesystem {
                path: PathBuf::from(raw_query),
                source,
            })?
    {
        let is_dir = match std::fs::metadata(&path) {
            Ok(metadata) => metadata.is_dir(),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => false,
            Err(source) => {
                return Err(ResolveError::Filesystem {
                    path: path.clone(),
                    source,
                });
            }
        };
        if is_dir {
            direct_dir = Some(path);
        } else if query.is_filesystem_prefix() {
            effective_query = query
                .fallback_segments()
                .join(std::path::MAIN_SEPARATOR_STR);
            if effective_query.is_empty() {
                return Err(ResolveError::PathNotFound(path.display().to_string()));
            }
            uses_prefix_fallback = true;
        } else {
            return Err(ResolveError::PathNotFound(path.display().to_string()));
        }
    } else if prefix_fallback == FilesystemPrefixFallback::AlwaysForFilesystemPrefix
        && query.is_filesystem_prefix()
    {
        effective_query = query
            .fallback_segments()
            .join(std::path::MAIN_SEPARATOR_STR);
        if effective_query.is_empty() {
            return Err(ResolveError::PathNotFound(raw_query.to_string()));
        }
        uses_prefix_fallback = true;
    }

    let fallback_policy =
        FallbackPolicy::from_query_context(cwd, configured_roots, query, uses_prefix_fallback);

    Ok(PreparedQuery {
        effective_query,
        direct_dir,
        fallback_policy,
    })
}

/// Abbreviation matches first, falling back to a root scan only when the
/// abbreviation pass finds nothing.
///
/// `on_error` decides whether an unreadable directory is skipped or surfaced —
/// the single knob that separates completion from resolution.
pub(super) fn resolve_search_candidates(
    effective_roots: &[PathBuf],
    query: &str,
    case_sensitive: bool,
    on_error: traversal::OnIoError,
) -> Result<Vec<PathBuf>, ResolveError> {
    let mut candidates =
        abbreviation::resolve_abbreviation(effective_roots, query, case_sensitive, on_error)
            .map_err(filesystem_error)?;
    if candidates.is_empty() {
        candidates = roots::resolve_fallbacks(effective_roots, query, case_sensitive, on_error)
            .map_err(filesystem_error)?;
    }
    Ok(candidates)
}

fn filesystem_error((path, source): traversal::TraversalError) -> ResolveError {
    ResolveError::Filesystem { path, source }
}
