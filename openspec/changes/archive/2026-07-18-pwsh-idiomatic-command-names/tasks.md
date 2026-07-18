## 1. PowerShell Primary Functions

- [x] 1.1 Rename generated PowerShell primary functions to `Step-Up`, `Undo-Location`, `Redo-Location`, `Set-FrecentLocation`, and `Set-RecentLocation`.
- [x] 1.2 Keep `Set-DxLocation` as the primary function for `cd` integration.
- [x] 1.3 Export the idiomatic primary functions from the generated `dx` module.

## 2. Alias Compatibility

- [x] 2.1 Install aliases `up` and `..` pointing to `Step-Up`.
- [x] 2.2 Install aliases `back` and `cd-` pointing to `Undo-Location`.
- [x] 2.3 Install aliases `forward` and `cd+` pointing to `Redo-Location`.
- [x] 2.4 Install aliases `cdf` and `z` pointing to `Set-FrecentLocation`, and `cdr` pointing to `Set-RecentLocation`.

## 3. Cleanup And Completions

- [x] 3.1 Expand module alias state capture and `OnRemove` cleanup to cover `up`, `..`, `back`, `forward`, `cdf`, and `cdr` in addition to existing aliases.
- [x] 3.2 Preserve existing completion registrations for short aliases and stack/menu behavior.
- [x] 3.3 Preserve menu-enabled PSReadLine handling and command-not-found behavior under the renamed functions.

## 4. Verification

- [x] 4.1 Update generated-hook tests for primary PowerShell function names and alias targets.
- [x] 4.2 Add or update tests for `..` alias generation and cleanup coverage.
- [x] 4.3 Evaluate generated PowerShell init output if feasible and run the relevant Rust test suite.
