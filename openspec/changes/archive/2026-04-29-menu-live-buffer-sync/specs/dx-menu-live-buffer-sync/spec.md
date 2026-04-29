## ADDED Requirements

### Requirement: Per-Keypress Typed Query Sync Actions
When interactive menu mode is active, typed filter keystrokes (printable input and Backspace edits) SHALL produce incremental replace actions that reflect query-token edits in the shell input buffer as each keypress occurs.

Incremental sync actions SHALL update only the active query token range and SHALL NOT modify command prefixes or unrelated suffix segments.

#### Scenario: Typing emits incremental replace updates
- **WHEN** menu is open for `cd D` and the user types `o`
- **THEN** menu action output SHALL include an incremental replace update that changes the query token from `D` to `Do`

#### Scenario: Backspace emits incremental replace update
- **WHEN** menu query is `Dow` and user presses Backspace
- **THEN** menu action output SHALL include an incremental replace update that changes the query token to `Do`

### Requirement: Selection Navigation Does Not Mutate Buffer
Selection-only navigation events (arrow keys, Tab/Shift+Tab, j/k) SHALL update menu highlight state without emitting shell-buffer mutation actions.

#### Scenario: Arrow navigation leaves shell line unchanged
- **WHEN** menu is open and user moves selection with Down/Up keys without typing
- **THEN** no incremental replace action SHALL be emitted for shell buffer updates

#### Scenario: Tab navigation leaves shell line unchanged
- **WHEN** user cycles candidates with Tab/Shift+Tab only
- **THEN** shell input buffer text SHALL remain unchanged until accept/cancel outcome processing

### Requirement: Final Accept/Cancel Outcomes with Typed Persistence
Menu sessions SHALL end with explicit accept or cancel outcomes.

- Accept SHALL emit final selected-candidate replacement.
- Cancel SHALL preserve typed query edits already synced to the shell buffer.
- Cancel with no typed query change SHALL leave shell buffer unchanged.

#### Scenario: Accept applies selected candidate after typed sync
- **WHEN** user types filter text, navigates selection, and presses Enter
- **THEN** final action SHALL replace active token with selected candidate path

#### Scenario: Cancel retains typed query edits
- **WHEN** user types additional query characters and presses Esc
- **THEN** the shell buffer SHALL retain those typed query edits

#### Scenario: Cancel without edits is no-op
- **WHEN** user opens menu and cancels without typing
- **THEN** no shell buffer mutation SHALL occur
