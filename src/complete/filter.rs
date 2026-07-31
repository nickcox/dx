//! Narrowing candidates by a typed query: exact, prefix and subsequence
//! matching, ranked so the closest match sorts first.

use std::path::{Component, Path, PathBuf};

use crate::resolve::path_query::{PathQuery, QueryKind};

/// Canonicalises a query for directory matching, returning encoded path bytes
/// ASCII-lowercased for comparison.
///
/// `./src` normalises to `src`: there is no cwd here to make it absolute, so both
/// must match the same candidates.
fn normalize_query(query: &str) -> Vec<u8> {
    let path_query = PathQuery::new(query);
    let query = if path_query.has_trailing_separator() && Path::new(query).file_name().is_some() {
        let separator = query.chars().next_back().expect("trailing separator");
        &query[..query.len() - separator.len_utf8()]
    } else {
        query
    };
    let mut path = match path_query.kind {
        QueryKind::Home => {
            let home = dirs::home_dir();
            match (home, query.strip_prefix('~')) {
                (Some(home), Some("")) => home,
                (Some(home), Some(rest)) => {
                    home.join(rest.trim_start_matches(std::path::is_separator))
                }
                _ => PathBuf::from(query),
            }
        }
        _ => PathBuf::from(query),
    };

    if path_query.kind == QueryKind::ExplicitRelative
        && matches!(
            Path::new(query).components().next(),
            Some(Component::CurDir)
        )
    {
        path = path.components().skip(1).collect();
    }

    crate::common::ascii_lowercase(path.as_os_str().as_encoded_bytes())
}

