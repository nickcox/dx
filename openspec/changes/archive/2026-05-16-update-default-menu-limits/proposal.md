## Why

The current defaults for menu display limits are too conservative. `DX_MENU_ITEM_MAX_LEN` defaults to no truncation cap (`usize::MAX`), which can produce overly wide columns. `DX_MENU_MAX_ROWS` defaults to 10, limiting visible candidates unnecessarily on larger terminals. These defaults should be more practical out of the box.

## What Changes

- Change default for `DX_MENU_ITEM_MAX_LEN` from `usize::MAX` (no cap) to `80`
- Change default for `DX_MENU_MAX_ROWS` from `10` to `20`

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `dx-menu-multicolumn`: Change default behavior of `DX_MENU_ITEM_MAX_LEN` from no cap to 80
- (No existing spec for `DX_MENU_MAX_ROWS` — it is an implementation detail in `menu.rs`)

## Impact

- `src/cli/menu.rs` — two default value changes in `parse_menu_item_max_len()` and `parse_menu_max_rows()`
- `openspec/specs/dx-menu-multicolumn/spec.md` — update scenarios referencing default behavior
- `tech-docs/configuration.md` — update documented defaults
- Test assertions that rely on current defaults
