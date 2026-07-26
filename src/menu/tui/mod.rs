//! Interactive TUI selection for `dx menu`.
//!
//! Renders an inline list immediately below the prompt line.
//! stdout stays free for JSON output; the TUI is drawn to stderr.
//! crossterm is built with `use-dev-tty` so `event::read()` reads from
//! `/dev/tty` directly, working even when stdin is redirected by a shell
//! completion hook.

use std::path::Path;

use crate::menu::MenuMode;
use crate::menu::ls_colors::LsColorsConfig;
use crate::resolve::CompletionCandidates;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuResult {
    Selected {
        filter_query: String,
        changed_query: bool,
        value: std::path::PathBuf,
        terminal: crate::menu::action::TerminalState,
        geometry: Option<crate::menu::action::TerminalGeometry>,
    },
    Cancelled {
        filter_query: String,
        changed_query: bool,
        geometry: Option<crate::menu::action::TerminalGeometry>,
    },
}

/// Re-query callback: given a query string, returns fresh candidates.
pub type QueryFn<'a> = Box<dyn Fn(&str) -> CompletionCandidates + 'a>;

/// What a single menu session should show. Fixed for as long as the menu is
/// open — only the typed refinement varies, and that is held internally.
pub struct MenuRequest<'a> {
    pub candidates: CompletionCandidates,
    pub query: &'a str,
    pub mode: MenuMode,
    pub cwd: &'a Path,
    /// Prompt row supplied by shells that can report it. Measured from the
    /// terminal when absent.
    pub prompt_row: Option<u16>,
    pub query_fn: QueryFn<'a>,
}

/// Presentation settings, all sourced from `DX_MENU_*` environment variables.
#[derive(Debug, Clone, Default)]
pub struct MenuOptions {
    pub max_rows: u16,
    pub item_max_len: Option<usize>,
    pub show_border: bool,
    /// Draw to `/dev/tty` rather than stderr, as PSReadLine requires.
    pub use_tty_backend: bool,
    pub ls_colors: Option<LsColorsConfig>,
}

// The interactive TUI targets Unix TTY semantics (`/dev/tty` and explicit
// terminal scrolling). Non-Unix builds fall back to the stub below, which keeps
// the JSON/noop contract the shell hooks rely on.
#[cfg(unix)]
mod input;
#[cfg(unix)]
mod labels;
#[cfg(unix)]
mod layout;
#[cfg(unix)]
mod render;
#[cfg(unix)]
mod selection;
#[cfg(unix)]
mod session;
#[cfg(unix)]
mod status;
#[cfg(unix)]
mod terminal;
#[cfg(unix)]
mod width;

#[cfg(all(unix, test))]
mod fixtures;

#[cfg(unix)]
pub use session::select;

#[cfg(not(unix))]
mod stub;
#[cfg(not(unix))]
pub use stub::select;
