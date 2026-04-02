## Why

Current menu filtering preserves typed refinement only when the menu exits, so users cannot see their shell buffer update in real time while typing. This creates a mismatch between visible menu interaction and the command line state users expect to be actively editing.

## What Changes

- Add live shell-buffer synchronization so typed filter keystrokes update the active shell command line immediately while the menu is open.
- Preserve current selection behavior: moving highlighted menu entries must not rewrite the shell buffer unless the selection is accepted.
- Introduce a menu interaction protocol that supports incremental edit actions during an interactive session and a final completion/cancel event.
- Keep existing per-shell fallback behavior for non-interactive contexts.

## Capabilities

### New Capabilities
- `dx-menu-live-buffer-sync`: real-time menu typing synchronization semantics, incremental edit action model, and confirm/cancel terminal outcomes.

### Modified Capabilities
- `shell-hooks`: update hook contracts to support incremental menu actions while preserving shell-specific safety and fallback behavior.

## Impact

- Affected code: `src/menu/tui.rs`, `src/cli/menu.rs`, and shell hook templates (`src/hooks/*.rs`) for session/action handling.
- Affected tests/docs: menu integration tests and shell-hook/menu interaction documentation.
- Potential protocol changes between `dx menu` and hook-side consumers (requires careful backward compatibility strategy).
