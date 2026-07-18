## Why

PowerShell modules should expose primary commands using idiomatic approved verb-noun style, with short shell conveniences implemented as aliases. The current generated `dx` module exports short function names such as `up`, `back`, `forward`, `cdf`, and `cdr`, which works interactively but looks less native in `Get-Command -Module dx` and makes alias cleanup/restoration less explicit.

## What Changes

- Rename PowerShell module primary navigation functions to idiomatic command names:
  - `up` -> `Step-Up`
  - `back` -> `Undo-Location`
  - `forward` -> `Redo-Location`
  - `cdf` -> `Set-FrecentLocation`
  - `cdr` -> `Set-RecentLocation`
- Keep user-facing short commands as aliases to the primary functions:
  - `up` and `..` -> `Step-Up`
  - `back` and `cd-` -> `Undo-Location`
  - `forward` and `cd+` -> `Redo-Location`
  - `cdf` and `z` -> `Set-FrecentLocation`
  - `cdr` -> `Set-RecentLocation`
- Preserve `cd` as an alias to `Set-DxLocation`.
- Preserve existing behavior, completions, menu mappings, stack interactions, and module unload cleanup.
- Apply this naming change only to generated PowerShell hooks; Bash, Zsh, and Fish hooks remain unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `shell-hooks`: PowerShell hooks expose idiomatic primary module function names while preserving short navigation aliases.

## Impact

- Affects generated PowerShell hook code in `src/hooks/pwsh.rs` and generated-hook tests in `src/hooks/mod.rs`.
- Requires updating PowerShell completion registration expectations if completions bind to any renamed primary command names.
- Requires module cleanup to save/restore/remove the expanded alias set, including `..`.
