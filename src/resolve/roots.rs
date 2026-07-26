//! Searching the configured roots when a query does not resolve against the
//! cwd.

use std::path::PathBuf;

use super::traversal::{OnIoError, TraversalError};
use super::{abbreviation::matches_segment, path_query, traversal};

pub fn resolve_fallbacks(
    roots: &[PathBuf],
    query: &str,
    case_sensitive: bool,
    on_error: OnIoError,
) -> Result<Vec<PathBuf>, TraversalError> {
    let has_separator = path_query::has_separator(query);
    let segments = path_query::segments(query);

    let mut matches = Vec::new();

    for root in roots {
        if !root.is_dir() {
            continue;
        }

        // A single-segment query can name a child outright; the segment scan
        // below only ever matches under the abbreviation rules.
        if !has_separator {
            let direct = root.join(query);
            if traversal::is_directory(&direct, on_error)? {
                matches.push(direct);
            }
        }

        matches.extend(traversal::traverse_segment_paths(
            vec![root.clone()],
            &segments,
            |name, segment| matches_segment(name, segment, case_sensitive),
            on_error,
        )?);
    }

    matches.sort();
    matches.dedup();
    Ok(matches)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::test_support::{TempDir, temp_dir};

    #[test]
    fn resolves_exact_match_in_root() {
        let temp: TempDir = temp_dir("roots-exact");
        let root = temp.path().join("root");
        let target = root.join("myproject");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = resolve_fallbacks(&[root], "myproject", true, OnIoError::Skip)
            .expect("skip policy never fails");
        assert_eq!(matches, vec![target]);
    }

    #[test]
    fn resolves_abbreviated_path_in_root() {
        let temp: TempDir = temp_dir("roots-abbrev");
        let root = temp.path().join("root");
        let target = root.join("project/src/components");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = resolve_fallbacks(&[root], "pro/sr/com", true, OnIoError::Skip)
            .expect("skip policy never fails");
        assert_eq!(matches, vec![target]);
    }

    #[test]
    fn resolves_delimiter_aware_single_segment_match_in_root() {
        let temp: TempDir = temp_dir("roots-delimiter");
        let root = temp.path().join("root");
        let target = root.join("cd-extras");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = resolve_fallbacks(&[root], "cd-e", true, OnIoError::Skip)
            .expect("skip policy never fails");
        assert_eq!(matches, vec![target]);
    }

    #[test]
    fn resolves_delimiter_aware_multi_segment_match_in_root() {
        let temp: TempDir = temp_dir("roots-delimiter-multi");
        let root = temp.path().join("root");
        let target = root.join("project/PowerShell/src/Microsoft.PowerShell.SDK");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = resolve_fallbacks(&[root], "pro/p..shell/s/.sdk", false, OnIoError::Skip)
            .expect("skip policy never fails");
        assert_eq!(matches, vec![target]);
    }
}
