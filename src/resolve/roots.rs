use std::fs;
use std::path::{Path, PathBuf};

use super::abbreviation::matches_prefix;

pub fn resolve_fallbacks(roots: &[PathBuf], query: &str, case_sensitive: bool) -> Vec<PathBuf> {
    let has_slash = query.contains('/');
    let segments = query
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

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
            matches.extend(resolve_segment_path(root, &segments, case_sensitive));
        } else {
            matches.extend(resolve_single_segment(root, query, case_sensitive));
        }
    }

    matches.sort();
    matches.dedup();
    matches
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
            if matches_prefix(name, segment, case_sensitive) {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

fn resolve_segment_path(root: &Path, segments: &[&str], case_sensitive: bool) -> Vec<PathBuf> {
    let mut current = vec![root.to_path_buf()];

    for segment in segments {
        let mut next = Vec::new();
        for base in &current {
            let Ok(entries) = fs::read_dir(base) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = entry.file_name();
                let Some(name) = name.to_str() else {
                    continue;
                };
                if matches_prefix(name, segment, case_sensitive) {
                    next.push(path);
                }
            }
        }
        current = next;
        if current.is_empty() {
            break;
        }
    }

    current
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn make_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dx-{label}-{nonce}-{}", std::process::id()));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn resolves_exact_match_in_root() {
        let temp = make_temp_dir("roots-exact");
        let root = temp.join("root");
        let target = root.join("myproject");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = resolve_fallbacks(&[root], "myproject", true);
        assert_eq!(matches, vec![target]);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn resolves_abbreviated_path_in_root() {
        let temp = make_temp_dir("roots-abbrev");
        let root = temp.join("root");
        let target = root.join("project/src/components");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = resolve_fallbacks(&[root], "pro/sr/com", true);
        assert_eq!(matches, vec![target]);
        let _ = fs::remove_dir_all(temp);
    }
}
