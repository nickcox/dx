//! Completing a filesystem prefix by listing the parent directory, filtered to
//! paths, directories or files.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use clap::ValueEnum;

use crate::common;
use crate::resolve::path_query::{PathQuery, QueryKind};
use crate::resolve::precedence;
use crate::resolve::{CompletionCandidates, Resolver};

/// What a filesystem completion is allowed to return.
///
/// This is the one spelling of "path | directory | file" in the crate: it backs
/// `dx complete filesystem <KIND>`, `dx menu --mode`, and the mode half of
/// `DX_MENU_COMMAND_MAPPINGS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FilesystemCompletionKind {
    Path,
    Directory,
    File,
}

impl FilesystemCompletionKind {
    /// The literal accepted on the command line and inside
    /// `DX_MENU_COMMAND_MAPPINGS`. Kept in step with clap's value names by
    /// `cli_arg_names_match_clap_value_names`.
    pub fn as_cli_arg(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Directory => "directory",
            Self::File => "file",
        }
    }
}

pub fn complete(
    resolver: &Resolver,
    query: Option<&str>,
    cwd: Option<&Path>,
    limit: Option<usize>,
    kind: FilesystemCompletionKind,
) -> CompletionCandidates {
    let cwd = match cwd {
        Some(path) => path.to_path_buf(),
        None => match std::env::current_dir() {
            Ok(value) => value,
            Err(_) => return CompletionCandidates::empty(),
        },
    };

    let raw_query = query.unwrap_or("");
    let mut combined = Vec::new();
    let mut seen = HashSet::new();

    if !raw_query.is_empty()
        && matches!(
            kind,
            FilesystemCompletionKind::Path | FilesystemCompletionKind::Directory
        )
        && !PathQuery::new(raw_query).is_filesystem_prefix()
    {
        let smart_dirs = resolver.collect_completion_candidates_with_limit_and_cwd(
            raw_query,
            None,
            Some(cwd.as_path()),
        );

        for path in smart_dirs.paths {
            if kind == FilesystemCompletionKind::Directory && !path.is_dir() {
                continue;
            }
            common::push_unique(&mut combined, &mut seen, path);
        }
    }

    let (parents, leaf_prefix) = parent_directories(resolver, &cwd, raw_query);
    for parent in parents {
        for child in list_children(&parent, &leaf_prefix, kind) {
            common::push_unique(&mut combined, &mut seen, child);
        }
    }

    CompletionCandidates::limited(combined, limit)
}

fn parent_directories(resolver: &Resolver, cwd: &Path, query: &str) -> (Vec<PathBuf>, String) {
    if query.is_empty() {
        return (vec![cwd.to_path_buf()], String::new());
    }

    let path_query = PathQuery::new(query);
    if path_query.kind == QueryKind::DriveRelative {
        return (Vec::new(), String::new());
    }
    let query_path = Path::new(query);
    let (parent_query, leaf_prefix) = if path_query.has_trailing_separator() {
        (Some(query_path), String::new())
    } else {
        let parent = query_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty());
        let leaf = query_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        (parent, leaf)
    };
    let Some(parent_query) = parent_query else {
        return (vec![cwd.to_path_buf()], leaf_prefix);
    };
    let Some(parent_query) = parent_query.to_str() else {
        return (Vec::new(), leaf_prefix);
    };

    if !PathQuery::new(parent_query).is_filesystem_prefix() {
        let smart_dirs = resolver.collect_completion_candidates_with_limit_and_cwd(
            parent_query,
            None,
            Some(cwd),
        );
        if !smart_dirs.paths.is_empty() {
            return (smart_dirs.paths, leaf_prefix);
        }
    }

    let fallback = precedence::resolve_direct(cwd, PathQuery::new(parent_query))
        .ok()
        .flatten()
        .filter(|path| path.is_dir())
        .into_iter()
        .collect::<Vec<_>>();

    (fallback, leaf_prefix)
}

fn list_children(parent: &Path, leaf_prefix: &str, kind: FilesystemCompletionKind) -> Vec<PathBuf> {
    let entries = match std::fs::read_dir(parent) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let prefix_lower = leaf_prefix.to_ascii_lowercase();
    let mut results = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !leaf_prefix.is_empty() && !name.to_ascii_lowercase().starts_with(&prefix_lower) {
            continue;
        }

        let path = entry.path();
        let keep = match kind {
            FilesystemCompletionKind::Path => true,
            FilesystemCompletionKind::Directory => path.is_dir(),
            FilesystemCompletionKind::File => path.is_file(),
        };
        if keep {
            results.push(path);
        }
    }

    common::sort_by_basename(&mut results);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn a_symlink_and_its_target_are_both_offered() {
        use crate::config::AppConfig;
        use crate::resolve::Resolver;
        use crate::test_support;

        let temp = test_support::temp_dir("filesystem-symlink-alias");
        let real = temp.path().join("real");
        std::fs::create_dir_all(&real).expect("create target directory");
        std::os::unix::fs::symlink(&real, temp.path().join("linked")).expect("create symlink");

        let resolver = Resolver::without_bookmarks(AppConfig::default());
        let query = format!("{}/", temp.path().display());
        let candidates = complete(
            &resolver,
            Some(&query),
            Some(temp.path()),
            None,
            FilesystemCompletionKind::Directory,
        );

        // Deduplicating by canonical path used to hide one of these, and which
        // one survived depended on readdir order. Both are valid `cd` targets.
        assert!(
            candidates.paths.iter().any(|path| path.ends_with("real")),
            "{:?}",
            candidates.paths
        );
        assert!(
            candidates.paths.iter().any(|path| path.ends_with("linked")),
            "{:?}",
            candidates.paths
        );
    }

    #[test]
    fn cli_arg_names_match_clap_value_names() {
        for kind in FilesystemCompletionKind::value_variants() {
            let clap_name = kind
                .to_possible_value()
                .expect("every kind is selectable on the command line");
            assert_eq!(kind.as_cli_arg(), clap_name.get_name());
        }
    }

    #[test]
    fn cli_arg_names_round_trip_case_insensitively() {
        for kind in FilesystemCompletionKind::value_variants() {
            assert_eq!(
                FilesystemCompletionKind::from_str(kind.as_cli_arg(), true).expect("round trip"),
                *kind
            );
        }
        assert_eq!(
            FilesystemCompletionKind::from_str("DIRECTORY", true).expect("case insensitive"),
            FilesystemCompletionKind::Directory
        );
        assert!(FilesystemCompletionKind::from_str("nonsense", true).is_err());
    }
}
