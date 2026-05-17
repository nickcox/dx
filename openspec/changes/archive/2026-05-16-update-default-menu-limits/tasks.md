## 1. Update Default Values

- [x] 1.1 Change `parse_menu_item_max_len()` default from `usize::MAX` to `80` in `src/cli/menu.rs`
- [x] 1.2 Change `parse_menu_max_rows()` default fallback from `10` to `20` in `src/cli/menu.rs`

## 2. Update Specs

- [x] 2.1 Update `openspec/specs/dx-menu-multicolumn/spec.md` requirement on default behavior when `DX_MENU_ITEM_MAX_LEN` is unset

## 3. Update Documentation

- [x] 3.1 Update documented defaults in `tech-docs/configuration.md`

## 4. Update Tests

- [x] 4.1 Update test assertions in `src/cli/menu.rs` that assert default `DX_MENU_MAX_ROWS=10`
- [x] 4.2 Update test assertions that depend on no-cap default behavior of `DX_MENU_ITEM_MAX_LEN`
