## 1. Module Scaffolding

- [x] 1.1 Create `src/hooks/mod.rs` with `Shell` enum (`Bash`, `Zsh`, `Fish`, `Pwsh`), `generate(shell, command_not_found) -> String` dispatcher, and per-shell module declarations
- [x] 1.2 Create `src/hooks/bash.rs` with `pub fn generate(command_not_found: bool) -> String` stub returning empty string
- [x] 1.3 Create `src/hooks/zsh.rs` with `pub fn generate(command_not_found: bool) -> String` stub returning empty string
- [x] 1.4 Create `src/hooks/fish.rs` with `pub fn generate(command_not_found: bool) -> String` stub returning empty string
- [x] 1.5 Create `src/hooks/pwsh.rs` with `pub fn generate(command_not_found: bool) -> String` stub returning empty string
- [x] 1.6 Add `mod hooks;` to `src/lib.rs`

## 2. CLI Subcommand

- [x] 2.1 Create `src/cli/init.rs` with `pub fn run_init(shell: &str, command_not_found: bool) -> i32` that calls `hooks::generate` and prints to stdout
- [x] 2.2 Add `Init` variant to `Commands` enum in `src/cli/mod.rs` with `shell: String` positional arg and `--command-not-found` flag
- [x] 2.3 Wire `Init` variant in `run()` match arm to call `init::run_init`
- [x] 2.4 Handle unsupported shell identifier: print diagnostic to stderr listing supported shells, exit non-zero

## 3. Bash Hook Generation

- [x] 3.1 Implement session identity export: `export DX_SESSION=$$` with guard for existing value
- [x] 3.2 Implement cd wrapper function: no-args path (`builtin cd`), dash path (`builtin cd -`), resolve-then-fallback path, flag passthrough, fire-and-forget `dx push "$PWD"`
- [x] 3.3 Implement command_not_found_handle function (conditionally included): path-like heuristic check, `DX_RESOLVE_GUARD` set/unset, resolve-then-cd-then-push on success, exit 127 on failure
- [x] 3.4 Verify generated Bash code is valid syntax (unit test that checks for balanced braces/quotes)

## 4. Zsh Hook Generation

- [x] 4.1 Implement session identity export: `export DX_SESSION=$$` with guard for existing value
- [x] 4.2 Implement cd wrapper function: same flow as Bash (Zsh supports `builtin cd` and same syntax)
- [x] 4.3 Implement command_not_found_handler function (note: Zsh uses `_handler` suffix, not `_handle`): same logic as Bash variant
- [x] 4.4 Verify generated Zsh code is valid syntax

## 5. Fish Hook Generation

- [x] 5.1 Implement session identity export: `set -gx DX_SESSION $fish_pid` with guard for existing value
- [x] 5.2 Implement cd wrapper as `function cd` wrapping `builtin cd`: no-args, dash, resolve-then-fallback, fire-and-forget `dx push`
- [x] 5.3 Implement fish_command_not_found function (conditionally included): path-like heuristic, `DX_RESOLVE_GUARD` via `set -lx`/`set -e`, resolve-then-cd-then-push
- [x] 5.4 Verify generated Fish code is valid syntax

## 6. PowerShell Hook Generation

- [x] 6.1 Implement session identity export: `$env:DX_SESSION = $PID` with guard for existing value
- [x] 6.2 Implement cd wrapper as `function cd` wrapping `Set-Location`: no-args (`Set-Location ~`), dash (track `$__dx_oldpwd`), resolve-then-fallback, fire-and-forget `dx push`
- [x] 6.3 Implement CommandNotFoundAction registration (conditionally included): feature-detect `CommandNotFoundAction` member, path-like heuristic, `$env:DX_RESOLVE_GUARD` set/remove, resolve-then-cd-then-push
- [x] 6.4 Verify generated PowerShell code is valid syntax

## 7. Unit Tests — Hook Content

- [x] 7.1 Test `generate("bash", false)` output contains cd wrapper, DX_SESSION export, but NOT command_not_found_handle
- [x] 7.2 Test `generate("bash", true)` output contains cd wrapper, DX_SESSION export, AND command_not_found_handle
- [x] 7.3 Test `generate("zsh", true)` output contains `command_not_found_handler` (not `_handle`)
- [x] 7.4 Test `generate("fish", false)` output contains `function cd` and `DX_SESSION` but NOT `fish_command_not_found`
- [x] 7.5 Test `generate("pwsh", true)` output contains `Set-Location`, `CommandNotFoundAction`, and `DX_RESOLVE_GUARD`
- [x] 7.6 Test `generate("pwsh", false)` output contains `Set-Location` but NOT `CommandNotFoundAction`
- [x] 7.7 Test all four shells' output contains `DX_SESSION` conditional guard (does not overwrite if set)

## 8. Integration Tests — CLI

- [x] 8.1 Create `tests/init_cli.rs` with test: `dx init bash` exits 0 and stdout is non-empty
- [x] 8.2 Test: `dx init zsh` exits 0 and stdout is non-empty
- [x] 8.3 Test: `dx init fish` exits 0 and stdout is non-empty
- [x] 8.4 Test: `dx init pwsh` exits 0 and stdout is non-empty
- [x] 8.5 Test: `dx init unknown` exits non-zero and stderr contains diagnostic
- [x] 8.6 Test: `dx init bash --command-not-found` output contains command_not_found handler
- [x] 8.7 Test: `dx init bash` (without flag) output does NOT contain command_not_found handler

## 9. Cleanup

- [x] 9.1 Verify full test suite passes (`cargo test`)
- [x] 9.2 Verify build succeeds (`cargo build`)
