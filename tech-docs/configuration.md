# dx Configuration

This document lists all current user-facing configuration inputs for `dx`.

## Precedence

When multiple sources set the same option, precedence is:

1. command-line flags (for commands that support them)
2. environment variables
3. config file values
4. built-in defaults

## Config File

Default location:

- `$(dirs::config_dir)/dx/config.toml`
- Examples:
  - macOS: `~/Library/Application Support/dx/config.toml`
  - Linux: `${XDG_CONFIG_HOME:-~/.config}/dx/config.toml`

Override file path with `DX_CONFIG`.

### Supported keys

```toml
search_roots = ["/Users/nick/code", "/Users/nick/work"]

[resolve]
case_sensitive = true
```

- `search_roots`: ordered list of root paths for abbreviation/fallback matching.
- `resolve.case_sensitive`: matching case behavior.

Notes:

- If no search roots are configured, `dx` implicitly includes current working directory as a root.

## Environment Variables

### Core config overrides

- `DX_CONFIG`: explicit path to config file.
- `DX_SEARCH_ROOTS`: colon-separated roots (replaces config roots when set).
- `DX_CASE_SENSITIVE`: boolean override (`1/0`, `true/false`, `yes/no`, `on/off`).

### Bookmarks

- `DX_BOOKMARKS_FILE`: explicit bookmarks TOML file path.
- `XDG_DATA_HOME`: affects default bookmarks path when `DX_BOOKMARKS_FILE` is unset.

### Session / stack behavior

- `DX_SESSION`: current session id used by stack/recents operations.
  - Usually set automatically by `dx init <shell>` output.
- `XDG_RUNTIME_DIR`: base directory for session stack files.

### Menu behavior

- `DX_MENU`: set to `0` to disable menu integration at runtime (hooks still loaded).
- `DX_MAX_MENU_RESULTS`: integer cap for menu candidate display/query pipelines.
  - Default: `1000`.
  - This is menu-specific and does not affect non-menu CLI output by default.
- `DX_MENU_ITEM_MAX_LEN`: controls multicolumn menu layout and max cell text length.
  - Default behavior (unset/empty/non-numeric): multicolumn enabled.
  - `>= 1`: multicolumn enabled with that max cell text length.
  - `<= 0`: disables multicolumn (single-column rendering).
- `DX_MENU_BORDER`: controls whether the menu list/grid block border is shown.
  - Truthy values (`1`, `true`, `yes`, `on`) enable the border.
  - Unset/empty/`0`/`false`/`no`/`off` keeps border off (default).
- `DX_MENU_DEBUG`: set to `1` to print menu debug diagnostics to stderr.

## Command-level Overrides

- `dx complete <mode> --limit <n>`: cap output rows for the current invocation.
- `dx complete <mode> --list <n>`: alias for `--limit`.

### Filesystem-prefixed query behavior

For `dx resolve` and `dx complete paths`, queries starting with `/`, `./`, `../`, `~`, or `~/` first use filesystem/direct-path semantics.

- For leading `/` queries: on direct miss, abbreviation/fallback remains anchored at filesystem root (`/`) rather than using generic search roots/cwd fallback.
- For `./`, `../`, `~`, and `~/`: on direct miss, the prefix is stripped and processing continues through the existing root-based abbreviation/fallback behavior (and bookmark lookup for `dx resolve`).

If stripping leaves an empty query (for example `~/` when the HOME target is missing), behavior remains unresolved / no candidates.

Supported modes: `paths`, `ancestors`, `frecents`, `recents`, `stack`.

## Internal Variables (normally do not set manually)

These are set by hooks/runtime internals and are not intended as stable user config:

- `DX_RESOLVE_GUARD`

Menu compatibility mode for PowerShell is enabled via a hidden
`dx menu --psreadline-mode` flag used by generated hooks.
