# Shell Setup

Shell integration is required for `dx` to change the current shell directory,
record navigation history, and install completions. The `dx init` command emits
shell code; it does not modify profile files itself.

## Initialization Options

| Option | Behavior |
|---|---|
| No flag | Directory wrappers, navigation commands, and completions |
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

On Windows, enable the native PSReadLine menu with the optional
command-not-found integration:

```powershell
Invoke-Expression ((& dx init pwsh --native-menu --command-not-found | Out-String))
```

On Unix, PowerShell can instead use the Rust TUI:

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

PowerShell users on Windows should use `--native-menu` when they want a menu.
The Rust TUI installed by `--menu` is Unix-only. `--native-menu` also works on
Unix, registers structured argument completers, and uses the user's existing
PSReadLine completion key bindings. The two flags are mutually exclusive.

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

`dx` command completion is generated from its Clap command definition. It
completes subcommands, options, enum values, and ordinary filesystem paths.
The navigation wrappers retain their runtime-aware directory, history, and
frecency candidates. Completing `cd` also offers bookmarks whose name starts
with what you have typed, after the filesystem candidates. Session IDs are not
completed yet.

## Loading Alongside Other `cd` Wrappers

`dx` replaces `cd` with a shell function. So does zoxide when initialised with
`--cmd cd`, and so do some prompt and directory tools. Whichever loads **last**
wins, which makes the order in your profile significant.

**Initialise `dx` last.**

```zsh
eval "$(zoxide init zsh --cmd cd)"
eval "$(dx init zsh)"          # dx last, so its cd is the one that survives
```

Reversed, zoxide's `cd` replaces the one `dx` installed, and `dx` quietly stops
working: abbreviated paths no longer resolve, and because nothing records the
move, `back` and `forward` stay empty. Nothing reports an error — `cd pr/dx`
simply fails as an unknown directory.

### Why this is safe for zoxide

`dx` changes directory with the shell builtin, which bypasses zoxide's `cd`
function. That does not cost zoxide anything, because zoxide does not record
directories from its `cd` shim: it registers a hook that fires on any directory
change — `chpwd_functions` in Zsh, `PROMPT_COMMAND` in Bash, and a `PWD` watcher
in Fish. Those still run, so zoxide's database keeps up to date with directories
you reach through `dx`.

`dx` only ever reads from zoxide, by running `zoxide query`. It never adds,
removes, or reweights entries.

### Overlapping command names

Plain `zoxide init <shell>`, without `--cmd cd`, defines `z` and `zi`. `dx` also
defines `z`. With `dx` loaded last its version wins, which is usually what you
want: it queries the same zoxide database, but also records the jump in the
back/forward stack and can open the interactive menu. Use `zoxide init --cmd cd`
if you would rather keep the two sets of commands distinct.

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

Use `--native-menu` in that command to route mapped commands through native
PowerShell completion instead. Native PowerShell commands are registered
against a `Path` or `LiteralPath` parameter when present, otherwise their first
positional string parameter; native applications use native command completion.

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
- [Scripting](./scripting.md)
