//! Walking directories segment by segment. The caller chooses whether an
//! unreadable directory is skipped or reported.

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
            let entries = match fs::read_dir(base) {
                Ok(entries) => entries,
                Err(source) if is_unenterable(base, &source) => continue,
                Err(source) => match on_error {
                    OnIoError::Skip => continue,
                    OnIoError::Propagate => return Err((base.to_path_buf(), source)),
                },
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
/// An entry that cannot be stat-ed is not a directory rather than an error, in
/// either policy: `cd` could reach neither a dangling symlink nor an entry the
/// process may not inspect, so declining them hides nothing.
pub fn is_directory(path: &Path, on_error: OnIoError) -> Result<bool, TraversalError> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(source) if is_unreachable(&source) => Ok(false),
        Err(source) => match on_error {
            OnIoError::Skip => Ok(false),
            OnIoError::Propagate => Err((path.to_path_buf(), source)),
        },
    }
}

fn is_unreachable(source: &io::Error) -> bool {
    matches!(
        source.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
    )
}

/// Whether a directory that would not list is also one the caller could never
/// have entered, making it safe to skip under either policy.
///
/// Entry without listing is what `Propagate` exists for: `cd` still works, so
/// failing to list can hide a real candidate. Denying both leaves nothing
/// reachable. Windows has no entry-without-listing state, which is how the
/// deny-ACL junctions in `%LOCALAPPDATA%` report — they sit beside `Temp` and
/// share its prefix, so every traversal there meets one.
fn is_unenterable(path: &Path, source: &io::Error) -> bool {
    if source.kind() != io::ErrorKind::PermissionDenied {
        return false;
    }

    #[cfg(unix)]
    {
        nix::unistd::access(path, nix::unistd::AccessFlags::X_OK).is_err()
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        true
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

    /// A directory that permits entry but not listing can still hold a `cd`-able
    /// candidate, so `Propagate` must say so rather than narrow the search.
    #[cfg(unix)]
    #[test]
    fn policies_diverge_on_enterable_but_unlistable_directories() {
        use std::os::unix::fs::PermissionsExt;

        let temp: TempDir = temp_dir("traversal-unlistable");
        let base = temp.path().join("root");
        fs::create_dir_all(base.join("project/src")).expect("create dirs");
        fs::set_permissions(base.join("project"), fs::Permissions::from_mode(0o111))
            .expect("make directory enterable but unlistable");

        let skipped = traverse_segment_paths(
            vec![base.clone()],
            &["pro", "sr"],
            prefix_matcher,
            OnIoError::Skip,
        )
        .expect("skip policy tolerates unlistable directories");
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

    /// Denying entry too leaves nothing the caller could have reached.
    #[cfg(unix)]
    #[test]
    fn unenterable_directories_abort_neither_policy() {
        use std::os::unix::fs::PermissionsExt;

        let temp: TempDir = temp_dir("traversal-unenterable");
        let base = temp.path().join("root");
        fs::create_dir_all(base.join("project/src")).expect("create dirs");
        fs::create_dir_all(base.join("probe/src")).expect("create dirs");
        fs::set_permissions(base.join("project"), fs::Permissions::from_mode(0o000))
            .expect("make directory unenterable");

        for policy in [OnIoError::Skip, OnIoError::Propagate] {
            let matches =
                traverse_segment_paths(vec![base.clone()], &["pro", "sr"], prefix_matcher, policy)
                    .expect("an unenterable directory is not an error");

            assert_eq!(
                matches,
                vec![base.join("probe/src")],
                "policy {policy:?} should skip past the unenterable sibling"
            );
        }

        fs::set_permissions(base.join("project"), fs::Permissions::from_mode(0o755))
            .expect("restore permissions for cleanup");
    }

    /// A sibling that cannot be stat-ed must not sink the whole walk.
    #[cfg(unix)]
    #[test]
    fn unstattable_siblings_do_not_abort_either_policy() {
        use std::os::unix::fs::PermissionsExt;

        let temp: TempDir = temp_dir("traversal-unstattable-sibling");
        let base = temp.path().join("root");
        let cloak = base.join("cloak");
        fs::create_dir_all(cloak.join("temporary-hidden")).expect("create dirs");
        fs::create_dir_all(cloak.join("temp/inner")).expect("create dirs");
        // Clearing search permission on the parent makes stat of either child
        // fail, which is the closest POSIX analogue of the Windows deny ACL.
        fs::set_permissions(&cloak, fs::Permissions::from_mode(0o444))
            .expect("drop search permission");

        for policy in [OnIoError::Skip, OnIoError::Propagate] {
            let matches =
                traverse_segment_paths(vec![cloak.clone()], &["temp"], prefix_matcher, policy)
                    .expect("an unstattable entry is not an error");

            assert!(
                matches.is_empty(),
                "policy {policy:?} kept an entry it could not stat: {matches:?}"
            );
        }

        fs::set_permissions(&cloak, fs::Permissions::from_mode(0o755))
            .expect("restore permissions for cleanup");
    }
}
