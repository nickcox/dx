# Troubleshooting

## `dx` Runs but Navigation Commands Are Missing

If `dx --help` works but `up`, `back`, or the wrapped `cd` behavior is missing,
the shell integration has not been loaded.

Regenerate and evaluate it in the current shell:

```bash
eval "$(dx init bash)"  # Bash
eval "$(dx init zsh)"   # Zsh
```

```fish
dx init fish | source
```

```powershell
Invoke-Expression ((& dx init pwsh | Out-String))
```

Then make sure the same command is present in the shell profile. Regenerate
hooks after upgrading `dx`.

## PowerShell Hook Loading Fails

PowerShell output contains multiline module definitions. Evaluate it as one
script block:

```powershell
Invoke-Expression ((& dx init pwsh | Out-String))
```

Do not use a direct line-by-line pipeline into `Invoke-Expression`.

To reset the loaded integration before testing again:

```powershell
Remove-Module dx -ErrorAction SilentlyContinue
Invoke-Expression ((& dx init pwsh | Out-String))
```

## Menu Does Not Open

Check these conditions:

1. Hooks were generated with `--menu`.
2. `DX_MENU` is not set to `0`.
3. The command at the cursor is supported or mapped.
4. The shell has an interactive TTY.
5. The terminal responds to cursor-position queries.

When interactive startup is unavailable, `dx` intentionally returns to native
completion rather than drawing at a guessed screen position.

Enable diagnostics temporarily:

```bash
export DX_MENU_DEBUG=1
```

PowerShell:

```powershell
$env:DX_MENU_DEBUG = "1"
```

## Menu Settings Do Not Change

Runtime appearance settings are read when the menu opens, but command mappings
and the PowerShell menu key are embedded in generated hook code.

After changing `DX_MENU_COMMAND_MAPPINGS`, rerun `dx init <shell> --menu` or
`dx init pwsh --native-menu` and reload the profile. After changing
`DX_PWSH_MENU_KEY`, rerun `dx init pwsh --menu`; native mode does not use that
setting.

## PowerShell Native Menu Does Not Open

`--native-menu` supplies structured candidates but does not replace the user's
PSReadLine key bindings. Check the current completion bindings:

```powershell
Get-PSReadLineKeyHandler -Bound | Where-Object Function -eq MenuComplete
```

To bind Tab to the native menu:

```powershell
Set-PSReadLineKeyHandler -Key Tab -Function MenuComplete
```

## PowerShell Warns About `CustomAction`

The configured menu key was already bound to a custom PSReadLine scriptblock.
PSReadLine does not expose that scriptblock for replay, so `dx` cannot preserve
it as native fallback. Choose a different `DX_PWSH_MENU_KEY` or accept
`TabCompleteNext` as fallback.

The warning is emitted when hooks load, not on every keypress.

## Frecent Commands Return Nothing

`z` and `cdf` depend on zoxide. Confirm it is installed and has recorded
directories:

```bash
zoxide --version
zoxide query --list
```

`dx` deliberately returns no frecent candidates when zoxide is unavailable or
its query fails.

## Back or Forward History Is Empty

History is scoped by `DX_SESSION`. Confirm generated hooks are loaded and the
variable is set:

```bash
printf '%s\n' "$DX_SESSION"
dx stack --list
```

PowerShell:

```powershell
$env:DX_SESSION
dx stack --list
```

Each shell normally has its own history. Starting a new shell creates a new
session unless `DX_SESSION` is inherited or set explicitly.

## Abbreviated Path Is Ambiguous

`dx` refuses to guess when multiple directories match. Inspect candidates:

```bash
dx resolve <query> --list
dx complete paths <query>
```

Both print the candidates to stdout. `dx resolve --list` still exits non-zero,
because the query did not resolve to one directory — see
[Scripting](./scripting.md#exit-codes).

Use a longer abbreviation, a direct path, or narrower search roots.

## Protected Directories Break External Programs

Some operating-system protected directories allow a shell builtin to change
location while refusing to start child processes with that directory as their
working directory. This can affect `dx`, prompts such as Starship, and unrelated
external programs.

`dx` runs stack traversal from a safe working directory where possible, but it
cannot make another program executable in a directory rejected by the operating
system. Leave the protected directory with a shell builtin or correct its
permissions/access policy.

## Config File Errors

If `DX_CONFIG` is set, the referenced file must exist and parse successfully.
Temporarily remove the override to test defaults:

```bash
unset DX_CONFIG
```

PowerShell:

```powershell
Remove-Item Env:DX_CONFIG -ErrorAction SilentlyContinue
```

See [Configuration Reference](./configuration.md) for valid keys and paths.

## Collecting Useful Diagnostics

When reporting an issue, include:

- `dx --version`
- operating system and terminal emulator
- shell and shell version
- whether `--menu` or `--command-not-found` is enabled
- relevant `DX_*` values with sensitive paths removed
- the smallest command buffer that reproduces the problem
- `DX_MENU_DEBUG=1` output for menu problems

## Related Guides

- [Shell Setup](./shell-setup.md)
- [Navigation Guide](./navigation.md)
- [Interactive Menu](./menu.md)
- [Configuration Reference](./configuration.md)
- [Scripting](./scripting.md)
