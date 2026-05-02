## Purpose
Define expected behavior for live filtering inside `dx menu`, including incremental query updates, filter editing, exit actions, and filtered selection semantics.

## Requirements

### Requirement: In-Menu Incremental Filter Input
When `dx menu` is open with multiple candidates, the menu runtime SHALL accept printable key input as an incremental filter string and SHALL update visible candidates after each keystroke.

Filter matching SHALL be implemented by re-invoking the same completion pipeline as `dx complete <mode>` with the updated filter query on each keystroke — not by in-memory string matching against already-sourced candidates. This ensures path-prefix queries (`~/D`, `/Users/nick/D`), abbreviation expansion, and all resolver logic work identically inside the menu as in `dx complete`.

#### Scenario: Typing narrows visible list
- **WHEN** the menu opens for `cd D` with visible candidates `Desktop`, `Documents`, `Downloads`, and `Dropbox`, and the user types `o`
- **THEN** the visible list SHALL update to `Documents` and `Downloads`

#### Scenario: Case-insensitive prefix filter
- **WHEN** the menu has candidates `Documents` and `Downloads`, and the user types `do`
- **THEN** both candidates SHALL remain visible regardless of whether input is `do`, `Do`, or `DO`

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

### Requirement: Selection Semantics Over Filtered Candidates
Navigation keys (arrow keys and Tab/Shift+Tab) SHALL operate over the currently filtered candidate list.

Printable character keys, including `j` and `k`, SHALL be treated as filter input rather than navigation commands.

On Enter with a selected filtered candidate, `dx menu` SHALL return a `replace` action for that selected candidate using existing replacement-range semantics.

#### Scenario: Enter applies selected filtered candidate
- **WHEN** the user filters to `Documents` and `Downloads`, moves selection to `Downloads`, and presses Enter
- **THEN** `dx menu` SHALL return `{"action":"replace", ... "value":"<downloads-path>"}` for the selected item

#### Scenario: j and k extend the filter query
- **WHEN** the menu is open and the user presses `j` or `k`
- **THEN** the menu SHALL append that character to the active filter query instead of treating it as a navigation command
