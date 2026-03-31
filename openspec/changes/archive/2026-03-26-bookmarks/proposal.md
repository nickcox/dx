## Why

dx can resolve abbreviated paths and maintain per-session undo/redo stacks, but there is no way to save a directory under a memorable name and jump to it later. Users frequently revisit the same handful of project roots, config directories, or deploy targets across different shells and terminal sessions. Named bookmarks eliminate the need to remember or retype paths, and because they're persistent and shared, they work the same whether you're in Bash, Zsh, Fish, or PowerShell.

## What Changes

- **New CLI commands**: `dx mark <name> [<path>]` saves the current (or given) directory under a name; `dx unmark <name>` removes it; `dx bookmarks` lists all saved bookmarks.
- **New persistent store**: A TOML file at `$XDG_DATA_HOME/dx/bookmarks.toml` (falling back to `~/.local/share/dx/bookmarks.toml`) holding the name-to-path mapping. TOML is human-readable and editable, matching the simplicity of the feature.
- **Lookup integration**: `dx resolve` gains awareness of bookmark names so that `dx resolve myproject` can return the bookmarked path when no filesystem match is found. Bookmark lookup sits after filesystem-based resolution in the precedence chain (direct > step-up > abbreviated > fallback roots > bookmarks > failure).
- **Output contract**: Same as other dx commands — success prints one absolute path to stdout (exit 0); failure prints diagnostic to stderr (exit non-zero). `dx bookmarks` prints one `name = path` line per entry, or `--json` for structured output.

## Capabilities

### New Capabilities

- `bookmarks`: Persistent named directory aliases — create, delete, list, and resolve bookmarks. Covers the TOML store, CLI commands, and integration with the resolve precedence chain.

### Modified Capabilities

- `path-resolution`: Bookmark lookup added as a new resolution stage between fallback roots and failure in the precedence chain.

## Impact

- **New files**: `src/bookmarks/mod.rs`, `src/bookmarks/storage.rs`, `src/cli/bookmarks.rs`, `tests/bookmarks_cli.rs`.
- **Modified files**: `src/cli/mod.rs` (register new subcommands), `src/lib.rs` (wire bookmarks module), `src/resolve/precedence.rs` (add bookmark stage).
- **New persistent state**: `$XDG_DATA_HOME/dx/bookmarks.toml` created on first `dx mark` invocation.
- **Dependencies**: No new crate dependencies expected (serde + toml are already available).
