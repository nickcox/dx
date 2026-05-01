## Purpose
Define expected behavior for the `dx menu` interactive selector, including buffer parsing, candidate sourcing, fallback behavior, completion-context interactivity, terminal cleanup, and shell buffer replacement semantics.

## Requirements

### Requirement: dx menu Command Contract
The system SHALL provide a `dx menu` subcommand for interactive, context-aware selection. The command SHALL accept shell buffer context inputs and emit a structured JSON action describing how the shell should update its command line.

`dx menu` SHALL support:
- `--buffer <text>`: full current command line buffer
- `--cursor <index>`: zero-based cursor position in the buffer
- `--cwd <path>`: current working directory used for candidate context
- `--session <id>`: session identity for recents/stack contexts

The command SHALL output JSON to stdout with one of:
- `{ "action": "replace", "replaceStart": <int>, "replaceEnd": <int>, "value": <string> }`
- `{ "action": "noop" }`

If `--cursor` exceeds the buffer length, the command SHALL clamp it to the end of the provided buffer before parsing command context.

#### Scenario: Replace action returned after selection
- **WHEN** `dx menu` is invoked with valid `--buffer`, `--cursor`, `--cwd`, and `--session`, and the user selects a candidate
- **THEN** stdout SHALL contain a JSON object with `action=replace`, replacement bounds, and replacement value, and exit code SHALL be 0

#### Scenario: Noop action on cancel without query edits
- **WHEN** `dx menu` is invoked with valid inputs, the user does not change the active query, and the user cancels the menu
- **THEN** stdout SHALL contain `{ "action": "noop" }` and exit code SHALL be 0

#### Scenario: Out-of-range cursor is clamped
- **WHEN** `dx menu --buffer "cd proj" --cursor 999` is invoked
- **THEN** the command SHALL treat the cursor as if it were positioned at the end of `cd proj`

### Requirement: Context-to-Mode Mapping
`dx menu` SHALL infer the candidate source mode from command-buffer context and map to existing completion capabilities:

- `cd <query>` -> `paths`
- `up <query>` -> `ancestors`
- `cdf|z <query>` -> `frecents`
- `cdr <query>` -> `recents`
- `back|cd- <query>` -> `stack` with `direction=back`
- `forward|cd+ <query>` -> `stack` with `direction=forward`

If buffer context does not match a supported dx navigation command, `dx menu` SHALL return `noop`.

#### Scenario: cd buffer maps to paths mode
- **WHEN** `dx menu` receives buffer `cd pr` with cursor at end of `pr`
- **THEN** it SHALL build candidates using `paths` mode semantics

#### Scenario: back buffer maps to stack back mode
- **WHEN** `dx menu` receives buffer `back 2` with cursor in selector token
- **THEN** it SHALL build candidates using `stack` mode with `direction=back`

#### Scenario: Unsupported buffer returns noop
- **WHEN** `dx menu` receives buffer `git status`
- **THEN** it SHALL return `{ "action": "noop" }`

### Requirement: Candidate Source Reuse
`dx menu` SHALL reuse the same candidate-generation pipelines as `dx complete` for each mapped mode.

For `paths` mode, menu candidate ordering SHALL match the corresponding `dx complete` output for equivalent query and cwd.

For non-`paths` modes, menu candidate ordering SHALL preserve the underlying provider order after de-duplication and removal of any candidate that resolves to the current working directory.

For mapped modes requiring session context (`recents`, `stack`), `dx menu` SHALL use the provided `--session` value.

#### Scenario: Ancestors ordering parity
- **WHEN** `dx menu` maps to `ancestors` from `/home/user/code/projects/dx`
- **THEN** candidate ordering SHALL preserve the same provider-relative ordering as `dx complete ancestors` after any exact current-directory entry is removed

#### Scenario: Frecents parity with provider output
- **WHEN** `dx menu` maps to `frecents` with query `proj`
- **THEN** candidates SHALL preserve the same provider-relative ordering used by `dx complete frecents proj`, except that duplicates and any exact current-directory entry MAY be removed

#### Scenario: Stack mode uses provided session
- **WHEN** `dx menu` maps to stack mode with `--session 12345`
- **THEN** candidates SHALL be sourced from session `12345` stack history

### Requirement: Single-Candidate Fast Path
When initial candidate sourcing yields exactly one candidate and there are no additional hidden candidates, `dx menu` MAY return a final `replace` action immediately without starting the interactive TUI.

