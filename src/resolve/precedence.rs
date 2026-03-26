use std::env;
use std::path::{Path, PathBuf};

use super::traversal::normalize_path;

pub fn resolve_direct(cwd: &Path, query: &str) -> Option<PathBuf> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(path) = resolve_home(trimmed) {
        return Some(path);
    }

    if trimmed.starts_with('/') {
        return Some(normalize_path(Path::new(trimmed)));
    }

    if trimmed.starts_with("./") || trimmed == "." {
        return Some(normalize_path(&cwd.join(trimmed)));
    }

    if trimmed.starts_with("../") || trimmed == ".." {
        return Some(normalize_path(&cwd.join(trimmed)));
    }

    let direct_child = cwd.join(trimmed);
    if direct_child.is_dir() {
        return Some(normalize_path(&direct_child));
    }

    None
}

fn resolve_home(query: &str) -> Option<PathBuf> {
    if query == "~" {
        return env::var("HOME").ok().map(PathBuf::from);
    }

    if let Some(rest) = query.strip_prefix("~/") {
        return env::var("HOME")
            .ok()
            .map(|home| Path::new(&home).join(rest));
    }

    None
}

#[cfg(test)]
mod tests {
    use std::fs;
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
    fn resolves_absolute_paths() {
        let cwd = PathBuf::from("/");
        let result = resolve_direct(&cwd, "/tmp/../tmp").expect("result");
        assert_eq!(result, PathBuf::from("/tmp"));
    }

    #[test]
    fn resolves_relative_paths() {
        let temp = make_temp_dir("precedence-rel");
        let cwd = temp.join("work");
        fs::create_dir_all(cwd.join("src")).expect("create dirs");

        let result = resolve_direct(&cwd, "./src").expect("result");
        assert_eq!(result, cwd.join("src"));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn resolves_direct_child_path() {
        let temp = make_temp_dir("precedence-child");
        let cwd = temp.join("work");
        fs::create_dir_all(cwd.join("src")).expect("create dirs");

        let result = resolve_direct(&cwd, "src").expect("result");
        assert_eq!(result, cwd.join("src"));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn resolves_home_paths() {
        let previous = env::var("HOME").ok();
        env::set_var("HOME", "/tmp/home-test");

        let resolved_home = resolve_direct(Path::new("/"), "~").expect("home result");
        assert_eq!(resolved_home, PathBuf::from("/tmp/home-test"));

        let resolved_child = resolve_direct(Path::new("/"), "~/work").expect("child result");
        assert_eq!(resolved_child, PathBuf::from("/tmp/home-test/work"));

        if let Some(value) = previous {
            env::set_var("HOME", value);
        } else {
            env::remove_var("HOME");
        }
    }
}
