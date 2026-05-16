## MODIFIED Requirements

### Requirement: Interactive Status Row Layout
The interactive `dx menu` status row SHALL present selected-item context as the primary left-aligned element.

Selected-item context in the status row SHALL use the full resolved path for the selected candidate by default, not the compact candidate display label.

When hidden-result overflow metadata is present, the status row SHALL treat it as secondary metadata that appears after selected-item context only when sufficient space is available.

When the user has typed refinement characters during the current menu session, the status row SHALL present the typed refinement as a right-aligned element prefixed with `/`.

The status row SHALL NOT display a refinement indicator before the user has typed refinement characters during the current menu session.

The status row SHALL NOT use the literal label `filter:` for the refinement indicator.

Status-row selected-item display SHALL NOT change the selected candidate identity or the shell replacement text emitted when the user accepts a selection.

#### Scenario: Selection shown without refinement
- **WHEN** `dx menu` opens with an initial query parsed from the shell buffer
- **AND** the user has not typed any refinement characters inside the menu
- **THEN** the status row SHALL show selected-item context without a refinement indicator

#### Scenario: Status uses full resolved selected path
- **WHEN** the selected candidate resolves to `/Users/nick/code/personal/dx/src`
- **AND** the candidate list displays that candidate as `./src`
- **THEN** the status row SHALL show `/Users/nick/code/personal/dx/src` as selected-item context

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

#### Scenario: Status display does not affect replacement text
- **WHEN** the selected candidate resolves to `/Users/nick/code/personal/dx/src`
- **AND** existing replacement formatting would insert `./src/`
- **THEN** accepting the selection SHALL still emit replacement text `./src/`
