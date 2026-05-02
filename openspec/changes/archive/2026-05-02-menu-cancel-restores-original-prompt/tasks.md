## 1. Menu Action Protocol

- [x] 1.1 Add an explicit final `cancel` action to the menu JSON protocol and update serialization tests.
- [x] 1.2 Update `dx menu` action mapping so explicit interactive cancel returns `cancel`, while non-interactive and runtime fallback paths still return `noop`.
- [x] 1.3 Remove cancel-time typed-refinement replacement behavior so cancel always restores the original prompt state.

## 2. Shell Hook Handling

- [x] 2.1 Update Bash menu hook handling so explicit `cancel` is treated as a handled no-op and does not fall back to native completion.
- [x] 2.2 Update Zsh and Fish menu hook handling so explicit `cancel` leaves the buffer unchanged without invoking `expand-or-complete` / `commandline -f complete`.
- [x] 2.3 Update PowerShell PSReadLine menu handling so explicit `cancel` returns without calling `TabCompleteNext`.

## 3. Verification

- [x] 3.1 Add or update Rust unit tests for menu action mapping and cancel semantics.
- [x] 3.2 Add or update generated-hook contract tests covering explicit `cancel` handling for Bash, Zsh, Fish, and PowerShell.
- [x] 3.3 Run focused menu tests and full `cargo test` to verify cancel no longer inserts the first completion item.
