## Why

PowerShell menu mode currently installs a global PSReadLine Tab handler. The handler falls back to `TabCompleteNext` for non-dx contexts, but it still replaces any existing Tab key binding and can interfere with users or modules that customize Tab behavior. Making the menu key configurable and preserving the previous key function on fallback lets users keep dx's interactive menu without losing their existing completion behavior.

## What Changes

- Add an init-time PowerShell menu key configuration consumed by `dx init pwsh --menu`.
- Keep `Tab` as the default key when no configuration is provided.
- Emit `Set-PSReadLineKeyHandler -Key <configured-key>` in generated PowerShell menu hooks.
- Capture the configured key's previous PSReadLine function before rebinding it and invoke that function for noop/error/non-replace fallback paths when possible.
- Emit a stderr warning when the previous binding is a `CustomAction`, because dx cannot replay arbitrary user-defined scriptblock handlers.
- Fall back to `TabCompleteNext` only when the prior function cannot be safely replayed.
- Document how to configure the key and that changing it requires regenerating and reloading `dx init pwsh --menu` output.
- Keep Bash, Zsh, and Fish behavior unchanged.

## Capabilities

### New Capabilities

### Modified Capabilities
- `shell-hooks`: PowerShell menu hook generation gains configurable PSReadLine key binding while retaining `Tab` as the default and preserving the prior key function for fallback behavior when possible.

## Impact

- Affected code: PowerShell hook generation in `src/hooks/pwsh.rs`, init-time configuration parsing in or near `src/cli/init.rs` / `src/hooks`, and shell hook tests in `tests/menu_cli.rs` and `src/hooks/mod.rs`.
- Affected docs: user-facing setup/configuration docs and shell hook technical docs.
- Compatibility: default behavior remains `Tab`; users only see changed behavior if they set the new PowerShell menu key configuration before running `dx init pwsh --menu`.