pub fn filter_candidates(candidates: &[PathBuf], query: &str) -> Vec<PathBuf> {
    let query = normalize_query(query);
    if query.is_empty() {
        return candidates.to_vec();
    }

    let mut exact_path = Vec::new();
    let mut exact_basename = Vec::new();
    let mut path_prefix = Vec::new();
    let mut basename_prefix = Vec::new();
    let mut substring = Vec::new();

    for candidate in candidates {
        let full_lower = crate::common::ascii_lowercase(candidate.as_os_str().as_encoded_bytes());
        let basename_lower = candidate
            .file_name()
            .map(|value| crate::common::ascii_lowercase(value.as_encoded_bytes()))
            .unwrap_or_default();

        if full_lower == query {
            exact_path.push(candidate.clone());
            continue;
        }

        if !basename_lower.is_empty() && basename_lower == query {
            exact_basename.push(candidate.clone());
            continue;
        }

        if full_lower.starts_with(&query) {
            path_prefix.push(candidate.clone());
            continue;
        }

        if !basename_lower.is_empty() && basename_lower.starts_with(&query) {
            basename_prefix.push(candidate.clone());
            continue;
        }

        if full_lower
            .windows(query.len())
            .any(|window| window == query)
        {
            substring.push(candidate.clone());
        }
    }

    exact_path
        .into_iter()
        .chain(exact_basename)
        .chain(path_prefix)
        .chain(basename_prefix)
        .chain(substring)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{filter_candidates, normalize_query};

    // --- normalize_query ---

    #[test]
    fn normalize_strips_native_trailing_separator_without_erasing_root() {
        assert_eq!(normalize_query("/foo/bar/"), b"/foo/bar");
        assert_eq!(normalize_query("/"), b"/");
    }

    #[test]
    fn normalize_expands_tilde() {
        if let Some(home) = dirs::home_dir() {
            let expected = crate::common::ascii_lowercase(
                home.join("projects").as_os_str().as_encoded_bytes(),
            );
            assert_eq!(normalize_query("~/projects"), expected);
        }
    }

    #[test]
    fn normalize_bare_tilde() {
        if let Some(home) = dirs::home_dir() {
            let expected = crate::common::ascii_lowercase(home.as_os_str().as_encoded_bytes());
            assert_eq!(normalize_query("~"), expected);
        }
    }

    #[test]
    fn normalize_strips_dot_slash_prefix() {
        assert_eq!(normalize_query("./src"), b"src");
    }

    #[test]
    fn normalize_trailing_slash_and_tilde() {
        if let Some(home) = dirs::home_dir() {
            let expected = crate::common::ascii_lowercase(
                home.join("projects").as_os_str().as_encoded_bytes(),
            );
            assert_eq!(normalize_query("~/projects/"), expected);
        }
    }

    // --- filter_candidates ---

    #[test]
    fn exact_basename_ranks_before_prefix_match() {
        let candidates = vec![
            PathBuf::from("/home/user/code-review"),
            PathBuf::from("/home/user/code"),
        ];

        let filtered = filter_candidates(&candidates, "code");
        assert_eq!(
            filtered,
            vec![
                PathBuf::from("/home/user/code"),
                PathBuf::from("/home/user/code-review")
            ]
        );
    }

    #[test]
    fn path_prefix_matches_are_included() {
        let candidates = vec![
            PathBuf::from("/home/user/projects/dx"),
            PathBuf::from("/tmp/scratch"),
        ];

        let filtered = filter_candidates(&candidates, "/home/user/pro");
        assert_eq!(filtered, vec![PathBuf::from("/home/user/projects/dx")]);
    }

    #[test]
    fn substring_matches_are_case_insensitive() {
        let candidates = vec![
            PathBuf::from("/home/user/projects/dx"),
            PathBuf::from("/tmp/scratch"),
        ];

        let filtered = filter_candidates(&candidates, "ProJ");
        assert_eq!(filtered, vec![PathBuf::from("/home/user/projects/dx")]);
    }

    #[test]
    fn preserves_input_order_within_same_match_tier() {
        let candidates = vec![
            PathBuf::from("/home/user/projects/alpha"),
            PathBuf::from("/home/user/projects/alpine"),
            PathBuf::from("/home/user/projects/algebra"),
        ];

        let filtered = filter_candidates(&candidates, "al");
        assert_eq!(filtered, candidates);
    }

    #[test]
    fn no_match_returns_empty() {
        let candidates = vec![PathBuf::from("/home/user/projects/dx")];
        let filtered = filter_candidates(&candidates, "zzz");
        assert!(filtered.is_empty());
    }

    #[test]
    fn trailing_slash_matches_candidate_without_slash() {
        let candidates = vec![PathBuf::from("/Users/nick/code/personal/dx")];
        let filtered = filter_candidates(&candidates, "/Users/nick/code/personal/dx/");
        assert_eq!(filtered, candidates);
    }

    #[test]
    fn tilde_matches_absolute_candidate() {
        if let Some(home) = dirs::home_dir() {
            let candidate = home.join("projects");
            let filtered = filter_candidates(std::slice::from_ref(&candidate), "~/projects");
            assert_eq!(filtered, vec![candidate]);
        }
    }

    #[test]
    fn dot_slash_prefix_matches_by_basename() {
        let candidates = vec![PathBuf::from("/some/deep/path/src")];
        let filtered = filter_candidates(&candidates, "./src");
        assert_eq!(filtered, candidates);
    }

    #[test]
    fn whitespace_in_query_is_significant() {
        let candidates = vec![
            PathBuf::from("/tmp/ project "),
            PathBuf::from("/tmp/project"),
        ];

        assert_eq!(
            filter_candidates(&candidates, " project "),
            vec![candidates[0].clone()]
        );
        assert_eq!(
            filter_candidates(&candidates, " project"),
            vec![candidates[0].clone()]
        );
    }

    #[cfg(unix)]
    #[test]
    fn unix_backslash_query_matches_filename_character() {
        let candidates = vec![PathBuf::from("/tmp/project\\source")];

        assert_eq!(
            filter_candidates(&candidates, "project\\source"),
            candidates
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_backslash_trailing_separator_preserves_root_selector() {
        assert_eq!(normalize_query(r"C:\"), b"c:\\");
    }
}
