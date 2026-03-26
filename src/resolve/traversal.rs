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

fn is_multi_dot_alias(input: &str) -> bool {
    input.len() >= 3 && input.chars().all(|c| c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
