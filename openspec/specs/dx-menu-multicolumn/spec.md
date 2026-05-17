## Purpose
Define expected behavior for multicolumn `dx menu` rendering, including activation rules, layout calculation, grid navigation, truncation, and shell-hook protocol compatibility.

## Requirements

### Requirement: Multicolumn Activation via Item Max Length
The system SHALL use `DX_MENU_ITEM_MAX_LEN` to control multicolumn rendering and optional cell text length limits.

- If `DX_MENU_ITEM_MAX_LEN` is unset, empty, or non-numeric, the menu SHALL default to a maximum item length of `80` characters for multicolumn calculations.
- If `DX_MENU_ITEM_MAX_LEN` is a valid integer greater than or equal to `1`, the menu SHALL enable multicolumn calculations for that render cycle and use the value as an upper bound for cell text length.
- If `DX_MENU_ITEM_MAX_LEN` is `0` or negative, the menu SHALL render using single-column layout.

#### Scenario: Missing or invalid value uses default max length
- **WHEN** `DX_MENU_ITEM_MAX_LEN` is unset, empty, or non-numeric
- **THEN** the menu SHALL default to a maximum item length of `80` for multicolumn calculations

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

### Requirement: Multicolumn Page Key Navigation
When multicolumn mode is active, PageDown and PageUp navigation SHALL move selection by one visible grid page while preserving row-major candidate ordering.

In multicolumn layout, one visible page SHALL equal the current visible row count multiplied by the active column count.

Multicolumn page navigation SHALL clamp at the first and last candidate and SHALL NOT wrap around the grid.

#### Scenario: PageDown advances by visible grid capacity
- **WHEN** multicolumn mode renders 4 visible rows and 3 columns
- **AND** the selected candidate index is 0
- **AND** the user presses PageDown
- **THEN** the selected candidate index SHALL become 12

#### Scenario: PageUp moves backward by visible grid capacity
- **WHEN** multicolumn mode renders 4 visible rows and 3 columns
- **AND** the selected candidate index is 14
- **AND** the user presses PageUp
- **THEN** the selected candidate index SHALL become 2

#### Scenario: Multicolumn page navigation clamps at grid boundaries
- **WHEN** multicolumn mode is active
- **AND** the selected candidate is within one visible grid page of the end of the candidate list
- **AND** the user presses PageDown
- **THEN** the selected candidate SHALL become the last candidate and SHALL NOT wrap to the beginning

### Requirement: Cell Truncation with Selected-Item Context
Multicolumn cell rendering SHALL truncate long labels to fit the active cell text limit while keeping selected-item context visible in the status area.

#### Scenario: Long candidate label is truncated in grid cell
- **WHEN** a candidate label exceeds the active cell text limit
- **THEN** the rendered cell SHALL show a truncated representation that fits the configured column width

#### Scenario: Selected-item context remains visible
- **WHEN** a truncated cell is selected
- **THEN** the status area SHALL show the selected item's display label or equivalent selected-item context, even if the grid cell itself is truncated

### Requirement: Dynamic Height Reduction in Multicolumn Mode
When multicolumn mode is active, menu height SHALL be recomputed from the current filtered grid row count after each filter update.

The rendered multicolumn menu SHALL shrink as filtered results require fewer rows, while respecting configured maximum rows.

This behavior SHALL apply in both bordered and borderless modes.

#### Scenario: Multicolumn height shrinks with fewer grid rows
- **WHEN** a multicolumn menu initially requires several grid rows
- **AND** filtering reduces results to a single grid row
- **THEN** the menu height SHALL shrink to match the reduced row count within configured limits

#### Scenario: Bordered multicolumn shrink keeps border integrity
- **WHEN** bordered multicolumn mode shrinks from a taller to a shorter height
- **THEN** the resulting border SHALL remain visually complete with no stale border fragments below the new bottom edge

#### Scenario: Borderless multicolumn shrink clears trailing separator and scrollbar artifacts
- **WHEN** borderless multicolumn mode shrinks and no longer needs previously rendered trailing rows
- **THEN** vacated rows and any prior scrollbar/separator artifacts SHALL be cleared

### Requirement: Menu Max Rows Default
The system SHALL default `DX_MENU_MAX_ROWS` to `20` when the environment variable is unset, empty, or contains a non-positive value.

#### Scenario: Unset uses default
- **WHEN** `DX_MENU_MAX_ROWS` is unset
- **THEN** the menu SHALL use `20` as the maximum visible row count

#### Scenario: Empty uses default
- **WHEN** `DX_MENU_MAX_ROWS` is set to an empty string
- **THEN** the menu SHALL use `20` as the maximum visible row count

#### Scenario: Invalid value uses default
- **WHEN** `DX_MENU_MAX_ROWS` is set to `"abc"`
- **THEN** the menu SHALL use `20` as the maximum visible row count

#### Scenario: Zero uses default
- **WHEN** `DX_MENU_MAX_ROWS=0`
- **THEN** the menu SHALL use `20` as the maximum visible row count

#### Scenario: Negative uses default
- **WHEN** `DX_MENU_MAX_ROWS=-3`
- **THEN** the menu SHALL use `20` as the maximum visible row count

#### Scenario: Positive value is honored
- **WHEN** `DX_MENU_MAX_ROWS=15`
- **THEN** the menu SHALL use `15` as the maximum visible row count

### Requirement: Protocol Compatibility with Shell Hooks
Multicolumn mode SHALL NOT change the JSON action protocol (`replace`/`noop`) consumed by shell hooks.

#### Scenario: Replace action shape unchanged
- **WHEN** a candidate is selected in multicolumn mode
- **THEN** stdout SHALL emit the same `replace` action schema used by single-column mode

#### Scenario: Cancel action shape unchanged
- **WHEN** menu is cancelled in multicolumn mode
- **THEN** stdout SHALL emit the same `noop` or query-commit action schema used by current menu behavior
