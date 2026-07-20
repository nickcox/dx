# Interactive Menu

Menu mode replaces supported completion bindings with an inline candidate
selector. It is opt-in and falls back to the shell's native completion when
`dx` cannot or should not handle the current buffer.

## Enable Menu Mode

Bash:

```bash
eval "$(dx init bash --menu)"
```

Zsh:

```zsh
eval "$(dx init zsh --menu)"
```

Fish:

```fish
dx init fish --menu | source
```

PowerShell:

```powershell
Invoke-Expression ((& dx init pwsh --menu | Out-String))
```

The interactive TUI is currently Unix-only. PowerShell on Windows loads the
handler but falls back without opening an interactive menu.

Reload generated hooks after upgrading `dx` or changing menu settings captured
at initialization time.

## Supported Navigation Commands

Menu mode recognizes these generated navigation commands:

| Commands | Candidate source |
|---|---|
| `cd` | Paths |
| `up` | Ancestors |
| `z`, `cdf` | Zoxide frecency results |
| `cdr` | Session recents |
| `back`, `cd-` | Back stack |
| `forward`, `cd+` | Forward stack |

When only one final candidate exists, `dx` may insert it without opening the
interactive menu.

## Keyboard Controls

| Key | Action |
|---|---|
| Arrow keys | Move selection |
| `Tab` / `Shift-Tab` | Move forward or backward |
| `PageDown` / `PageUp` | Move by one visible page |
| Printable characters | Refine the current query |
| `Backspace` | Remove menu-entered refinement |
| `Enter` | Accept selection |
| `Escape` / `Ctrl+C` | Cancel without changing the buffer |

Backspace cannot broaden the query beyond the text that was present when the
menu opened. The status row shows the selected resolved path and any refinement
typed during the current menu session.

## Map External Commands

`DX_MENU_COMMAND_MAPPINGS` adds menu-backed filesystem completion to other
commands. Set it before generating hooks:

```bash
export DX_MENU_COMMAND_MAPPINGS="ls=path,open=path,cat=file"
eval "$(dx init zsh --menu)"
```

Mapping syntax is `<command>=<mode>,...`.

| Mode | Candidates |
|---|---|
| `path` | Files and directories |
| `directory` | Directories only |
| `file` | Regular files only |

Mappings are captured in generated hook code. Changing the variable without
rerunning `dx init <shell> --menu` does not update active bindings. Invalid or
duplicate entries make initialization fail instead of producing partial hooks.

## PowerShell Menu Key

PowerShell binds menu mode to `Tab` by default. Set `DX_PWSH_MENU_KEY` before
generating hooks to choose another PSReadLine key:

```powershell
$env:DX_PWSH_MENU_KEY = "F12"
Invoke-Expression ((& dx init pwsh --menu | Out-String))
```

When `dx` falls back, it attempts to invoke the key's previous PSReadLine
function. If the previous binding is a custom scriptblock, initialization emits
a warning because PSReadLine does not expose that scriptblock for replay;
fallback uses `TabCompleteNext`.

## Appearance

### Rows

`DX_MENU_MAX_ROWS` controls visible candidate rows. The default is `20`.

```bash
export DX_MENU_MAX_ROWS=12
```

### Columns and item width

`DX_MENU_ITEM_MAX_LEN` controls the maximum candidate cell width. The default is
`80`. A positive integer keeps multicolumn rendering enabled; zero or a negative
value switches to a single column.

```bash
export DX_MENU_ITEM_MAX_LEN=40
```

### Border

Enable a border with:

```bash
export DX_MENU_BORDER=1
```

### File colors

Use `LS_COLORS` for unselected filesystem candidates:

```bash
export DX_MENU_LS_COLORS=1
```

This requires `LS_COLORS` to be present. The selected item retains the menu's
selection highlight.

### Candidate limit

`DX_MAX_MENU_RESULTS` caps candidate sourcing for the menu. The default is
`1000`.

## Disable Menu Mode Temporarily

Set `DX_MENU=0` to keep loaded hooks but bypass menu handling:

```bash
export DX_MENU=0
```

PowerShell:

```powershell
$env:DX_MENU = "0"
```

Unset the variable or change it to another value to re-enable menu handling.

## Terminal Requirements

The interactive menu requires a TTY for keyboard input and rendering. POSIX
shells also query the terminal cursor position before reserving menu space. If
TTY access or cursor reporting is unavailable, `dx` returns control to native
completion rather than guessing a screen position.

Menu rendering keeps stdout reserved for the final shell action. This allows
hooks to capture the result while interaction continues through the TTY.

The reproducible VHS source for the documentation animation is in
[demo/README.md](./demo/README.md).

## Diagnostics

Set `DX_MENU_DEBUG=1` to print menu diagnostics to stderr:

```bash
export DX_MENU_DEBUG=1
```

Remove it after troubleshooting because diagnostics can be noisy during
completion.

## Related Guides

- [Shell Setup](./shell-setup.md)
- [Navigation Guide](./navigation.md)
- [Configuration Reference](./configuration.md)
- [Troubleshooting](./troubleshooting.md)
