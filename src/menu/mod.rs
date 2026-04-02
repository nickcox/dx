pub mod action;
pub mod buffer;
pub mod tui;

use std::collections::HashSet;
use std::path::PathBuf;

use crate::complete::{
    self, ancestors, paths as paths_mode, recents as recents_mode, stack as stack_mode,
    CompletionMode,
};
use crate::frecency::ZoxideProvider;
use crate::resolve::Resolver;

pub use action::MenuAction;
pub use buffer::{parse_buffer, ParsedBuffer};
pub use tui::MenuResult;

/// Source completion candidates for the given mode and query,
/// reusing the same pipelines as `dx complete`.
/// Duplicates are removed (first occurrence wins) before returning.
pub fn source_candidates(
    resolver: &Resolver,
    mode: CompletionMode,
    query: Option<&str>,
    session: Option<&str>,
) -> Vec<PathBuf> {
    let raw = match mode {
        CompletionMode::Paths => paths_mode::complete(resolver, query.unwrap_or("")),
        CompletionMode::Ancestors => ancestors::complete(query),
        CompletionMode::Frecents => {
            let provider = ZoxideProvider::default();
            complete::complete_frecents(&provider, query)
        }
        CompletionMode::Recents => recents_mode::complete(session, query),
        CompletionMode::Stack(direction) => stack_mode::complete(session, direction, query),
    };

    // Deduplicate while preserving order (first occurrence wins).
    let mut seen = HashSet::new();
    raw.into_iter().filter(|p| seen.insert(p.clone())).collect()
}
