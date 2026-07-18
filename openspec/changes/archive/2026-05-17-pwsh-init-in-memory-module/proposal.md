## Why

The current `dx init pwsh` output evaluates a large loose script directly into the user's session, which makes PowerShell integration harder to inspect, reload, and unload cleanly. PowerShell modules provide a native lifecycle boundary for import-time side effects such as alias replacement, argument completers, PSReadLine bindings, and command-not-found hooks while preserving the current single-binary distribution model.

## What Changes

- Change generated PowerShell init output to import an in-memory module named `dx` instead of defining loose functions and aliases directly in the caller scope.
- Move the generated PowerShell hook body into a module script block with module-scoped helper functions and state.
- Install user-facing PowerShell commands through real command functions plus aliases, including replacing `cd` by aliasing it to the dx location wrapper rather than defining a function literally named `cd`.
- Add module unload cleanup via `OnRemove` so `Remove-Module dx` restores the prior `cd` alias and removes or restores other dx-installed aliases/hooks where feasible.
- Preserve existing environment-driven configuration for menu mode, command-not-found mode, menu command mappings, and PowerShell menu key behavior.
- Keep the generated module in memory for this change; do not introduce a cached `.psm1`, packaged PowerShell module, or PowerShell Gallery distribution.
- Do not rewrite the location wrapper into a full native `Set-Location`-compatible steppable-pipeline wrapper in this change; reserve deeper `Set-LocationEx` parity for a follow-up.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `shell-hooks`: PowerShell init uses module lifecycle semantics and aliases `cd` to a dx location wrapper while preserving existing navigation behavior.

## Impact

- Affects generated PowerShell hook code in `src/hooks/pwsh.rs` and hook-generation tests in `src/hooks/mod.rs`.
- May require additional tests for generated module structure, `Import-Module`/`Remove-Module` markers, `OnRemove` cleanup, and `cd` alias replacement.
- The public `dx init pwsh` command remains the entry point and continues printing a script for profile evaluation.
- Bash, Zsh, and Fish hook generation are not affected.
