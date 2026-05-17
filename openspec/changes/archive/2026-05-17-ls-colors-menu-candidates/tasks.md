## 1. Configuration and Style Resolution

- [x] 1.1 Add parsing for `DX_MENU_LS_COLORS=1` gated by a present `LS_COLORS` value
- [x] 1.2 Add an LS_COLORS-to-Ratatui style resolver for candidate paths, using a focused dependency or a documented internal subset
- [x] 1.3 Ensure disabled, missing, empty, or non-`1` configuration produces the existing monochrome candidate style

## 2. Menu Rendering Integration

- [x] 2.1 Refactor candidate presentation so each visible candidate keeps its display label paired with the corresponding path-derived style
- [x] 2.2 Apply LS_COLORS-derived styles to non-selected single-column list items
- [x] 2.3 Apply LS_COLORS-derived styles to non-selected multicolumn grid cells while preserving existing truncation and padding
- [x] 2.4 Preserve the existing selected-candidate highlight as an override in both single-column and multicolumn rendering

## 3. Behavioral Preservation

- [x] 3.1 Verify candidate styling does not alter filtering, ordering, selected candidate identity, status-row text, JSON action shape, or shell replacement text
- [x] 3.2 Add or update tests for enabled styling, disabled styling, missing `LS_COLORS`, and selected-highlight override
- [x] 3.3 Add or update tests covering multicolumn styled rendering behavior without changing grid ordering or truncation

## 4. Documentation and Verification

- [x] 4.1 Document `DX_MENU_LS_COLORS` in the menu configuration docs
- [x] 4.2 Run the relevant menu test suite
- [x] 4.3 Run the full test suite if the dependency or rendering refactor touches shared behavior
