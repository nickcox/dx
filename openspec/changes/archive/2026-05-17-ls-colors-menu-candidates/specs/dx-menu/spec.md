## ADDED Requirements

### Requirement: Opt-In LS_COLORS Candidate Styling
The interactive `dx menu` renderer SHALL support opt-in candidate label styling from `LS_COLORS` when `DX_MENU_LS_COLORS=1` and `LS_COLORS` is present.

When LS_COLORS candidate styling is not enabled, candidate labels SHALL retain the existing monochrome rendering behavior.

LS_COLORS candidate styling SHALL apply only to candidate item labels during interactive rendering and SHALL NOT change candidate sourcing, ordering, filtering, selected candidate identity, status-row text, JSON action output, terminal cleanliness semantics, or shell replacement text.

Selected candidates SHALL use the existing selected-item highlight instead of LS_COLORS-derived styling.

#### Scenario: Enabled with DX_MENU_LS_COLORS and LS_COLORS
- **WHEN** `dx menu` enters interactive rendering with `DX_MENU_LS_COLORS=1`
- **AND** `LS_COLORS` is present
- **THEN** non-selected candidate item labels SHALL be styled according to each candidate path's LS_COLORS match

#### Scenario: Missing feature flag stays monochrome
- **WHEN** `dx menu` enters interactive rendering with `LS_COLORS` present
- **AND** `DX_MENU_LS_COLORS` is unset or not equal to `1`
- **THEN** candidate item labels SHALL use the existing monochrome rendering behavior

#### Scenario: Missing LS_COLORS stays monochrome
- **WHEN** `dx menu` enters interactive rendering with `DX_MENU_LS_COLORS=1`
- **AND** `LS_COLORS` is not present
- **THEN** candidate item labels SHALL use the existing monochrome rendering behavior

#### Scenario: Selection highlight overrides LS_COLORS
- **WHEN** LS_COLORS candidate styling is enabled
- **AND** a candidate item is selected
- **THEN** the selected candidate SHALL render with the existing selected-item highlight rather than its LS_COLORS-derived style

#### Scenario: Candidate styling does not affect action output
- **WHEN** LS_COLORS candidate styling is enabled
- **AND** the user accepts a selected candidate
- **THEN** stdout SHALL emit the same replace action schema and replacement value that would be emitted with monochrome candidate rendering

#### Scenario: Candidate styling works in multicolumn rendering
- **WHEN** LS_COLORS candidate styling is enabled
- **AND** the menu renders candidates in multicolumn layout
- **THEN** non-selected candidate cells SHALL apply the LS_COLORS-derived style for their corresponding candidate paths while preserving existing grid ordering, truncation, and navigation behavior
