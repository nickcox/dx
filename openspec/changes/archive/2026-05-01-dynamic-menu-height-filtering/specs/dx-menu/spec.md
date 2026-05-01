## MODIFIED Requirements

### Requirement: Stable Interactive Session
When interactive mode starts with available candidates, `dx menu` SHALL keep the menu visible until explicit user selection, explicit user cancellation, or unrecoverable runtime failure.

During interactive filtering, typed character input SHALL be treated as refinement of the initial query parsed from the shell buffer, not as arbitrary rewriting that can broaden the query beyond its starting value.

During interactive filtering, `dx menu` SHALL dynamically reduce the rendered menu height as the number of visible candidates decreases, while preserving an interactive status area.

Dynamic height updates SHALL NOT overlap prompt content, SHALL NOT leave stale rendered rows, and SHALL remain usable in both bordered and borderless modes.

#### Scenario: Menu does not auto-dismiss on open
- **WHEN** `dx menu` enters interactive mode with at least one candidate
- **THEN** it SHALL remain visible and await user input instead of immediately returning `noop`

#### Scenario: Interactive filtering remains clamped to initial query
- **WHEN** `dx menu` enters interactive mode with initial query `Do`
- **AND** the user presses Backspace without adding new refinement characters
- **THEN** the active query SHALL remain `Do` rather than broadening to `D` or the empty string

#### Scenario: Shrink menu height as filtered candidates narrow
- **WHEN** `dx menu` opens with many visible candidates and a taller menu body
- **AND** typed filtering reduces visible candidates to a smaller set
- **THEN** the rendered menu height SHALL reduce to fit the smaller set up to configured row limits

#### Scenario: No stale rows after shrink transition
- **WHEN** the menu height shrinks after filtering from many matches to few matches
- **THEN** lines that are no longer part of the menu SHALL be cleared so no stale borders, separators, or item rows remain visible

## ADDED Requirements

### Requirement: Dynamic Resize Terminal Safety
Interactive runtime SHALL preserve terminal safety guarantees while applying dynamic menu height changes.

If a dynamic resize draw step fails, `dx menu` SHALL return `{ "action": "noop" }` and restore terminal state before exit.

Dynamic height changes SHALL preserve stdout for final JSON action output and SHALL continue using TTY channels for interactive rendering.

#### Scenario: Resize draw failure exits safely
- **WHEN** a runtime draw error occurs while applying a dynamic height change
- **THEN** `dx menu` SHALL return `{ "action": "noop" }` and restore terminal state

#### Scenario: Completion-context interaction remains safe during shrink
- **WHEN** `dx menu` is running in completion context with stdout captured and TTY-backed interaction
- **AND** filtering causes menu height reductions
- **THEN** interaction SHALL remain usable and final action JSON SHALL still be emitted only on stdout
