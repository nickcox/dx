//! The bookmark seam resolution reads through. Mirrors [`FrecencyProvider`], an
//! injectable candidate source with one production implementation and cheap test
//! doubles, so nothing inside `resolve` reaches the store directly.
//!
//! [`FrecencyProvider`]: crate::frecency::FrecencyProvider

use std::path::PathBuf;

/// Where [`Resolver`] gets bookmarks from.
///
/// `case_sensitive` is a parameter rather than source state so an implementation
/// never has to know about [`AppConfig`], matching how the resolver already
/// threads the flag into its search-candidate pass.
///
/// [`Resolver`]: super::Resolver
/// [`AppConfig`]: crate::config::AppConfig
pub trait BookmarkSource: std::fmt::Debug {
    /// Exact name match. A stale target — one that is no longer a directory —
    /// yields `None`, because a path the user cannot `cd` to is not a match.
    fn get(&self, name: &str) -> Option<PathBuf>;

    /// Live targets of every bookmark whose name starts with `prefix`, in name
    /// order. An empty prefix yields all of them. Stale targets are excluded for
    /// the same reason [`get`](Self::get) drops them.
    fn prefix_matches(&self, prefix: &str, case_sensitive: bool) -> Vec<PathBuf>;
}

/// A source with no bookmarks, for resolution paths and tests that do not
/// exercise them.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoBookmarks;

impl BookmarkSource for NoBookmarks {
    fn get(&self, _name: &str) -> Option<PathBuf> {
        None
    }

    fn prefix_matches(&self, _prefix: &str, _case_sensitive: bool) -> Vec<PathBuf> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{BookmarkSource, NoBookmarks};

    #[test]
    fn no_bookmarks_source_returns_nothing() {
        assert!(NoBookmarks.get("work").is_none());
        assert!(NoBookmarks.prefix_matches("", true).is_empty());
        assert!(NoBookmarks.prefix_matches("wo", false).is_empty());
    }
}
