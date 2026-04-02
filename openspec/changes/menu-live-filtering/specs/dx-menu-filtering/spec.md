## ADDED Requirements

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
The menu runtime SHALL support filter editing with Backspace. Pressing Backspace SHALL remove one character from the active filter query and re-render the visible candidates immediately.

If no candidates match the active filter query, the menu SHALL remain open, SHALL show an explicit no-match state, and SHALL continue accepting additional input, Backspace, or cancellation.

#### Scenario: Backspace restores broader result set
- **WHEN** the user types a filter that narrows results and then presses Backspace
- **THEN** the menu SHALL broaden the visible candidate set according to the updated query

#### Scenario: No-match state remains interactive
- **WHEN** the user types a filter query that matches no candidates
- **THEN** the menu SHALL display a no-match indication and remain interactive until further input, Enter on a valid selection, or cancel

### Requirement: Exit Actions Preserve Typed Refinement
On menu exit, `dx menu` SHALL emit a single final JSON action. If the user has modified the filter query while the menu is open, that typed refinement SHALL be preserved in the shell buffer through a final `replace` action even when no candidate is selected.

If no filter delta was typed, cancel SHALL continue returning `{ "action": "noop" }`.

#### Scenario: Cancel after typing commits typed filter text
- **WHEN** the menu opens for `cd D`, the user types `o`, and then presses Esc
- **THEN** `dx menu` SHALL return a `replace` action that updates only the active query token to `Do`

#### Scenario: Cancel without typing remains noop
- **WHEN** the menu opens and the user presses Esc without modifying filter text
- **THEN** `dx menu` SHALL return `{ "action": "noop" }`

### Requirement: Selection Semantics Over Filtered Candidates
Navigation keys (arrow keys, Tab/Shift+Tab, and vim j/k) SHALL operate over the currently filtered candidate list.

On Enter with a selected filtered candidate, `dx menu` SHALL return a `replace` action for that selected candidate using existing replacement-range semantics.

#### Scenario: Enter applies selected filtered candidate
- **WHEN** the user filters to `Documents` and `Downloads`, moves selection to `Downloads`, and presses Enter
- **THEN** `dx menu` SHALL return `{"action":"replace", ... "value":"<downloads-path>"}` for the selected item
