use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

/// How a directory walk reacts to a filesystem error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnIoError {
    /// Skip whatever could not be read and keep going. Completion prefers a
    /// partial candidate list over none at all.
    Skip,
    /// Abort on the first failure. `resolve` must not silently narrow the
    /// candidate set before calling a query unresolvable or ambiguous.
    Propagate,
}

/// The path that could not be read, paired with the underlying error.
pub type TraversalError = (PathBuf, io::Error);

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
    let mut normal_components = 0;
    let mut anchored = false;

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if normal_components > 0 {
                    normalized.pop();
                    normal_components -= 1;
                } else if !anchored {
                    normalized.push(component.as_os_str());
                }
            }
            Component::Prefix(_) | Component::RootDir => {
                anchored = true;
                normalized.push(component.as_os_str());
            }
            Component::Normal(_) => {
                normalized.push(component.as_os_str());
                normal_components += 1;
            }
        }
    }

    normalized
}

/// Walks `bases` down one level per segment, keeping the directories whose name
/// matches that segment.
///
/// Directory-ness is decided by [`is_directory`], which follows symlinks, so a
/// symlink pointing at a directory is a valid navigation target — the same view
/// the shell takes when it runs `cd`.
pub fn traverse_segment_paths<S, F>(
    bases: Vec<PathBuf>,
    segments: &[S],
    matches_segment: F,
    on_error: OnIoError,
) -> Result<Vec<PathBuf>, TraversalError>
where
    F: Fn(&str, &S) -> bool,
{
    let mut current = bases;

    for segment in segments {
        let mut next = Vec::new();
        for base in &current {
            let Some(entries) = apply_io_policy(fs::read_dir(base), base, on_error)? else {
                continue;
            };

            for entry in entries {
                let Some(entry) = apply_io_policy(entry, base, on_error)? else {
                    continue;
                };
                let name = entry.file_name();
                let Some(name) = name.to_str() else {
                    continue;
                };
                if !matches_segment(name, segment) {
                    continue;
                }

                // Match the name before stat-ing, so only candidate entries
                // cost a syscall in large directories.
                let path = entry.path();
                if is_directory(&path, on_error)? {
                    next.push(path);
                }
            }
        }

        current = next;
        if current.is_empty() {
            break;
        }
    }

    Ok(current)
}

/// Whether `path` is a directory, following symlinks.
///
/// A path that does not exist — a dangling symlink, say — is not a directory
/// rather than an error, in either policy.
pub fn is_directory(path: &Path, on_error: OnIoError) -> Result<bool, TraversalError> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(source) => match on_error {
            OnIoError::Skip => Ok(false),
            OnIoError::Propagate => Err((path.to_path_buf(), source)),
        },
    }
}

/// Applies `on_error` to a filesystem result. `Ok(None)` means "skip this and
/// carry on"; `Err` means the caller must abort the walk.
fn apply_io_policy<T>(
    result: io::Result<T>,
    path: &Path,
    on_error: OnIoError,
) -> Result<Option<T>, TraversalError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(source) => match on_error {
            OnIoError::Skip => Ok(None),
            OnIoError::Propagate => Err((path.to_path_buf(), source)),
        },
    }
}

