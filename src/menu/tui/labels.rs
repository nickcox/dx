//! Candidate labels for one menu session, rendered in the style the query
//! implies and cached so repeated keystrokes do not re-canonicalise.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::menu::QueryStyle;

/// Renders candidate labels for one menu session.
///
/// Labels are recomputed for every candidate on every keystroke.
/// Canonicalising each candidate measured ~35 ms per keystroke at 2000
/// candidates, so the cwd is resolved once and canonical parents are cached:
/// candidates share parents heavily, which turns that into roughly one call
/// per distinct directory.
pub(super) struct LabelContext<'a> {
    pub(super) cwd: &'a Path,
    pub(super) canonical_cwd: Option<PathBuf>,
    pub(super) home: Option<&'a Path>,
    pub(super) canonical_parents: HashMap<PathBuf, Option<PathBuf>>,
}

impl<'a> LabelContext<'a> {
    pub(super) fn new(cwd: &'a Path, home: Option<&'a Path>) -> Self {
        Self {
            cwd,
            canonical_cwd: std::fs::canonicalize(cwd).ok(),
            home,
            canonical_parents: HashMap::new(),
        }
    }

    pub(super) fn label(&mut self, path: &Path, style: QueryStyle) -> String {
        match style {
            QueryStyle::Compact => self.plain_label(path, true),
            QueryStyle::BareRelative => match self.cwd_relative(path, false) {
                Some(label) => label,
                None => self.plain_label(path, false),
            },
            QueryStyle::DotRelative => match self.cwd_relative(path, true) {
                Some(label) => label,
                None => self.plain_label(path, false),
            },
            QueryStyle::ParentRelative => {
                match crate::complete::parent_relative_path_from(self.cwd, path) {
                    Some(relative) => relative.display().to_string(),
                    None => self.plain_label(path, false),
                }
            }
            QueryStyle::HomeRelative => {
                match crate::complete::home_relative_label(path, self.home) {
                    Some(label) => label,
                    None => self.plain_label(path, false),
                }
            }
            QueryStyle::Absolute => path.display().to_string(),
        }
    }

    pub(super) fn plain_label(&mut self, path: &Path, prefer_relative_paths: bool) -> String {
        if prefer_relative_paths && let Some(relative) = self.relative_path(path) {
            use std::path::Component;

            return if relative.as_os_str().is_empty() {
                "./".to_string()
            } else if relative
                .components()
                .next()
                .is_some_and(|component| matches!(component, Component::ParentDir))
            {
                relative.display().to_string()
            } else {
                format!("./{}", relative.display())
            };
        }

        if let Some(home) = self.home
            && let Ok(relative) = path.strip_prefix(home)
        {
            return format!("~/{}", relative.display());
        }
        path.display().to_string()
    }

    pub(super) fn relative_path(&mut self, path: &Path) -> Option<PathBuf> {
        if let Ok(relative) = path.strip_prefix(self.cwd) {
            return Some(crate::complete::sanitize_relative_components(relative));
        }

        let canonical = self.canonical(path)?;
        let canonical_cwd = self.canonical_cwd.as_deref()?;
        canonical
            .strip_prefix(canonical_cwd)
            .ok()
            .map(crate::complete::sanitize_relative_components)
    }

    pub(super) fn cwd_relative(&mut self, path: &Path, dot_prefix: bool) -> Option<String> {
        if let Some(label) = crate::complete::cwd_relative_label(path, self.cwd, dot_prefix) {
            return Some(label);
        }

        let canonical = self.canonical(path)?;
        let canonical_cwd = self.canonical_cwd.as_deref()?;
        crate::complete::cwd_relative_label(&canonical, canonical_cwd, dot_prefix)
    }

