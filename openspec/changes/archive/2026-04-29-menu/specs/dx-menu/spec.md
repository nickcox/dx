## ADDED Requirements

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
- **THEN** candidate ordering SHALL match `dx complete ancestors` ordering for the same cwd

#### Scenario: Frecents parity with provider output
- **WHEN** `dx menu` maps to `frecents` with query `proj`
- **THEN** candidates SHALL preserve the same provider-relative ordering used by `dx complete frecents proj`, except that duplicates and any exact current-directory entry MAY be removed

#### Scenario: Stack mode uses provided session
- **WHEN** `dx menu` maps to stack mode with `--session 12345`
- **THEN** candidates SHALL be sourced from session `12345` stack history

### Requirement: Non-Interactive Fallback Behavior
If interactive menu rendering is unavailable, `dx menu` SHALL degrade gracefully by returning `noop` with exit code 0 and no stderr diagnostics unless opt-in debug logging is enabled.

If interactive initialization, rendering, or event handling becomes unavailable during startup or runtime, `dx menu` SHALL return `noop` and restore terminal state before exit when terminal setup had already begun.

#### Scenario: No interactive stdin returns noop
- **WHEN** `dx menu` is invoked without interactive stdin
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

#### Scenario: Menu does not auto-dismiss on open
- **WHEN** `dx menu` enters interactive mode with at least one candidate
- **THEN** it SHALL remain visible and await user input instead of immediately returning `noop`

### Requirement: Selection Replacement Semantics
For `replace` actions, `replaceStart` and `replaceEnd` SHALL define a half-open byte range in the original buffer to replace. `value` SHALL be the selected absolute directory path (or a command token that includes it when mode requires command context preservation).

Replacement bounds SHALL only target the active query token under the cursor and SHALL NOT modify unrelated buffer segments.

#### Scenario: Replace only query token
- **WHEN** buffer is `cd pr --flag` and user selects `/home/user/projects`
- **THEN** replacement bounds SHALL cover only `pr`, and resulting buffer SHALL be `cd /home/user/projects --flag`

#### Scenario: Preserve command prefix
- **WHEN** buffer is `up co` and user selects `/home/user/code`
- **THEN** replacement SHALL preserve `up ` prefix and update only selector token
