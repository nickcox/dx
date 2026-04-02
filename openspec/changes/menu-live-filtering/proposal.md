## Why

The current menu supports selection from an initial candidate list, but users cannot continue narrowing results from the keyboard once the menu is open. This makes multi-result completion slower and forces extra round-trips back to the shell prompt.

## What Changes

- Add in-menu live filtering so printable keystrokes narrow the currently displayed menu items in real time.
- Add backspace-based filter editing and empty-state behavior when no entries match.
- Commit typed filter text back into the shell buffer when the menu exits, including cancel paths, so user refinement is not lost.
- Keep existing navigation controls (arrow keys, Tab/Shift+Tab, and vim j/k) and make them operate on the filtered list.

## Capabilities

### New Capabilities
- `dx-menu-filtering`: interactive menu filtering behavior, filter editing, selection semantics on filtered candidates, typed-query commit behavior, and no-match handling.

### Modified Capabilities
- `shell-hooks`: clarify hook interaction so hooks continue applying only final menu JSON actions, including replacement actions emitted when filter typing should be committed without a selection.

## Impact

- Affected code: `src/menu/tui.rs`, `src/cli/menu.rs` (final action generation for confirm/cancel with typed filter), and hook behavior tests.
- Affected tests/docs: `tests/menu_cli.rs` and relevant shell-hook/menu docs.
- No new external dependencies expected.
