## Why

The current single-column menu becomes hard to scan when many candidates are present, especially on wide terminals where horizontal space is unused. An optional multicolumn layout can increase information density and reduce scrolling without changing existing default behavior.

## What Changes

- Add optional multicolumn display mode for `dx menu` candidate lists.
- Use `DX_MENU_ITEM_MAX_LEN` as the activation and sizing control for multicolumn rendering.
- Keep single-column layout as the default when `DX_MENU_ITEM_MAX_LEN` is unset, empty, non-numeric, or less than `1`.
- Compute column count dynamically from terminal width and configured item max length plus padding.
- Keep final replacement JSON action semantics unchanged so shell hooks do not require protocol changes.

## Capabilities

### New Capabilities
- `dx-menu-multicolumn`: Optional multicolumn rendering, dynamic column calculation, truncation controls, and navigation semantics for menu candidate presentation.

### Modified Capabilities
- None.

## Impact

- Affected code: `src/menu/tui.rs` (layout + rendering), `src/cli/menu.rs` (configuration wiring), and menu-related tests.
- Affected tests/docs: `tests/menu_cli.rs` and docs covering menu behavior and `DX_MENU_ITEM_MAX_LEN` usage.
- Dependencies: no new dependencies expected; implementation stays within existing ratatui/crossterm stack.
- Workflow note: this change is independent of `menu-live-buffer-sync`; if that change is applied later, multicolumn layout should continue to operate on the currently presented candidate set.
