# Configuration Reference

`dx` reads settings from a TOML file and from environment variables. Every
setting available in the file can also be set in the environment, and the
environment wins.

## Precedence

When multiple sources configure the same behavior, precedence is:

1. Command-line flags, where supported
2. Environment variables
3. Config file values
4. Built-in defaults

## Value Formats

Boolean settings accept `1/0`, `true/false`, `yes/no`, and `on/off`, in any
case and with surrounding whitespace ignored.

A value that cannot be understood — an unrecognised boolean, or a number that is
not a number — is skipped rather than treated as "off" or reset to the default.
The next source in the precedence list applies instead, so a typo in an
environment variable leaves the config file's value in place.

Numbers that are out of range or nonsensical for a setting, such as a
`max_rows` of `0`, fall back to the built-in default from either source.

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

[menu]
item_max_len = 80
border = false
max_rows = 20
max_results = 1000
ls_colors = false
command_mappings = "ls=path,open=path,cat=file"
pwsh_key = "Tab"
```

`search_roots` is an ordered list used for abbreviation and fallback matching.
The current working directory is appended unless it duplicates a configured
root. Matches collected across all effective roots must still be unambiguous.

`resolve.case_sensitive` defaults to `true`.

Each `[menu]` key has an equivalent environment variable, documented under
[Menu Runtime Settings](#menu-runtime-settings) and
[Hook Generation Settings](#hook-generation-settings).

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

This one is read by the generated shell hook rather than by `dx`, so it has no
config-file equivalent.

### `DX_MAX_MENU_RESULTS`

Config key: `menu.max_results`.

Maximum candidates sourced for a Rust TUI or native PowerShell menu invocation.
Default: `1000`. Native PowerShell completion reads this value when completion
runs and does not use PSReadLine's `CompletionQueryItems` as a candidate limit.

### `DX_MENU_MAX_ROWS`

Config key: `menu.max_rows`.

Maximum visible candidate rows. Default: `20`. Invalid and non-positive values
use the default.

### `DX_MENU_ITEM_MAX_LEN`

Config key: `menu.item_max_len`.

Maximum candidate cell width or native PowerShell list-item length. Default:
`80`. Native truncation keeps the end of the label and prefixes it with `…`.

The Rust TUI counts terminal cells, so a double-width character such as a CJK
ideograph uses two of the budget. Native PowerShell counts text elements
instead, which means a wide label can render up to twice this many cells.

- Positive values enable multicolumn layout with that width.
- Zero or negative values disable multicolumn layout and native label truncation.
- Empty or invalid values use the default.

### `DX_MENU_BORDER`

Config key: `menu.border`. Enables the menu border. The default is off.

### `DX_MENU_LS_COLORS`

Config key: `menu.ls_colors`. Enable it to style unselected filesystem
candidates using `LS_COLORS`; both this setting and a non-empty `LS_COLORS` are
required. The selected item always uses the menu highlight.

### `DX_MENU_DEBUG`

Enable to print menu diagnostics to stderr. Diagnostic output only, so there is
no config-file equivalent.

## Hook Generation Settings

These values are captured when `dx init` generates hooks. Rerun initialization
and reload the shell profile after changing them.

Because `dx init` output is evaluated by shell profiles, a problem in the config
file never stops a usable hook being emitted: `dx init` reports the problem on
stderr and falls back to defaults. An invalid *environment* value still fails,
so a setting you have just changed does not silently do nothing. Every other
subcommand rejects an unreadable config file outright.

### `DX_MENU_COMMAND_MAPPINGS`

Config key: `menu.command_mappings`.

Comma-separated external command mappings:

```text
ls=path,open=path,cat=file
```

Valid modes are:

- `path`: files and directories
- `directory`: directories only
- `file`: regular files only

Mappings apply when hooks are generated with `--menu`, or with PowerShell's
`--native-menu`. Invalid or duplicate entries fail initialization when they come
from the environment, and are reported and ignored when they come from the config
file.

### `DX_PWSH_MENU_KEY`

Config key: `menu.pwsh_key`.

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
