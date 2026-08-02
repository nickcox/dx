//! `CliError`, the one type every handler returns. Its `Display` is the exact
//! stderr line, so `cli::run` is the only place that formats an error.

use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::bookmarks::{BookmarkError, storage::StorageError as BookmarkStorageError};
use crate::complete::SelectorError;
use crate::config::ConfigError;
use crate::hooks::{MenuCommandMappingError, PwshMenuKeyError};
use crate::resolve::ResolveError;
use crate::stacks::{StackError, storage::StorageError as StackStorageError};

/// Every failure that can end a `dx` subcommand.
///
/// `Display` renders the exact line written to stderr, including the
/// `dx <command>:` prefix, so [`super::run`] is the only place that formats a
/// diagnostic or chooses an exit code.
#[derive(Debug, Error)]
pub enum CliError {
    #[error("dx: {0}")]
    Config(#[from] ConfigError),

    #[error("dx resolve: {0}")]
    Resolve(#[from] ResolveError),
    #[error("dx resolve: ambiguous query; candidates:{}", bullet_list(.0))]
    AmbiguousResolve(Vec<PathBuf>),
    #[error("dx resolve: failed to read current directory: {0}")]
    ResolveCurrentDir(#[source] io::Error),
    #[error("dx resolve: failed to serialize json: {0}")]
    ResolveJson(#[source] serde_json::Error),
    /// Resolution failed in a machine-readable mode (`--list` or `--json`), so
    /// the outcome is already on stdout. Carries no stderr output — see
    /// [`CliError::is_silent`].
    #[error("resolve reported the failure on stdout")]
    ResolveReportedOnStdout,

    #[error("dx complete: failed to serialize json: {0}")]
    CompleteJson(#[source] serde_json::Error),

    #[error("dx navigate: {0}")]
    Navigate(#[from] SelectorError),

    #[error("dx stack: {0}")]
    Stack(#[from] StackError),
    #[error("dx stack: {0}")]
    StackStorage(#[from] StackStorageError),
    #[error("dx stack: failed to serialize json: {0}")]
    StackJson(#[source] serde_json::Error),
    #[error("dx stack: missing session id (use --session or DX_SESSION)")]
    MissingSessionId,
    #[error("dx stack: cannot combine --list/--clear with subcommands")]
    StackFlagsWithSubcommand,
    #[error("dx stack: cannot combine --list and --clear")]
    StackListAndClear,
    #[error("dx stack: provide one of --list, --clear, or a subcommand")]
    StackNoAction,
    #[error("dx stack: target must be an absolute path: {0}")]
    StackTargetNotAbsolute(String),
    #[error("dx stack: target not reachable: {}", .0.display())]
    StackTargetUnreachable(PathBuf),
    #[error("dx stack push: path was empty")]
    StackPushEmptyPath,
    #[error("dx stack push: failed to read current directory: {0}")]
    StackPushCurrentDir(#[source] io::Error),

    #[error("dx bookmarks: {0}")]
    Bookmark(#[from] BookmarkError),
    #[error("dx bookmarks: {0}")]
    BookmarkStorage(#[from] BookmarkStorageError),
    #[error("dx bookmarks: failed to serialize json: {0}")]
    BookmarksJson(#[source] serde_json::Error),
    #[error("dx bookmarks: failed to read current directory: {0}")]
    BookmarksCurrentDir(#[source] io::Error),

    #[error("dx init: --native-menu is only supported for pwsh")]
    NativeMenuRequiresPwsh,
    #[error("dx init: invalid DX_MENU_COMMAND_MAPPINGS: {0}")]
    MenuCommandMappings(#[from] MenuCommandMappingError),
    #[error("dx init: invalid DX_PWSH_MENU_KEY: {0}")]
    PwshMenuKey(#[from] PwshMenuKeyError),
}

impl CliError {
    /// True when the failure has already been communicated on stdout and must
    /// not be repeated on stderr. The exit code is still non-zero.
    pub fn is_silent(&self) -> bool {
        matches!(self, Self::ResolveReportedOnStdout)
    }
}

/// Renders candidates as a leading-newline bullet list, or an empty string when
/// there are none, so the header line never gains a stray trailing newline.
fn bullet_list(candidates: &[PathBuf]) -> String {
    candidates
        .iter()
        .map(|candidate| format!("\n- {}", candidate.display()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambiguous_resolve_renders_one_candidate_per_line() {
        let error = CliError::AmbiguousResolve(vec![
            PathBuf::from("/work/proj/alpha"),
            PathBuf::from("/work/prod/alpha"),
        ]);

        assert_eq!(
            error.to_string(),
            "dx resolve: ambiguous query; candidates:\n- /work/proj/alpha\n- /work/prod/alpha"
        );
    }

    #[test]
    fn ambiguous_resolve_without_candidates_keeps_header_on_one_line() {
        let error = CliError::AmbiguousResolve(Vec::new());

        assert_eq!(
            error.to_string(),
            "dx resolve: ambiguous query; candidates:"
        );
    }

    #[test]
    fn stdout_reported_failures_are_silent() {
        assert!(CliError::ResolveReportedOnStdout.is_silent());
        assert!(!CliError::MissingSessionId.is_silent());
        assert!(!CliError::Resolve(ResolveError::NotFound).is_silent());
    }

    #[test]
    fn domain_errors_keep_their_command_prefix() {
        assert_eq!(
            CliError::Resolve(ResolveError::NotFound).to_string(),
            "dx resolve: unable to resolve query"
        );
        assert_eq!(
            CliError::Stack(StackError::NothingToUndo).to_string(),
            "dx stack: nothing to undo"
        );
        assert_eq!(
            CliError::Navigate(SelectorError::OutOfRange { index: 3, total: 1 }).to_string(),
            "dx navigate: selector index 3 out of range (1..=1)"
        );
        assert_eq!(
            CliError::Bookmark(BookmarkError::NotFound("proj".to_string())).to_string(),
            "dx bookmarks: bookmark not found: proj"
        );
    }
}
