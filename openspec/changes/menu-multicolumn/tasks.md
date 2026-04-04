## 1. Configuration and Mode Wiring

- [x] 1.1 Parse `DX_MENU_ITEM_MAX_LEN` in menu runtime and validate integer semantics
- [x] 1.2 Keep single-column as default when value is missing/empty/non-numeric/<1
- [x] 1.3 Thread parsed item max length into TUI render path without changing action output contracts

## 2. Multicolumn Layout Engine

- [x] 2.1 Implement geometry calculation using `cell_width = item_max_len + padding`
- [x] 2.2 Implement dynamic `columns = max(1, floor(terminal_width / cell_width))`
- [x] 2.3 Implement row-major candidate-to-cell mapping that preserves source ordering
- [x] 2.4 Implement cell truncation to configured max length and keep status line full path display

## 3. Navigation Behavior in Grid Mode

- [x] 3.1 Implement deterministic left/right movement rules across grid cells
- [x] 3.2 Implement deterministic up/down movement rules across grid rows
- [x] 3.3 Preserve existing Enter/Esc/Ctrl+C semantics in multicolumn mode
- [x] 3.4 Ensure filter updates reset/retain selection consistently under grid reflow

## 4. Verification

- [x] 4.1 Add unit/integration tests for activation/default fallback behavior
- [x] 4.2 Add tests for dynamic column calculation across terminal widths
- [x] 4.3 Add tests for row-major ordering, navigation behavior, and truncation semantics
- [x] 4.4 Add tests to verify JSON action protocol compatibility (`replace`/`noop`) in multicolumn mode
- [x] 4.5 Update menu documentation with `DX_MENU_ITEM_MAX_LEN` usage and behavior notes
- [x] 4.6 Run full test suite and strict OpenSpec validation for this change
