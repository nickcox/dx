use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn resolve_step_up(cwd: &Path, query: &str) -> Option<PathBuf> {
    let trimmed = query.trim();
    if trimmed == "up" {
        return Some(cwd.parent().unwrap_or(cwd).to_path_buf());
    }

    if !is_multi_dot_alias(trimmed) {
        return None;
    }

    let mut current = cwd.to_path_buf();
    let levels = trimmed.len().saturating_sub(1);
    for _ in 0..levels {
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        }
    }

    Some(normalize_path(&current))
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    if normalized.as_os_str().is_empty() {
        PathBuf::from("/")
    } else {
        normalized
    }
}

pub fn traverse_segment_paths<F>(
    bases: Vec<PathBuf>,
    segments: &[&str],
    matches_segment: F,
) -> Vec<PathBuf>
where
    F: Fn(&str, &str) -> bool,
{
    let mut current = bases;

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
                if matches_segment(name, segment) {
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

fn is_multi_dot_alias(input: &str) -> bool {
    input.len() >= 3 && input.chars().all(|c| c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolves_three_dots() {
        let cwd = PathBuf::from("/tmp/a/b/c");
        let result = resolve_step_up(&cwd, "...").expect("should resolve");
        assert_eq!(result, PathBuf::from("/tmp/a"));
    }

    #[test]
    fn resolves_n_dot_alias() {
        let cwd = PathBuf::from("/tmp/a/b/c/d");
        let result = resolve_step_up(&cwd, ".....").expect("should resolve");
        assert_eq!(result, PathBuf::from("/tmp"));
    }

    #[test]
    fn resolves_up_keyword() {
        let cwd = PathBuf::from("/tmp/a/b");
        let result = resolve_step_up(&cwd, "up").expect("should resolve");
        assert_eq!(result, PathBuf::from("/tmp/a"));
    }

    #[test]
    fn excessive_depth_stops_at_root() {
        let cwd = PathBuf::from("/");
        let result = resolve_step_up(&cwd, "......").expect("should resolve");
        assert_eq!(result, PathBuf::from("/"));
    }

    #[test]
    fn ignores_non_alias_inputs() {
        let cwd = PathBuf::from("/tmp/a/b");
        assert!(resolve_step_up(&cwd, ".. ").is_none());
        assert!(resolve_step_up(&cwd, "abc").is_none());
    }

    fn make_temp_dir(label: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dx-{label}-{nonce}-{}", std::process::id()));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn traverses_multi_segment_paths_with_callback_matcher() {
        let temp = make_temp_dir("traversal-case");
        let base = temp.join("root");
        let target = base.join("Project/Source");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = traverse_segment_paths(vec![base], &["pro", "sou"], |name, segment| {
            name.to_ascii_lowercase()
                .starts_with(&segment.to_ascii_lowercase())
        });

        assert_eq!(matches, vec![target]);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn preserves_base_order_for_matches() {
        let temp = make_temp_dir("traversal-order");
        let root_a = temp.join("a");
        let root_b = temp.join("b");
        let target_a = root_a.join("project/src");
        let target_b = root_b.join("project/src");
        fs::create_dir_all(&target_a).expect("create dirs");
        fs::create_dir_all(&target_b).expect("create dirs");

        let matches =
            traverse_segment_paths(vec![root_a, root_b], &["pro", "sr"], |name, segment| {
                name.starts_with(segment)
            });

        assert_eq!(matches, vec![target_a, target_b]);
        let _ = fs::remove_dir_all(temp);
    }
}
