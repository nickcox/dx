# Configuration Reference

`dx` reads core resolution settings from a TOML file and environment variables.
Menu and shell-generation settings are environment-only.

## Precedence

When multiple sources configure the same behavior, precedence is:

1. Command-line flags, where supported
2. Environment variables
3. Config file values
4. Built-in defaults

## Config File

The default file is `dx/config.toml` beneath the platform configuration
directory. Common locations are:

- macOS: `~/Library/Application Support/dx/config.toml`
- Linux: `${XDG_CONFIG_HOME:-~/.config}/dx/config.toml`
- Windows: the platform application configuration directory under `dx`

Set `DX_CONFIG` to use an explicit file:

```bash
export DX_CONFIG="$HOME/.config/dx-work.toml"
```

An explicitly configured file must exist and contain valid TOML.

### Supported keys

```toml
search_roots = ["/home/me/code", "/home/me/work"]

[resolve]
case_sensitive = true
```

`search_roots` is an ordered list used for abbreviation and fallback matching.
The current working directory is appended unless it duplicates a configured
root. Matches collected across all effective roots must still be unambiguous.

`resolve.case_sensitive` defaults to `true`.

## Core Environment Variables

### `DX_CONFIG`

Explicit config file path. An empty value is treated as unset.

### `DX_SEARCH_ROOTS`

Replaces config-file search roots. The value is a platform path list, using `:`
on Unix-like systems and `;` on Windows.

```bash
export DX_SEARCH_ROOTS="$HOME/code:$HOME/work"
```

### `DX_CASE_SENSITIVE`

Overrides matching case behavior. Accepted values are `1/0`, `true/false`,
`yes/no`, and `on/off`.

```bash
export DX_CASE_SENSITIVE=false
```

## Bookmark Storage

### `DX_BOOKMARKS_FILE`

Explicit bookmarks TOML path.

### `XDG_DATA_HOME`

When `DX_BOOKMARKS_FILE` is unset, this changes the default bookmark path to
`$XDG_DATA_HOME/dx/bookmarks.toml`. Otherwise `dx` uses the platform data
directory.

## Session Storage

### `DX_SESSION`

Session identifier used by back/forward stacks and recent directories. Generated
shell hooks set it to the shell process ID when it is not already present.

### `XDG_RUNTIME_DIR`

Base directory for session stack files on platforms that provide it. If unset,
`dx` uses the system temporary directory.

Most users should let generated hooks manage session identity.

## Menu Runtime Settings

### `DX_MENU`

Set to `0` to bypass interactive menu handling while keeping hooks loaded.

### `DX_MAX_MENU_RESULTS`

Maximum candidates sourced for a Rust TUI or native PowerShell menu invocation.
Default: `1000`. Native PowerShell completion reads this value when completion
runs and does not use PSReadLine's `CompletionQueryItems` as a candidate limit.

### `DX_MENU_MAX_ROWS`

Maximum visible candidate rows. Default: `20`. Invalid and non-positive values
use the default.

### `DX_MENU_ITEM_MAX_LEN`

Maximum candidate cell width or native PowerShell list-item length. Default:
`80`. Native truncation keeps the end of the label and prefixes it with `…`.

- Positive values enable multicolumn layout with that width.
- Zero or negative values disable multicolumn layout and native label truncation.
- Empty or invalid values use the default.

### `DX_MENU_BORDER`

Enables the menu border with `1`, `true`, `yes`, or `on`. The default is off.

### `DX_MENU_LS_COLORS`

Set to `1` to style unselected filesystem candidates using `LS_COLORS`. Both
variables must be present. The selected item always uses the menu highlight.

### `DX_MENU_DEBUG`

Set to `1` to print menu diagnostics to stderr.

## Hook Generation Settings

These values are captured when `dx init` generates hooks. Rerun initialization
and reload the shell profile after changing them.

### `DX_MENU_COMMAND_MAPPINGS`

Comma-separated external command mappings:

```text
ls=path,open=path,cat=file
```

Valid modes are:

- `path`: files and directories
- `directory`: directories only
- `file`: regular files only

Mappings apply when hooks are generated with `--menu`, or with PowerShell's
`--native-menu`. Invalid or duplicate entries make initialization fail.

### `DX_PWSH_MENU_KEY`

PowerShell-only PSReadLine key for the Rust TUI mode. Default: `Tab`. It does not
affect `--native-menu`, which preserves the existing PSReadLine key map.

```powershell
$env:DX_PWSH_MENU_KEY = "F12"
Invoke-Expression ((& dx init pwsh --menu | Out-String))
```

## Command-Level Limits

Completion commands accept a per-invocation limit:

```bash
dx complete paths --limit 20 project
```

`--list` is an alias for `--limit` on completion commands.

## Internal Variables

Variables such as `DX_RESOLVE_GUARD` are managed by generated hooks and are not
stable user configuration.

## Related Guides

- [Navigation Guide](./navigation.md)
- [Interactive Menu](./menu.md)
- [Troubleshooting](./troubleshooting.md)
