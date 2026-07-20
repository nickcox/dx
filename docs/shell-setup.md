# Shell Setup

Shell integration is required for `dx` to change the current shell directory,
record navigation history, and install completions. The `dx init` command emits
shell code; it does not modify profile files itself.

## Initialization Options

| Option | Behavior |
|---|---|
| No flag | Directory wrappers, navigation commands, and normal completions |
| `--menu` | Adds the interactive completion menu |
| `--command-not-found` | Lets path-like unknown commands resolve as directories |

The flags can be combined:

```text
dx init <shell> --menu --command-not-found
```

Generated hooks capture settings such as menu command mappings and the
PowerShell menu key. Re-run `dx init`, then reload the profile, after changing
those settings or upgrading `dx`.

## Bash

Add to `~/.bashrc`:

```bash
eval "$(dx init bash)"
```

With all optional integrations:

```bash
eval "$(dx init bash --menu --command-not-found)"
```

Reload the profile:

```bash
source ~/.bashrc
```

## Zsh

Add to `~/.zshrc`:

```zsh
eval "$(dx init zsh)"
```

With all optional integrations:

```zsh
eval "$(dx init zsh --menu --command-not-found)"
```

Reload the profile:

```zsh
source ~/.zshrc
```

## Fish

Add to `~/.config/fish/config.fish`:

```fish
dx init fish | source
```

With all optional integrations:

```fish
dx init fish --menu --command-not-found | source
```

Reload the profile:

```fish
source ~/.config/fish/config.fish
```

## PowerShell

Find the active profile path:

```powershell
$PROFILE
```

Create the file if needed:

```powershell
New-Item -ItemType File -Path $PROFILE -Force
```

Add this initialization command:

```powershell
Invoke-Expression ((& dx init pwsh | Out-String))
```

With all optional integrations:

```powershell
Invoke-Expression ((& dx init pwsh --menu --command-not-found | Out-String))
```

Reload the profile:

```powershell
. $PROFILE
```

PowerShell must evaluate the generated output as a single script block. Do not
pipe `dx init pwsh` directly to `Invoke-Expression`; line-by-line evaluation can
break multiline constructs in the generated module.

The interactive TUI is currently Unix-only. On Windows, `--menu` installs the
PowerShell handler but interactive selection falls back without opening a menu.

The integration loads an in-memory module named `dx`. `Remove-Module dx`
removes it and restores replaced aliases where possible.

## What Gets Installed

The generated hooks provide these interactive commands:

| Command | Purpose |
|---|---|
| `cd` | Native directory change with `dx` path resolution |
| `up` | Move to an ancestor (`..` is also installed in PowerShell) |
| `back` / `cd-` | Undo directory navigation |
| `forward` / `cd+` | Redo directory navigation |
| `z` / `cdf` | Jump using zoxide frecency results |
| `cdr` | Jump to a directory recently visited in this shell session |

The hooks also set `DX_SESSION` when it is not already present. This session ID
keeps each shell's back, forward, and recent-directory state separate.

## Command-Not-Found Integration

`--command-not-found` enables directory resolution for unknown commands that
look path-like. It is deliberately conservative and ignores ordinary misspelled
commands.

Examples that can trigger resolution:

```text
pr/dx
...
cd-e
P..Shell
```

On success, the shell changes to the resolved directory. On failure, it emits
the shell's normal command-not-found result. The generated integration replaces
rather than chains an existing custom command-not-found handler, so review your
profile before enabling it. PowerShell installs this option only when the host
exposes `CommandNotFoundAction`.

## Menu-Backed External Commands

Menu mode handles the built-in navigation commands automatically. You can also
map external commands such as `ls`, `open`, or `cat` by setting
`DX_MENU_COMMAND_MAPPINGS` before running `dx init`.

Bash or Zsh:

```bash
export DX_MENU_COMMAND_MAPPINGS="ls=path,open=path,cat=file"
eval "$(dx init zsh --menu)"
```

Fish:

```fish
set -gx DX_MENU_COMMAND_MAPPINGS "ls=path,open=path,cat=file"
dx init fish --menu | source
```

PowerShell:

```powershell
$env:DX_MENU_COMMAND_MAPPINGS = "ls=path,open=path,cat=file"
Invoke-Expression ((& dx init pwsh --menu | Out-String))
```

Valid mapping modes are `path`, `directory`, and `file`. See
[Interactive Menu](./menu.md) for details.

## Verify Setup

After reloading the profile:

1. Run `dx --help` and confirm it prints help.
2. Run `Get-Command dx` in PowerShell or `command -v dx` in POSIX shells.
3. Change between two directories and run `back`, then `forward`.
4. Type a partial path and use the shell's completion key.

If the executable works but the navigation commands are missing, the generated
hook has not been loaded. See [Troubleshooting](./troubleshooting.md).

## Related Guides

- [Quickstart](./quickstart.md)
- [Navigation Guide](./navigation.md)
- [Interactive Menu](./menu.md)
- [Configuration Reference](./configuration.md)
