use std::io;
use std::path::{Path, PathBuf};

use super::path_query::{PathQuery, QueryKind, root_anchor};
use super::traversal::normalize_path;

pub fn resolve_direct(cwd: &Path, query: PathQuery<'_>) -> Result<Option<PathBuf>, io::Error> {
    let path = match query.kind {
        QueryKind::Home => resolve_home(query.raw()),
        QueryKind::Absolute => Some(PathBuf::from(query.raw())),
        QueryKind::RootRelative => {
            let anchor = root_anchor(cwd).expect("root-relative paths require a rooted cwd");
            Some(
                anchor.join(
                    query
                        .fallback_segments()
                        .join(std::path::MAIN_SEPARATOR_STR),
                ),
            )
        }
        QueryKind::ExplicitRelative => Some(cwd.join(query.raw())),
        QueryKind::DriveRelative => return Ok(None),
        QueryKind::Plain => {
            let candidate = cwd.join(query.raw());
            match std::fs::metadata(&candidate) {
                Ok(metadata) if metadata.is_dir() => Some(candidate),
                Ok(_) => None,
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(error) => return Err(error),
            }
        }
    };
    Ok(path.map(|path| normalize_path(&path)))
}

fn resolve_home(query: &str) -> Option<PathBuf> {
    let home = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(dirs::home_dir);

    if query == "~" {
        return home;
    }

    if let Some(rest) = query.strip_prefix('~')
        && rest.chars().next().is_some_and(std::path::is_separator)
    {
        return home.map(|home| home.join(rest.trim_start_matches(std::path::is_separator)));
    }

    None
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::test_support;

    #[test]
    fn resolves_absolute_paths() {
        let cwd = PathBuf::from("/");
        let result = resolve_direct(&cwd, PathQuery::new("/tmp/../tmp"))
            .expect("resolve")
            .expect("result");
        assert_eq!(result, PathBuf::from("/tmp"));
    }

    #[test]
    fn resolves_relative_paths() {
        let temp = test_support::temp_dir("precedence-rel");
        let cwd = temp.path().join("work");
        fs::create_dir_all(cwd.join("src")).expect("create dirs");

        let result = resolve_direct(&cwd, PathQuery::new("./src"))
            .expect("resolve")
            .expect("result");
        assert_eq!(result, cwd.join("src"));
    }

    #[test]
    fn resolves_direct_child_path() {
        let temp = test_support::temp_dir("precedence-child");
        let cwd = temp.path().join("work");
        fs::create_dir_all(cwd.join("src")).expect("create dirs");

        let result = resolve_direct(&cwd, PathQuery::new("src"))
            .expect("resolve")
            .expect("result");
        assert_eq!(result, cwd.join("src"));
    }

    #[test]
    fn resolves_home_paths() {
        let mut process = test_support::ScopedProcess::new();
        process.set("HOME", "/tmp/home-test");

        let resolved_home = resolve_direct(Path::new("/"), PathQuery::new("~"))
            .expect("resolve")
            .expect("home result");
        assert_eq!(resolved_home, PathBuf::from("/tmp/home-test"));

        let resolved_child = resolve_direct(Path::new("/"), PathQuery::new("~/work"))
            .expect("resolve")
            .expect("child result");
        assert_eq!(resolved_child, PathBuf::from("/tmp/home-test/work"));
    }
}
