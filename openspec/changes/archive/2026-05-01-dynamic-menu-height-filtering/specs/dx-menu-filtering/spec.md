## MODIFIED Requirements

### Requirement: Filter Editing and Empty Results Handling
The menu runtime SHALL support filter editing with Backspace, but interactive editing SHALL be clamped to the initial query derived when the menu opens.

The active filter query SHALL be treated as the initial query plus any refinement characters typed during the current menu session.

Pressing Backspace SHALL remove one character only from the refinement typed during the current menu session. If no refinement characters remain, Backspace SHALL leave the active filter query unchanged.

If no candidates match the active filter query, the menu SHALL remain open, SHALL show an explicit no-match state, and SHALL continue accepting additional input, Backspace, or cancellation.

During filtering, the menu runtime SHALL reduce visible menu body height as the filtered candidate set narrows, without collapsing below a minimal interactive no-match layout.

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

#### Scenario: Filtering to a small set shrinks list footprint
- **WHEN** a filter reduces visible candidates from many rows to one row
- **THEN** the rendered menu body SHALL shrink to the smaller footprint while keeping filter/status context visible

#### Scenario: No-match state keeps minimal interactive footprint
- **WHEN** a filter query matches zero candidates
- **THEN** the menu SHALL keep a minimal interactive layout for no-match feedback and additional typing instead of collapsing to zero-height or exiting
