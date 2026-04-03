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
/// Duplicates and the cwd itself are removed before returning.
pub fn source_candidates(
    resolver: &Resolver,
    mode: CompletionMode,
    query: Option<&str>,
    session: Option<&str>,
    cwd: Option<&std::path::Path>,
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

    // Canonicalize cwd once for comparison (handles macOS /private/var symlinks).
    let canonical_cwd = cwd.and_then(|p| std::fs::canonicalize(p).ok());

    // Deduplicate (first occurrence wins) and strip the cwd itself.
    let mut seen = HashSet::new();
    raw.into_iter()
        .filter(|p| {
            let canonical = std::fs::canonicalize(p).unwrap_or_else(|_| p.clone());
            if let Some(ref ccwd) = canonical_cwd {
                if &canonical == ccwd {
                    return false;
                }
            }
            seen.insert(canonical)
        })
        .collect()
}
