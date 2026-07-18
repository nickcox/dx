use std::path::{Component, Path, PathBuf, is_separator};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryKind {
    Home,
    Absolute,
    RootRelative,
    ExplicitRelative,
    DriveRelative,
    Plain,
}

#[derive(Debug, Clone, Copy)]
pub struct PathQuery<'a> {
    raw: &'a str,
    pub kind: QueryKind,
}

impl<'a> PathQuery<'a> {
    pub fn new(raw: &'a str) -> Self {
        let path = Path::new(raw);
        let kind = if raw == "~" || raw.strip_prefix('~').is_some_and(starts_with_separator) {
            QueryKind::Home
        } else if path.is_absolute() {
            QueryKind::Absolute
        } else if path.has_root() {
            QueryKind::RootRelative
        } else if has_prefix(path) {
            QueryKind::DriveRelative
        } else if raw == "."
            || raw == ".."
            || raw.strip_prefix('.').is_some_and(starts_with_separator)
            || raw.strip_prefix("..").is_some_and(starts_with_separator)
        {
            QueryKind::ExplicitRelative
        } else {
            QueryKind::Plain
        };
        Self { raw, kind }
    }

    pub fn raw(self) -> &'a str {
        self.raw
    }

    pub fn is_filesystem_prefix(self) -> bool {
        !matches!(self.kind, QueryKind::Plain | QueryKind::DriveRelative)
    }

    pub fn has_trailing_separator(self) -> bool {
        self.raw.chars().next_back().is_some_and(is_separator)
    }

    pub fn fallback_segments(self) -> Vec<&'a str> {
        let mut segments = self.raw.split(is_separator).filter(|part| !part.is_empty());
        match self.kind {
            QueryKind::Home => {
                let _ = segments.next();
            }
            QueryKind::Absolute => {
                #[cfg(windows)]
                {
                    let _ = segments.next();
                    if self.raw.starts_with(r"\\") || self.raw.starts_with("//") {
                        let _ = segments.next();
                    }
                }
            }
            QueryKind::RootRelative => {}
            QueryKind::ExplicitRelative => {
                let _ = segments.next();
            }
            QueryKind::DriveRelative | QueryKind::Plain => {}
        }
        segments.collect()
    }

    pub fn root_anchor(self, cwd: &Path) -> Option<PathBuf> {
        match self.kind {
            QueryKind::Absolute => root_anchor(Path::new(self.raw)),
            QueryKind::RootRelative => root_anchor(cwd),
            _ => None,
        }
    }
}

pub fn segments(query: &str) -> Vec<&str> {
    query
        .split(is_separator)
        .filter(|part| !part.is_empty())
        .collect()
}

pub fn has_separator(query: &str) -> bool {
    query.chars().any(is_separator)
}

pub fn root_anchor(path: &Path) -> Option<PathBuf> {
    let mut anchor = PathBuf::new();
    let mut found_root = false;
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => anchor.push(prefix.as_os_str()),
            Component::RootDir => {
                anchor.push(component.as_os_str());
                found_root = true;
                break;
            }
            _ => break,
        }
    }
    found_root.then_some(anchor)
}

fn starts_with_separator(value: &str) -> bool {
    value.chars().next().is_some_and(is_separator)
}

fn has_prefix(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Prefix(_)))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    #[cfg(unix)]
    #[test]
    fn unix_backslash_is_a_filename_character() {
        let query = PathQuery::new("project\\source");
        assert_eq!(query.kind, QueryKind::Plain);
        assert_eq!(segments(query.raw()), vec!["project\\source"]);
    }

    #[cfg(unix)]
    #[test]
    fn unix_root_is_preserved() {
        assert_eq!(
            root_anchor(Path::new("/work/project")),
            Some(PathBuf::from("/"))
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_drive_and_unc_roots_are_preserved() {
        assert_eq!(
            root_anchor(Path::new(r"C:\work\project")),
            Some(PathBuf::from(r"C:\"))
        );
        assert_eq!(
            root_anchor(Path::new(r"\\server\share\work")),
            Some(PathBuf::from(r"\\server\share\"))
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_classifies_root_and_drive_relative_paths() {
        assert_eq!(
            PathQuery::new(r"\work\project").kind,
            QueryKind::RootRelative
        );
        assert_eq!(PathQuery::new("C:work").kind, QueryKind::DriveRelative);
    }
}
