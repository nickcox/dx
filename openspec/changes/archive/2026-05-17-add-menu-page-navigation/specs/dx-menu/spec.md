## ADDED Requirements

### Requirement: Page Key Navigation
The interactive `dx menu` renderer SHALL support PageDown and PageUp keyboard navigation when candidate items are visible.

PageDown SHALL move the selected candidate forward by one visible page. PageUp SHALL move the selected candidate backward by one visible page.

In single-column layout, one visible page SHALL be the current visible candidate row count.

Page key navigation SHALL clamp at the first and last candidate and SHALL NOT wrap around the candidate list.

Page key navigation SHALL NOT change candidate sourcing, filtering, selected candidate identity semantics, status-row rendering rules, JSON action output, or shell replacement text.

#### Scenario: PageDown advances by visible rows
- **WHEN** `dx menu` is rendered in single-column layout with 10 visible candidate rows
- **AND** the selected candidate index is 0
- **AND** the user presses PageDown
- **THEN** the selected candidate index SHALL become 10

#### Scenario: PageUp moves backward by visible rows
- **WHEN** `dx menu` is rendered in single-column layout with 10 visible candidate rows
- **AND** the selected candidate index is 15
- **AND** the user presses PageUp
- **THEN** the selected candidate index SHALL become 5

#### Scenario: Page navigation clamps at boundaries
- **WHEN** the selected candidate is within one page of the end of the candidate list
- **AND** the user presses PageDown
- **THEN** the selected candidate SHALL become the last candidate and SHALL NOT wrap to the beginning

#### Scenario: Page navigation preserves output behavior
- **WHEN** the user moves selection with PageDown or PageUp and accepts a candidate
- **THEN** stdout SHALL emit the same replace action schema and replacement formatting used for accepting that candidate after arrow-key navigation
