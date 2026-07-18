## 1. Module Structure

- [x] 1.1 Wrap generated PowerShell hook output in an in-memory `dx` module imported by the profile-evaluable init script.
- [x] 1.2 Move PowerShell helper functions and saved cleanup state into module scope while preserving existing hook behavior.
- [x] 1.3 Register module `OnRemove` cleanup for dx-installed aliases and hooks where feasible.

## 2. PowerShell Navigation Aliases

- [x] 2.1 Replace the literal PowerShell `function cd` wrapper with a named dx location wrapper command.
- [x] 2.2 Install `Alias:cd` pointing at the dx location wrapper and preserve current no-arg, dash, resolve, and stack-push behavior.
- [x] 2.3 Preserve `up`, `back`, `forward`, `cdf`, `z`, `cdr`, `cd-`, and `cd+` behavior under the module-backed hook.

## 3. Optional PowerShell Hooks

- [x] 3.1 Preserve PowerShell argument completer registration for dx and navigation aliases.
- [x] 3.2 Preserve menu-enabled PSReadLine handling, custom menu key handling, and mapped command behavior.
- [x] 3.3 Preserve optional `CommandNotFoundAction` registration and feature detection.

## 4. Verification

- [x] 4.1 Add generated-hook tests for in-memory module markers, module import, `OnRemove`, and `Alias:cd` replacement.
- [x] 4.2 Add or update tests confirming no literal PowerShell `function cd` wrapper is emitted.
- [x] 4.3 Run the relevant Rust test suite and fix regressions.