fn is_multi_dot_alias(input: &str) -> bool {
    input.len() >= 3 && input.chars().all(|c| c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{TempDir, temp_dir};
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
    fn lexical_normalization_preserves_empty_relative_paths() {
        assert_eq!(normalize_path(Path::new(".")), PathBuf::new());
        assert_eq!(
            normalize_path(Path::new("../../work")),
            PathBuf::from("../../work")
        );
    }

    #[cfg(unix)]
    #[test]
    fn lexical_normalization_does_not_escape_unix_root() {
        assert_eq!(
            normalize_path(Path::new("/../../work")),
            PathBuf::from("/work")
        );
    }

    #[cfg(windows)]
    #[test]
    fn lexical_normalization_preserves_windows_roots() {
        assert_eq!(
            normalize_path(Path::new(r"C:\..\work")),
            PathBuf::from(r"C:\work")
        );
        assert_eq!(
            normalize_path(Path::new(r"\\server\share\..\work")),
            PathBuf::from(r"\\server\share\work")
        );
    }

    #[test]
    fn ignores_non_alias_inputs() {
        let cwd = PathBuf::from("/tmp/a/b");
        assert!(resolve_step_up(&cwd, ".. ").is_none());
        assert!(resolve_step_up(&cwd, "abc").is_none());
    }

    fn prefix_matcher(name: &str, segment: &&str) -> bool {
        name.starts_with(*segment)
    }

    #[test]
    fn traverses_multi_segment_paths_with_callback_matcher() {
        let temp: TempDir = temp_dir("traversal-case");
        let base = temp.path().join("root");
        let target = base.join("Project/Source");
        fs::create_dir_all(&target).expect("create dirs");

        let matches = traverse_segment_paths(
            vec![base],
            &["pro", "sou"],
            |name, segment| {
                name.to_ascii_lowercase()
                    .starts_with(&segment.to_ascii_lowercase())
            },
            OnIoError::Skip,
        )
        .expect("skip policy never fails");

        assert_eq!(matches, vec![target]);
    }

    #[test]
    fn preserves_base_order_for_matches() {
        let temp: TempDir = temp_dir("traversal-order");
        let root_a = temp.path().join("a");
        let root_b = temp.path().join("b");
        let target_a = root_a.join("project/src");
        let target_b = root_b.join("project/src");
        fs::create_dir_all(&target_a).expect("create dirs");
        fs::create_dir_all(&target_b).expect("create dirs");

        let matches = traverse_segment_paths(
            vec![root_a, root_b],
            &["pro", "sr"],
            prefix_matcher,
            OnIoError::Skip,
        )
        .expect("skip policy never fails");

        assert_eq!(matches, vec![target_a, target_b]);
    }

    #[cfg(unix)]
    #[test]
    fn both_policies_follow_symlinks_to_directories() {
        use std::os::unix::fs::symlink;

        let temp: TempDir = temp_dir("traversal-symlink");
        let base = temp.path().join("root");
        let real = base.join("real");
        fs::create_dir_all(real.join("inner")).expect("create dirs");
        symlink(&real, base.join("linked")).expect("create directory symlink");

        for policy in [OnIoError::Skip, OnIoError::Propagate] {
            let matches = traverse_segment_paths(
                vec![base.clone()],
                &["linked", "inn"],
                prefix_matcher,
                policy,
            )
            .expect("symlinked directories traverse");

            assert_eq!(
                matches,
                vec![base.join("linked/inner")],
                "policy {policy:?} should follow a directory symlink"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn both_policies_skip_dangling_symlinks() {
        use std::os::unix::fs::symlink;

        let temp: TempDir = temp_dir("traversal-dangling");
        let base = temp.path().join("root");
        fs::create_dir_all(base.join("present")).expect("create dirs");
        symlink(base.join("missing"), base.join("phantom")).expect("create dangling symlink");

        for policy in [OnIoError::Skip, OnIoError::Propagate] {
            let matches =
                traverse_segment_paths(vec![base.clone()], &["p"], prefix_matcher, policy)
                    .expect("a dangling symlink is not an error");

            assert_eq!(matches, vec![base.join("present")], "policy {policy:?}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn policies_diverge_only_on_unreadable_directories() {
        use std::os::unix::fs::PermissionsExt;

        let temp: TempDir = temp_dir("traversal-unreadable");
        let base = temp.path().join("root");
        fs::create_dir_all(base.join("project/src")).expect("create dirs");
        fs::set_permissions(base.join("project"), fs::Permissions::from_mode(0o000))
            .expect("make directory unreadable");

        let skipped = traverse_segment_paths(
            vec![base.clone()],
            &["pro", "sr"],
            prefix_matcher,
            OnIoError::Skip,
        )
        .expect("skip policy tolerates unreadable directories");
        assert!(skipped.is_empty());

        let propagated = traverse_segment_paths(
            vec![base.clone()],
            &["pro", "sr"],
            prefix_matcher,
            OnIoError::Propagate,
        );
        let (path, _) = propagated.expect_err("propagate policy surfaces the failure");
        assert_eq!(path, base.join("project"));

        fs::set_permissions(base.join("project"), fs::Permissions::from_mode(0o755))
            .expect("restore permissions for cleanup");
    }
}
