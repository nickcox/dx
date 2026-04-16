pub mod action;
pub mod buffer;
pub mod tui;

use std::collections::HashSet;
use std::path::PathBuf;

use crate::complete::{
    self, ancestors, recents as recents_mode, stack as stack_mode,
    CompletionMode,
};
use crate::common;
use crate::frecency::ZoxideProvider;
use crate::resolve::{CompletionCandidates, Resolver};

pub use action::MenuAction;
pub use buffer::{parse_buffer, parse_buffer_with_mode, ParsedBuffer};
pub use tui::MenuResult;

/// Source completion candidates for the given mode and query,
/// reusing the same pipelines as `dx complete`.
/// Duplicates are removed for all modes.
/// The cwd itself is filtered out for non-`Paths` modes only; `Paths` mode
/// intentionally keeps cwd-targeted results because listing cwd children is a
/// primary navigation use case.
pub fn source_candidates(
    resolver: &Resolver,
    mode: CompletionMode,
    query: Option<&str>,
    session: Option<&str>,
    cwd: Option<&std::path::Path>,
) -> Vec<PathBuf> {
    source_candidates_with_meta(
        resolver,
        mode,
        query,
        session,
        cwd,
        None,
    )
    .paths
}

pub fn source_candidates_with_meta(
    resolver: &Resolver,
    mode: CompletionMode,
    query: Option<&str>,
    session: Option<&str>,
    cwd: Option<&std::path::Path>,
    limit: Option<usize>,
) -> CompletionCandidates {
    let raw_meta = match mode {
        CompletionMode::Paths => resolver.collect_completion_candidates_with_limit_and_cwd(
            query.unwrap_or(""),
            limit,
            cwd,
        ),
        CompletionMode::Ancestors => {
            apply_limit_with_has_more(ancestors::complete(query), limit)
        }
        CompletionMode::Frecents => {
            let provider = ZoxideProvider::default();
            apply_limit_with_has_more(complete::complete_frecents(&provider, query), limit)
        }
        CompletionMode::Recents => {
            apply_limit_with_has_more(recents_mode::complete(session, query), limit)
        }
        CompletionMode::Stack(direction) => {
            apply_limit_with_has_more(stack_mode::complete(session, direction, query), limit)
        }
    };

    let canonical_cwd = match mode {
        CompletionMode::Paths => None,
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

fn apply_limit_with_has_more(
    paths: Vec<PathBuf>,
    limit: Option<usize>,
) -> CompletionCandidates {
    let (paths, has_more) = common::truncate_with_has_more(paths, limit);

    CompletionCandidates { paths, has_more }
}
