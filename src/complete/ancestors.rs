use std::env;
use std::path::{Path, PathBuf};

use super::filter::filter_candidates;

pub fn complete(query: Option<&str>) -> Vec<PathBuf> {
    let cwd = match env::current_dir() {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    complete_from(&cwd, query)
}

pub fn complete_from(cwd: &Path, query: Option<&str>) -> Vec<PathBuf> {
    let mut output = Vec::new();
    let mut cursor = cwd;

    while let Some(parent) = cursor.parent() {
        if parent == cursor {
            break;
        }
        output.push(parent.to_path_buf());
        cursor = parent;
    }

    match query.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => filter_candidates(&output, value),
        None => output,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::complete_from;

    #[test]
    fn full_ancestor_list_is_nearest_first() {
        let output = complete_from(PathBuf::from("/home/user/code/projects/dx").as_path(), None);
        assert_eq!(
            output,
            vec![
                PathBuf::from("/home/user/code/projects"),
                PathBuf::from("/home/user/code"),
                PathBuf::from("/home/user"),
                PathBuf::from("/home"),
                PathBuf::from("/")
            ]
        );
    }

    #[test]
    fn filtered_ancestor_list_uses_query_matching() {
        let output = complete_from(
            PathBuf::from("/home/user/code/projects/dx").as_path(),
            Some("code"),
        );
        assert_eq!(output[0], PathBuf::from("/home/user/code"));
        assert!(output.contains(&PathBuf::from("/home/user/code/projects")));
    }

    #[test]
    fn root_has_no_ancestors() {
        let output = complete_from(PathBuf::from("/").as_path(), None);
        assert!(output.is_empty());
    }
}
