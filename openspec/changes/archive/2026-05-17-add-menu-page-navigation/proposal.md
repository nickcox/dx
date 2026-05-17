## Why

`dx menu` supports line-by-line navigation with arrows and Tab, but long candidate lists require many key presses to traverse. PageUp and PageDown are standard terminal navigation keys and should move selection by one visible page in both list and grid layouts.

## What Changes

- Add PageDown handling to move the selected candidate forward by one visible page.
- Add PageUp handling to move the selected candidate backward by one visible page.
- In single-column layout, a page equals the visible candidate row count.
- In multicolumn layout, a page equals the visible row count multiplied by the active column count.
- Clamp page movement at the first and last candidate rather than wrapping.
- Preserve existing arrow, Tab, filtering, selection, rendering, and JSON action behavior.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `dx-menu`: Add PageUp/PageDown interactive navigation behavior.
- `dx-menu-multicolumn`: Clarify page navigation behavior when the menu is rendered as a grid.

## Impact

- Affected code: `src/menu/tui.rs` key mapping, selection movement helpers, and tests.
- No shell hook changes, config changes, or JSON action protocol changes.
