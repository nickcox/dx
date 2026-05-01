## MODIFIED Requirements

### Requirement: Filter Editing and Empty Results Handling
The menu runtime SHALL support filter editing with Backspace, but interactive editing SHALL be clamped to the initial query derived when the menu opens.

The active filter query SHALL be treated as the initial query plus any refinement characters typed during the current menu session.

Pressing Backspace SHALL remove one character only from the refinement typed during the current menu session. If no refinement characters remain, Backspace SHALL leave the active filter query unchanged.

If no candidates match the active filter query, the menu SHALL remain open, SHALL show an explicit no-match state, and SHALL continue accepting additional input, Backspace, or cancellation.

#### Scenario: Backspace removes only typed refinement
- **WHEN** the menu opens with initial query `Do`, the user types `w`, and then presses Backspace
- **THEN** the active filter query SHALL return to `Do`

#### Scenario: Backspace does not broaden past initial query
- **WHEN** the menu opens with initial query `Do`
- **AND** the user presses Backspace before typing any additional refinement
- **THEN** the active filter query SHALL remain `Do`

#### Scenario: Empty-seed query can still narrow and return to empty
- **WHEN** the menu opens with an empty initial query
- **AND** the user types `a` and then presses Backspace
- **THEN** the active filter query SHALL return to the empty string

#### Scenario: No-match state remains interactive
- **WHEN** the user types a filter query that matches no candidates
- **THEN** the menu SHALL display a no-match indication and remain interactive until further input, Enter on a valid selection, or cancel

### Requirement: Exit Actions Preserve Typed Refinement
On menu exit, `dx menu` SHALL emit a single final JSON action.

If the final active query differs from the initial query, cancellation SHALL preserve that final refinement in the shell buffer through a `replace` action even when no candidate is selected.

If the final active query is identical to the initial query, cancel SHALL return `{ "action": "noop" }`.

#### Scenario: Cancel after typing commits typed filter text
- **WHEN** the menu opens for `cd D`, the user types `o`, and then presses Esc
- **THEN** `dx menu` SHALL return a `replace` action that updates only the active query token to `Do`

#### Scenario: Cancel after net-zero edits remains noop
- **WHEN** the menu opens for `cd Do`
- **AND** the user types `w` and then presses Backspace to return to `Do`
- **THEN** `dx menu` SHALL return `{ "action": "noop" }`

#### Scenario: Cancel without typing remains noop
- **WHEN** the menu opens and the user presses Esc without modifying filter text
- **THEN** `dx menu` SHALL return `{ "action": "noop" }`
