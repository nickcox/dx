## ADDED Requirements

### Requirement: Interactive Status Row Layout
The interactive `dx menu` status row SHALL present selected-item context as the primary left-aligned element.

When hidden-result overflow metadata is present, the status row SHALL treat it as secondary metadata that appears after selected-item context only when sufficient space is available.

When the user has typed refinement characters during the current menu session, the status row SHALL present the typed refinement as a right-aligned element prefixed with `/`.

The status row SHALL NOT display a refinement indicator before the user has typed refinement characters during the current menu session.

The status row SHALL NOT use the literal label `filter:` for the refinement indicator.

#### Scenario: Selection shown without refinement
- **WHEN** `dx menu` opens with an initial query parsed from the shell buffer
- **AND** the user has not typed any refinement characters inside the menu
- **THEN** the status row SHALL show selected-item context without a refinement indicator

#### Scenario: Typed refinement appears on the right
- **WHEN** `dx menu` opens with initial query `Do`
- **AND** the user types refinement character `w` inside the menu
- **THEN** the status row SHALL show selected-item context on the left and `/w` as a right-aligned refinement indicator

#### Scenario: Initial query is not repeated as refinement
- **WHEN** `dx menu` opens with initial query `Do`
- **AND** the user types refinement character `w` inside the menu
- **THEN** the status row refinement indicator SHALL display `/w` rather than `/Dow`

#### Scenario: Overflow metadata is secondary
- **WHEN** the current candidate source has hidden-result overflow metadata
- **AND** the status row has enough width for selection, overflow metadata, and typed refinement
- **THEN** the status row SHALL place overflow metadata after selected-item context and before any right-aligned refinement indicator

### Requirement: Status Row Compression Priority
When the terminal width cannot fit all status-row elements, `dx menu` SHALL preserve selected-item context ahead of overflow metadata and refinement visibility.

The status row SHALL drop or omit overflow metadata before truncating selected-item context or typed refinement.

The status row SHALL cap typed-refinement display width so an unusually long refinement cannot reduce selected-item context to zero width.

If the terminal is too narrow to present both useful selected-item context and useful refinement text, the status row SHALL hide the refinement indicator before hiding selected-item context.

Selected-item context and typed-refinement text MAY be truncated independently when both are visible.

#### Scenario: Overflow is omitted before selection or refinement
- **WHEN** selection, overflow metadata, and typed refinement cannot all fit in the status row
- **THEN** `dx menu` SHALL omit overflow metadata before omitting selected-item context or typed refinement

#### Scenario: Long selection preserves refinement when space allows
- **WHEN** the selected-item context is wider than the available left-side status area
- **AND** there is sufficient width to keep useful selected-item context and typed refinement visible
- **THEN** `dx menu` SHALL truncate selected-item context while keeping the refinement indicator visible on the right

#### Scenario: Long refinement cannot consume entire status row
- **WHEN** the typed refinement is unusually long
- **THEN** `dx menu` SHALL cap or truncate the refinement indicator so selected-item context remains visible

#### Scenario: Extremely narrow terminal favors selection
- **WHEN** the terminal is too narrow to show both useful selected-item context and useful typed refinement
- **THEN** `dx menu` SHALL hide the refinement indicator and show selected-item context using the available status-row width