    /// `path` with its parent resolved, reusing the result for siblings.
    ///
    /// A symlinked leaf is left unresolved. That can only change whether a
    /// label renders relative or absolute, never where selecting it leads.
    pub(super) fn canonical(&mut self, path: &Path) -> Option<PathBuf> {
        let parent = path.parent()?;
        let name = path.file_name()?;

        if let Some(cached) = self.canonical_parents.get(parent) {
            return Some(cached.as_ref()?.join(name));
        }

        let resolved = std::fs::canonicalize(parent).ok();
        self.canonical_parents
            .insert(parent.to_path_buf(), resolved.clone());
        Some(resolved?.join(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};

    use super::super::session::selected_status_path;

    /// The tests exercise labels one path at a time; production shares a
    /// `LabelContext` across the whole session.
    fn display_label_for_style(
        path: &Path,
        cwd: &Path,
        home: Option<&Path>,
        style: QueryStyle,
    ) -> String {
        LabelContext::new(cwd, home).label(path, style)
    }

    fn display_label(
        path: &Path,
        cwd: &Path,
        home: Option<&Path>,
        prefer_relative_paths: bool,
    ) -> String {
        LabelContext::new(cwd, home).plain_label(path, prefer_relative_paths)
    }

    #[test]
    fn display_label_relative_under_cwd() {
        let cwd = Path::new("/Users/nick");
        let path = Path::new("/Users/nick/Desktop");
        assert_eq!(display_label(path, cwd, None, true), "./Desktop");
    }

    #[test]
    fn cached_parent_resolution_labels_every_sibling_correctly() {
        let temp = crate::test_support::temp_dir("menu-label-parent-cache");
        let real_cwd = temp.path().join("real");
        let linked_cwd = temp.path().join("linked");
        let nested = real_cwd.join("group");
        fs::create_dir_all(nested.join("alpha")).expect("create alpha");
        fs::create_dir_all(nested.join("beta")).expect("create beta");
        symlink("real", &linked_cwd).expect("create cwd symlink");

        // One context, several candidates sharing a parent: the second and
        // third must come from the cache with the same answer as the first.
        let mut context = LabelContext::new(&linked_cwd, None);
        let labels: Vec<String> = [
            nested.join("alpha"),
            nested.join("beta"),
            nested.join("alpha"),
        ]
        .iter()
        .map(|path| context.label(path, QueryStyle::BareRelative))
        .collect();

        assert_eq!(labels, ["group/alpha", "group/beta", "group/alpha"]);
    }

    #[test]
    fn display_label_is_relative_for_equivalent_symlinked_cwd() {
        let temp = crate::test_support::temp_dir("menu-display-symlink-cwd");
        let real_cwd = temp.path().join("real");
        let linked_cwd = temp.path().join("linked");
        let path = real_cwd.join("documentation");
        fs::create_dir_all(&path).expect("create candidate directory");
        symlink("real", &linked_cwd).expect("create cwd symlink");

        assert_eq!(
            display_label(&path, &linked_cwd, None, true),
            "./documentation"
        );
        assert_eq!(
            display_label_for_style(&path, &linked_cwd, None, QueryStyle::BareRelative,),
            "documentation"
        );
    }

    #[test]
    fn candidate_label_style_from_query_only_applies_to_filesystem_modes() {
        assert_eq!(
            QueryStyle::from_query(
                crate::menu::MenuMode::Completion(crate::complete::CompletionMode::Paths),
                "",
            ),
            QueryStyle::BareRelative
        );
        assert_eq!(
            QueryStyle::from_query(
                crate::menu::MenuMode::Completion(crate::complete::CompletionMode::Frecents),
                "",
            ),
            QueryStyle::Compact
        );
    }

    #[test]
    fn candidate_label_style_from_query_detects_explicit_styles() {
        let mode = crate::menu::MenuMode::Completion(crate::complete::CompletionMode::Paths);

        assert_eq!(
            QueryStyle::from_query(mode, "src"),
            QueryStyle::BareRelative
        );
        assert_eq!(
            QueryStyle::from_query(mode, "./src"),
            QueryStyle::DotRelative
        );
        assert_eq!(
            QueryStyle::from_query(mode, "../src"),
            QueryStyle::ParentRelative
        );
        assert_eq!(
            QueryStyle::from_query(mode, "~/src"),
            QueryStyle::HomeRelative
        );
        assert_eq!(
            QueryStyle::from_query(mode, "/tmp/src"),
            QueryStyle::Absolute
        );
    }

    #[test]
    fn display_label_for_empty_query_uses_bare_cwd_relative_label() {
        let cwd = Path::new("/Users/nick/project");
        let path = Path::new("/Users/nick/project/src");

        assert_eq!(
            display_label_for_style(path, cwd, None, QueryStyle::BareRelative),
            "src"
        );
    }

    #[test]
    fn display_label_for_bare_query_uses_bare_cwd_relative_label() {
        let cwd = Path::new("/Users/nick/project");
        let path = Path::new("/Users/nick/project/src");

        assert_eq!(
            display_label_for_style(path, cwd, None, QueryStyle::BareRelative),
            "src"
        );
    }

    #[test]
    fn display_label_for_dot_query_preserves_dot_prefix() {
        let cwd = Path::new("/Users/nick/project");
        let path = Path::new("/Users/nick/project/src");

        assert_eq!(
            display_label_for_style(path, cwd, None, QueryStyle::DotRelative),
            "./src"
        );
    }

    #[test]
    fn display_label_for_parent_query_preserves_parent_prefix() {
        let cwd = Path::new("/Users/nick/project");
        let path = Path::new("/Users/nick/sibling");

        assert_eq!(
            display_label_for_style(path, cwd, None, QueryStyle::ParentRelative),
            "../sibling"
        );
    }

    #[test]
    fn display_label_for_parent_query_keeps_anchor_for_cwd_candidate() {
        let cwd = Path::new("/Users/nick/project");

        assert_eq!(
            display_label_for_style(cwd, cwd, None, QueryStyle::ParentRelative),
            "../project"
        );
    }

    #[test]
    fn display_label_for_parent_query_normalizes_candidate_parent_components() {
        let cwd = Path::new("/Users/nick/code/personal/dx");
        let path = Path::new("/Users/nick/code/personal/dx/../sibling");

        assert_eq!(
            display_label_for_style(path, cwd, None, QueryStyle::ParentRelative),
            "../sibling"
        );
    }

    #[test]
    fn display_label_for_multi_parent_query_preserves_parent_prefix() {
        let cwd = Path::new("/Users/nick/project/deep");
        let path = Path::new("/Users/nick/outer");

        assert_eq!(
            display_label_for_style(path, cwd, None, QueryStyle::ParentRelative),
            "../../outer"
        );
    }

    #[test]
    fn display_label_for_home_query_preserves_home_prefix() {
        let cwd = Path::new("/tmp");
        let home = Path::new("/Users/nick");
        let path = Path::new("/Users/nick/code");

        assert_eq!(
            display_label_for_style(path, cwd, Some(home), QueryStyle::HomeRelative),
            "~/code"
        );
    }

    #[test]
    fn display_label_for_absolute_query_preserves_absolute_path() {
        let cwd = Path::new("/tmp");
        let path = Path::new("/Users/nick/code");

        assert_eq!(
            display_label_for_style(path, cwd, None, QueryStyle::Absolute),
            "/Users/nick/code"
        );
    }

    #[test]
    fn compact_label_style_preserves_non_filesystem_mode_behavior() {
        let cwd = Path::new("/Users/nick/project");
        let path = Path::new("/Users/nick/project/src");

        assert_eq!(
            display_label_for_style(path, cwd, None, QueryStyle::Compact),
            "./src"
        );
    }

    #[test]
    fn display_label_tilde_when_under_home_but_not_cwd() {
        let cwd = Path::new("/tmp");
        let home = Path::new("/Users/nick");
        let path = Path::new("/Users/nick/code/dx");
        assert_eq!(display_label(path, cwd, Some(home), true), "~/code/dx");
    }

    #[test]
    fn display_label_absolute_when_outside_home() {
        let cwd = Path::new("/tmp");
        let home = Path::new("/Users/nick");
        let path = Path::new("/opt/homebrew/bin");
        assert_eq!(
            display_label(path, cwd, Some(home), true),
            "/opt/homebrew/bin"
        );
    }

    #[test]
    fn display_label_cwd_itself_shows_dot() {
        let cwd = Path::new("/Users/nick");
        let path = Path::new("/Users/nick");
        assert_eq!(display_label(path, cwd, None, true), "./");
    }

    #[test]
    fn display_label_paths_mode_relative_under_cwd_uses_dot_slash() {
        let cwd = Path::new("/tmp/work");
        let path = Path::new("/tmp/work/./benches");
        assert_eq!(display_label(path, cwd, None, true), "./benches");
    }

    #[test]
    fn display_label_paths_mode_parent_relative_prefix_is_preserved() {
        let cwd = Path::new("/tmp/work");
        let path = Path::new("/tmp/work/../sibling");
        assert_eq!(display_label(path, cwd, None, true), "../sibling");
    }

    #[test]
    fn display_label_paths_mode_multi_parent_relative_prefix_is_preserved() {
        let cwd = Path::new("/tmp/work");
        let path = Path::new("/tmp/work/../../outer");
        assert_eq!(display_label(path, cwd, None, true), "../../outer");
    }

    #[test]
    fn display_label_explicit_absolute_mode_preserves_absolute_path() {
        let cwd = Path::new("/tmp/work");
        let path = Path::new("/tmp/work/./benches");
        assert_eq!(display_label(path, cwd, None, false), "/tmp/work/./benches");
    }

    #[test]
    fn status_path_remains_full_when_item_label_is_query_style_relative() {
        let cwd = Path::new("/Users/nick/project");
        let path = PathBuf::from("/Users/nick/project/src");

        assert_eq!(
            display_label_for_style(&path, cwd, None, QueryStyle::BareRelative),
            "src"
        );
        assert_eq!(
            selected_status_path(&[path], Some(0)),
            "/Users/nick/project/src"
        );
    }
}
