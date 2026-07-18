use std::fs;
use std::path::{Path, PathBuf};

use super::{abbreviation::matches_segment, path_query, traversal};

pub fn resolve_fallbacks(roots: &[PathBuf], query: &str, case_sensitive: bool) -> Vec<PathBuf> {
    let has_slash = path_query::has_separator(query);
    let segments = path_query::segments(query);

    let mut matches = Vec::new();

    for root in roots {
        if !root.is_dir() {
            continue;
        }

        if !has_slash {
            let direct = root.join(query);
            if direct.is_dir() {
                matches.push(direct);
            }
        }

        if has_slash {
            matches.extend(traversal::traverse_segment_paths(
                vec![root.to_path_buf()],
                &segments,
                |name, segment| matches_segment(name, segment, case_sensitive),
            ));
        } else {
            matches.extend(resolve_single_segment(root, query, case_sensitive));
        }
    }

    matches.sort();
    matches.dedup();
    matches
}

pub fn resolve_fallbacks_exact(
    roots: &[PathBuf],
    query: &str,
    case_sensitive: bool,
) -> Result<Vec<PathBuf>, (PathBuf, std::io::Error)> {
    let has_separator = path_query::has_separator(query);
    let segments = path_query::segments(query);
    let mut matches = Vec::new();

    for root in roots {
        if !root.is_dir() {
            continue;
        }
        if !has_separator {
            let direct = root.join(query);
            match fs::metadata(&direct) {
                Ok(metadata) if metadata.is_dir() => matches.push(direct),
                Ok(_) | Err(_) => {}
            }
        }
        if has_separator {
            matches.extend(traversal::try_traverse_segment_paths(
                vec![root.clone()],
                &segments,
                |name, segment| matches_segment(name, segment, case_sensitive),
            )?);
        } else {
            matches.extend(resolve_single_segment_exact(root, query, case_sensitive)?);
        }
    }
    matches.sort();
    matches.dedup();
    Ok(matches)
}

fn resolve_single_segment(root: &Path, segment: &str, case_sensitive: bool) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };

    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }
            let name = entry.file_name();
            let name = name.to_str()?;
            if matches_segment(name, segment, case_sensitive) {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

fn resolve_single_segment_exact(
    root: &Path,
    segment: &str,
    case_sensitive: bool,
) -> Result<Vec<PathBuf>, (PathBuf, std::io::Error)> {
    let entries = fs::read_dir(root).map_err(|source| (root.to_path_buf(), source))?;
    let mut matches = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| (root.to_path_buf(), source))?;
        let path = entry.path();
        let name = entry.file_name();
        if let Some(name) = name.to_str()
            && matches_segment(name, segment, case_sensitive)
        {
            let metadata = entry.metadata().map_err(|source| (path.clone(), source))?;
            if metadata.is_dir() {
                matches.push(path);
            }
        }
    }
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

        let matches = resolve_fallbacks(&[root], "myproject", true);
        assert_eq!(matches, vec![target]);
    }

    #[test]
    fn resolves_abbreviated_path_in_root() {
        let temp: TempDir = temp_dir("roots-abbrev");
        let root = temp.path().join("root");
        let target = root.join("project/src/components");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = resolve_fallbacks(&[root], "pro/sr/com", true);
        assert_eq!(matches, vec![target]);
    }

    #[test]
    fn resolves_delimiter_aware_single_segment_match_in_root() {
        let temp: TempDir = temp_dir("roots-delimiter");
        let root = temp.path().join("root");
        let target = root.join("cd-extras");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = resolve_fallbacks(&[root], "cd-e", true);
        assert_eq!(matches, vec![target]);
    }

    #[test]
    fn resolves_delimiter_aware_multi_segment_match_in_root() {
        let temp: TempDir = temp_dir("roots-delimiter-multi");
        let root = temp.path().join("root");
        let target = root.join("project/PowerShell/src/Microsoft.PowerShell.SDK");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = resolve_fallbacks(&[root], "pro/p..shell/s/.sdk", false);
        assert_eq!(matches, vec![target]);
    }
}
