//! What a menu invocation is completing, and the path style its query implies.

use crate::complete::CompletionMode;
use crate::complete::filesystem::FilesystemCompletionKind;
use crate::resolve::path_query::{PathQuery, QueryKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuMode {
    Completion(CompletionMode),
    Path,
    Directory,
    File,
}

impl From<FilesystemCompletionKind> for MenuMode {
    fn from(kind: FilesystemCompletionKind) -> Self {
        match kind {
            FilesystemCompletionKind::Path => Self::Path,
            FilesystemCompletionKind::Directory => Self::Directory,
            FilesystemCompletionKind::File => Self::File,
        }
    }
}

/// The explicit filesystem anchor expressed by the active query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryStyle {
    Compact,
    BareRelative,
    DotRelative,
    ParentRelative,
    HomeRelative,
    Absolute,
}

impl QueryStyle {
    pub fn from_query(mode: MenuMode, query: &str) -> Self {
        if !mode.prefers_query_relative_rendering() {
            return Self::Compact;
        }

        match PathQuery::new(query).kind {
            QueryKind::Home => Self::HomeRelative,
            QueryKind::Absolute | QueryKind::RootRelative | QueryKind::DriveRelative => {
                Self::Absolute
            }
            QueryKind::ExplicitRelative if query == ".." || query.starts_with("../") => {
                Self::ParentRelative
            }
            QueryKind::ExplicitRelative => Self::DotRelative,
            QueryKind::Plain => Self::BareRelative,
        }
    }
}

impl MenuMode {
    pub fn completion(mode: CompletionMode) -> Self {
        Self::Completion(mode)
    }

    pub fn is_directory_drill_in(self) -> bool {
        matches!(
            self,
            Self::Completion(CompletionMode::Paths) | Self::Directory
        )
    }

    pub fn prefers_query_relative_rendering(self) -> bool {
        matches!(
            self,
            Self::Completion(CompletionMode::Paths) | Self::Path | Self::Directory | Self::File
        )
    }

    /// Whether candidates must be canonicalised to drop the cwd itself. Listed
    /// exhaustively rather than with a wildcard so a new mode has to choose:
    /// canonicalising costs ~12us per candidate, which the high-volume
    /// filesystem modes cannot absorb.
    pub fn needs_cwd_filtering(self) -> bool {
        match self {
            Self::Completion(CompletionMode::Paths) | Self::Path | Self::Directory | Self::File => {
                false
            }
            Self::Completion(
                CompletionMode::Ancestors
                | CompletionMode::Frecents
                | CompletionMode::Recents
                | CompletionMode::Stack(_),
            ) => true,
        }
    }

    pub fn is_mapped_filesystem_mode(self) -> bool {
        self.filesystem_kind().is_some()
    }

    /// The filesystem kind this mode scans for, or `None` for the built-in
    /// completion modes, which source candidates from `dx complete` instead.
    pub fn filesystem_kind(self) -> Option<FilesystemCompletionKind> {
        match self {
            Self::Path => Some(FilesystemCompletionKind::Path),
            Self::Directory => Some(FilesystemCompletionKind::Directory),
            Self::File => Some(FilesystemCompletionKind::File),
            Self::Completion(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MenuMode, QueryStyle};
    use crate::complete::CompletionMode;

    /// Canonicalising every candidate is the expensive path; the filesystem
    /// modes that produce the most candidates must stay off it.
    #[test]
    fn only_the_low_volume_modes_filter_the_cwd() {
        use crate::complete::StackDirection;

        for mode in [
            MenuMode::Completion(CompletionMode::Paths),
            MenuMode::Path,
            MenuMode::Directory,
            MenuMode::File,
        ] {
            assert!(
                !mode.needs_cwd_filtering(),
                "{mode:?} must not canonicalise"
            );
        }
        for mode in [
            MenuMode::Completion(CompletionMode::Ancestors),
            MenuMode::Completion(CompletionMode::Frecents),
            MenuMode::Completion(CompletionMode::Recents),
            MenuMode::Completion(CompletionMode::Stack(StackDirection::Back)),
        ] {
            assert!(mode.needs_cwd_filtering(), "{mode:?} must filter the cwd");
        }
    }

    #[test]
    fn filesystem_query_styles_preserve_explicit_anchors() {
        let mode = MenuMode::Completion(CompletionMode::Paths);
        assert_eq!(
            QueryStyle::from_query(mode, "src"),
            QueryStyle::BareRelative
        );
        assert_eq!(QueryStyle::from_query(mode, "."), QueryStyle::DotRelative);
        assert_eq!(
            QueryStyle::from_query(mode, "./src"),
            QueryStyle::DotRelative
        );
        assert_eq!(
            QueryStyle::from_query(mode, ".."),
            QueryStyle::ParentRelative
        );
        assert_eq!(
            QueryStyle::from_query(mode, "../../src"),
            QueryStyle::ParentRelative
        );
        assert_eq!(QueryStyle::from_query(mode, "~"), QueryStyle::HomeRelative);
        assert_eq!(
            QueryStyle::from_query(mode, "~/src"),
            QueryStyle::HomeRelative
        );
        assert_eq!(
            QueryStyle::from_query(mode, "/tmp/src"),
            QueryStyle::Absolute
        );
    }
}