#### Scenario: Non-interactive single candidate returns replace
- **WHEN** `dx menu` is invoked without interactive stdin and initial candidate sourcing yields exactly one candidate with no overflow
- **THEN** the command MAY return a `replace` action for that candidate instead of `noop`

### Requirement: Non-Interactive Fallback Behavior
If interactive menu rendering is unavailable and the command does not take the single-candidate fast path, `dx menu` SHALL degrade gracefully by returning `noop` with exit code 0 and no stderr diagnostics unless opt-in debug logging is enabled.

If interactive initialization, rendering, or event handling becomes unavailable during startup or runtime, `dx menu` SHALL return `noop` and restore terminal state before exit when terminal setup had already begun.

#### Scenario: No interactive stdin returns noop
- **WHEN** `dx menu` is invoked without interactive stdin and initial candidate sourcing does not produce exactly one final candidate
- **THEN** it SHALL output `{ "action": "noop" }` and exit 0

#### Scenario: Interactive runtime failure returns noop safely
- **WHEN** `dx menu` begins interactive initialization or rendering and encounters a terminal runtime error
- **THEN** it SHALL return `{ "action": "noop" }` and leave terminal state restored

### Requirement: Completion-Context Interactivity Contract
`dx menu` SHALL remain interactive when invoked from shell completion contexts where stdout is captured, provided input is attached to an interactive TTY.

The command SHALL preserve stdout for JSON action output and SHALL use TTY input/output channels for interactive key handling and rendering.

#### Scenario: Captured stdout with TTY stdin remains interactive
- **WHEN** `dx menu` is invoked via command substitution with stdout captured and stdin redirected from `/dev/tty`
- **THEN** the menu SHALL remain open for user selection and SHALL NOT immediately return `noop`

#### Scenario: Completion context returns replace after selection
- **WHEN** `dx menu` is invoked from completion context with candidates and the user selects one
- **THEN** stdout SHALL contain a `replace` action JSON payload

### Requirement: Terminal Lifecycle Safety
Interactive menu execution SHALL restore terminal state on all exit paths, including selection, Esc cancel, Ctrl+C cancel, read errors, and draw/render errors.

Terminal restoration SHALL include raw mode disablement and alternate-screen/mouse-capture teardown when previously enabled.

#### Scenario: Ctrl+C cancel restores terminal state
- **WHEN** the user presses Ctrl+C while the menu is open
- **THEN** `dx menu` SHALL restore terminal state before returning its final action to the shell

#### Scenario: Render error restores terminal state
- **WHEN** a draw/read error occurs during interactive menu execution
- **THEN** `dx menu` SHALL return `{ "action": "noop" }` and restore terminal state

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

### Requirement: Selection Replacement Semantics
For `replace` actions, `replaceStart` and `replaceEnd` SHALL define a half-open byte range in the original buffer to replace. `value` SHALL be the formatted replacement token produced for the selected candidate.

For `paths` mode, the replacement formatter SHALL preserve the user's query style when practical:

- cwd-relative selections MAY be emitted as `./child/`
- parent-relative selections MAY be emitted as `../sibling/`
- explicitly absolute input SHALL preserve absolute replacement output

Paths-mode replacements SHALL include a trailing slash and MAY include shell quoting when needed for the selected path text.

For non-`paths` modes, replacements SHALL identify the selected destination without a trailing slash and MAY include shell quoting when needed.

Replacement bounds SHALL only target the active query token under the cursor and SHALL NOT modify unrelated buffer segments.

#### Scenario: Relative query preserves dot-slash style
- **WHEN** buffer is `cd b`, cwd contains `./benches`, and the selected candidate is that child directory
- **THEN** the returned replacement value MAY be `./benches/`

#### Scenario: Explicit absolute query preserves absolute replacement
- **WHEN** buffer is `cd /tmp/b`, the selected candidate is `/tmp/benches`, and `paths` mode is active
- **THEN** the returned replacement value SHALL be `/tmp/benches/`

#### Scenario: Replace only query token
- **WHEN** buffer is `cd pr --flag` and the selected replacement token is `./projects/`
- **THEN** replacement bounds SHALL cover only `pr`, and resulting buffer SHALL be `cd ./projects/ --flag`

#### Scenario: Preserve command prefix
- **WHEN** buffer is `up co` and user selects `/home/user/code`
- **THEN** replacement SHALL preserve `up ` prefix and update only selector token
