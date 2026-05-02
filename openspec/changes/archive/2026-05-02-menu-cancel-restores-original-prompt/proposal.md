## Why

`dx menu` currently treats Escape cancel as either `noop` or a cancel-time `replace`, and menu-enabled shell hooks interpret non-applied outcomes by falling back to native completion. In practice, that means cancelling the menu can still insert the first shell completion candidate instead of leaving the original prompt unchanged.

## What Changes

- Change explicit menu cancel semantics so Escape always restores the original prompt-derived query token rather than preserving any typed in-menu refinement.
- Distinguish explicit menu cancel from generic non-handled `noop` outcomes so shell hooks can leave the buffer unchanged without triggering native completion insertion.
- Update menu and shell-hook action contracts so only true fallback conditions (non-interactive/no-candidate/runtime failure/invalid payload) continue to invoke native shell completion.
- Remove the current "cancel commits typed refinement" behavior from the live-filtering contract.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `dx-menu`: update the command contract so explicit cancel no longer reuses the generic noop behavior when the shell must suppress native completion insertion.
- `dx-menu-filtering`: change exit behavior so cancelling always restores the original prompt token instead of preserving typed refinement.
- `shell-hooks`: change menu action handling so explicit cancel leaves the shell buffer unchanged and does not trigger native completion fallback.

## Impact

- Affected code: `src/menu/action.rs`, `src/cli/menu.rs`, `src/menu/tui.rs`, and menu-enabled shell hook generators under `src/hooks/`.
- Affected tests/docs: menu action tests, menu CLI tests, and generated hook contract tests for Bash, Zsh, Fish, and PowerShell.
- No new dependencies or external integrations are required.
