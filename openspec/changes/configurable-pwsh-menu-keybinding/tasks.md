## 1. Init-Time PowerShell Key Configuration

- [x] 1.1 Add parser/resolver for `DX_PWSH_MENU_KEY` with default `Tab` for unset, empty, or whitespace-only values.
- [x] 1.2 Reject unsafe `DX_PWSH_MENU_KEY` values that would produce malformed PowerShell hook code, including newline and quote characters.
- [x] 1.3 Wire the resolved PowerShell menu key into `dx init pwsh --menu` generation only; keep non-menu and non-PowerShell init unaffected.

## 2. PowerShell Hook Generation

- [x] 2.1 Replace the hard-coded `Set-PSReadLineKeyHandler -Key Tab` output with the resolved generated key.
- [x] 2.2 Add generated PowerShell logic to capture the configured key's previous PSReadLine function before installing dx's handler.
- [x] 2.3 Route native fallback paths through a generated helper that invokes the captured previous PSReadLine function when safely supported, otherwise falls back to `TabCompleteNext`.
- [x] 2.4 Emit a one-time stderr warning during hook evaluation when the configured key's previous PSReadLine function is `CustomAction`.
- [x] 2.5 Preserve existing PSReadLine handler behavior, including mapped-command mode routing, `dx menu --psreadline-mode`, buffer replacement, and cancel behavior.
- [x] 2.6 Make repeated hook evaluation idempotent by not capturing dx's own handler as the previous fallback function.
- [x] 2.7 Remove or restore a prior dx menu binding when `DX_PWSH_MENU_KEY` changes in the same PowerShell session.
- [x] 2.8 Update stale hook-generation assertions that still expect the old PowerShell menu eligibility gate.

## 3. Tests

- [x] 3.1 Add tests proving default `dx init pwsh --menu` still emits `-Key Tab`.
- [x] 3.2 Add tests proving custom `DX_PWSH_MENU_KEY` values are emitted in PowerShell menu hooks.
- [x] 3.3 Add tests proving empty or whitespace-only `DX_PWSH_MENU_KEY` falls back to `Tab`.
- [x] 3.4 Add tests proving unsafe `DX_PWSH_MENU_KEY` values fail init generation.
- [x] 3.5 Add tests proving `DX_PWSH_MENU_KEY` is ignored by `dx init pwsh` without `--menu` and by POSIX shell init.
- [x] 3.6 Add generated-hook tests proving previous PSReadLine function capture is emitted and fallback uses the captured function helper.
- [x] 3.7 Add tests or script-level checks for prior `MenuComplete`, prior `TabCompleteNext`, and unsupported prior `CustomAction` fallback behavior.
- [x] 3.8 Add tests or script-level checks proving prior `CustomAction` emits a warning during hook evaluation and prior built-in functions do not warn.
- [x] 3.9 Add tests or script-level checks proving repeated hook evaluation preserves the original previous function and key changes do not leave stale dx bindings active.

## 4. Documentation and Verification

- [x] 4.1 Update user-facing configuration/setup docs with `DX_PWSH_MENU_KEY`, examples, default behavior, and re-init requirement.
- [x] 4.2 Update shell hook technical docs to explain that PowerShell menu keybinding is configurable, native fallback attempts to preserve the configured key's previous PSReadLine function, and prior `CustomAction` handlers produce a warning because they cannot be replayed.
- [x] 4.3 Run targeted hook-generation tests and `openspec validate configurable-pwsh-menu-keybinding --strict`.
