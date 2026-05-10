use crate::complete::CompletionMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuMode {
    Completion(CompletionMode),
    Path,
    Directory,
    File,
}

impl MenuMode {
    pub fn completion(mode: CompletionMode) -> Self {
        Self::Completion(mode)
    }

    pub fn is_directory_drill_in(self) -> bool {
        matches!(self, Self::Completion(CompletionMode::Paths) | Self::Directory)
    }

    pub fn prefers_query_relative_rendering(self) -> bool {
        matches!(self, Self::Completion(CompletionMode::Paths) | Self::Path | Self::Directory | Self::File)
    }

    pub fn is_mapped_filesystem_mode(self) -> bool {
        matches!(self, Self::Path | Self::Directory | Self::File)
    }
}
