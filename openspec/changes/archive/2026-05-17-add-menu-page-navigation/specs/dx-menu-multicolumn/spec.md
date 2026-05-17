## ADDED Requirements

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
