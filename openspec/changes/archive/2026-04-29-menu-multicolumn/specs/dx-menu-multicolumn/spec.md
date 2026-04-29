## ADDED Requirements

### Requirement: Multicolumn Activation via Item Max Length
The system SHALL use `DX_MENU_ITEM_MAX_LEN` to control multicolumn rendering and optional cell text length limits.

- If `DX_MENU_ITEM_MAX_LEN` is unset, empty, or non-numeric, the menu SHALL keep multicolumn calculations enabled using default behavior with no artificial configuration cap.
- If `DX_MENU_ITEM_MAX_LEN` is a valid integer greater than or equal to `1`, the menu SHALL enable multicolumn calculations for that render cycle and use the value as an upper bound for cell text length.
- If `DX_MENU_ITEM_MAX_LEN` is `0` or negative, the menu SHALL render using single-column layout.

#### Scenario: Missing or invalid value keeps default multicolumn behavior
- **WHEN** `DX_MENU_ITEM_MAX_LEN` is unset, empty, or non-numeric
- **THEN** the menu SHALL keep multicolumn calculations enabled without a configured cell text cap

#### Scenario: Non-positive value disables multicolumn
- **WHEN** `DX_MENU_ITEM_MAX_LEN=0`
- **THEN** the menu SHALL render using the existing single-column layout

#### Scenario: Positive value enables multicolumn calculations
- **WHEN** `DX_MENU_ITEM_MAX_LEN=24`
- **THEN** the menu SHALL calculate columns using an effective max item length no greater than `24`, plus padding

### Requirement: Dynamic Column Count Calculation
In multicolumn mode, the system SHALL calculate possible columns from terminal width and effective cell width.

- `effective_item_max_len` SHALL be derived from the longest visible label and any positive `DX_MENU_ITEM_MAX_LEN` cap.
- `cell_width` SHALL be derived from `effective_item_max_len` plus a fixed padding allowance.
- `columns` SHALL be `max(1, floor(terminal_width / cell_width))`, capped at the number of visible items.

#### Scenario: Width supports multiple columns
- **WHEN** terminal width is `120`, the longest visible label is `18` characters, `DX_MENU_ITEM_MAX_LEN=24`, and padding yields `cell_width=20`
- **THEN** the system SHALL calculate `columns=6`

#### Scenario: Width supports only one column
- **WHEN** terminal width is less than `2 * cell_width`
- **THEN** the system SHALL calculate `columns=1` and render effectively as single-column

#### Scenario: Effective max length uses visible label width
- **WHEN** the longest visible label is `3` characters and `DX_MENU_ITEM_MAX_LEN=10`
- **THEN** the system SHALL use `3` as the effective item max length for layout calculations

### Requirement: Deterministic Grid Ordering
In multicolumn mode, the candidate display SHALL preserve original candidate ordering and map candidates into a deterministic row-major grid.

#### Scenario: Row-major ordering is preserved
- **WHEN** candidates are `[A, B, C, D, E, F]` and grid capacity is 3 columns
- **THEN** rows SHALL render as `[A, B, C]` then `[D, E, F]` without reordering source rank

### Requirement: Grid Navigation Semantics
Multicolumn mode SHALL support deterministic keyboard navigation across the grid while preserving existing selection and confirmation behavior.

#### Scenario: Horizontal navigation moves between columns
- **WHEN** multicolumn mode is active and the user presses Right from a selectable cell
- **THEN** selection SHALL move to the next selectable cell in the row according to defined wrap/clamp rules

#### Scenario: Vertical navigation moves between rows
- **WHEN** multicolumn mode is active and the user presses Down from a selectable cell
- **THEN** selection SHALL move to the corresponding row-neighbor cell when available

#### Scenario: Enter confirms selected candidate
- **WHEN** multicolumn mode is active and user presses Enter on a selected candidate
- **THEN** the resulting action SHALL be equivalent to selecting the same candidate in single-column mode

### Requirement: Cell Truncation with Full-Value Context
Multicolumn cell rendering SHALL truncate long labels to fit `DX_MENU_ITEM_MAX_LEN` while keeping full selected value visible in the status context area.

#### Scenario: Long candidate label is truncated in grid cell
- **WHEN** a candidate label exceeds `DX_MENU_ITEM_MAX_LEN`
- **THEN** the rendered cell SHALL show a truncated representation that fits the configured max length

#### Scenario: Full selected value remains visible
- **WHEN** a truncated cell is selected
- **THEN** the status area SHALL show the full selected path/value

### Requirement: Protocol Compatibility with Shell Hooks
Multicolumn mode SHALL NOT change the JSON action protocol (`replace`/`noop`) consumed by shell hooks.

#### Scenario: Replace action shape unchanged
- **WHEN** a candidate is selected in multicolumn mode
- **THEN** stdout SHALL emit the same `replace` action schema used by single-column mode

#### Scenario: Cancel action shape unchanged
- **WHEN** menu is cancelled in multicolumn mode
- **THEN** stdout SHALL emit the same `noop` or query-commit action schema used by current menu behavior
