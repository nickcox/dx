## MODIFIED Requirements

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

## ADDED Requirements

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
