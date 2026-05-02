## MODIFIED Requirements

### Requirement: Exit Actions Preserve Typed Refinement
On menu exit, `dx menu` SHALL emit a single final JSON action.

If the user accepts a candidate selection, `dx menu` SHALL return a `replace` action for the selected value.

If the user explicitly cancels the menu, `dx menu` SHALL discard any typed in-menu refinement and return `{ "action": "cancel" }`.

Fallback `noop` outcomes SHALL be reserved for non-interactive, unavailable, or runtime-failure paths where the menu does not complete a normal interactive accept/cancel session.

#### Scenario: Cancel after typing restores original prompt state
- **WHEN** the menu opens for `cd D`, the user types `o`, and then presses Esc
- **THEN** `dx menu` SHALL return `{ "action": "cancel" }` rather than committing `Do` into the shell buffer

#### Scenario: Cancel after net-zero edits returns cancel
- **WHEN** the menu opens for `cd Do`
- **AND** the user types `w` and then presses Backspace to return to `Do`
- **THEN** `dx menu` SHALL return `{ "action": "cancel" }`

#### Scenario: Cancel without typing returns cancel
- **WHEN** the menu opens and the user presses Esc without modifying filter text
- **THEN** `dx menu` SHALL return `{ "action": "cancel" }`
