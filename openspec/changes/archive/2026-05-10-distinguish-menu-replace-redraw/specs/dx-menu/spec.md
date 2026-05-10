## MODIFIED Requirements

### Requirement: dx menu Command Contract
The system SHALL provide a `dx menu` subcommand for interactive, context-aware selection. The command SHALL accept shell buffer context inputs and emit a structured JSON action describing how the shell should update its command line.

`dx menu` SHALL support:
- `--buffer <text>`: full current command line buffer
- `--cursor <index>`: zero-based cursor position in the buffer
- `--cwd <path>`: current working directory used for candidate context
- `--session <id>`: session identity for recents/stack contexts

The command SHALL output JSON to stdout with one of:
- `{ "action": "replace", "replaceStart": <int>, "replaceEnd": <int>, "value": <string>, "terminal": "clean" | "dirty" }`
- `{ "action": "cancel" }`
- `{ "action": "noop" }`

For `replace` actions, `terminal` SHALL describe whether terminal presentation is clean or dirty after `dx menu` returns. `terminal=clean` SHALL mean shell hooks can apply the replacement without redrawing the prompt. `terminal=dirty` SHALL mean shell hooks need to redraw or repaint the prompt after applying the replacement.

If `--cursor` exceeds the buffer length, the command SHALL clamp it to the end of the provided buffer before parsing command context.

#### Scenario: Replace action returned after selection
- **WHEN** `dx menu` is invoked with valid `--buffer`, `--cursor`, `--cwd`, and `--session`, and the user selects a candidate
- **THEN** stdout SHALL contain a JSON object with `action=replace`, replacement bounds, replacement value, and `terminal` set to either `clean` or `dirty`, and exit code SHALL be 0

#### Scenario: Cancel action returned on explicit cancellation
- **WHEN** `dx menu` is invoked with valid inputs and the user explicitly cancels the interactive menu
- **THEN** stdout SHALL contain `{ "action": "cancel" }` and exit code SHALL be 0

#### Scenario: Out-of-range cursor is clamped
- **WHEN** `dx menu --buffer "cd proj" --cursor 999` is invoked
- **THEN** the command SHALL treat the cursor as if it were positioned at the end of `cd proj`

### Requirement: Single-Candidate Fast Path
When initial candidate sourcing yields exactly one candidate and there are no additional hidden candidates, `dx menu` SHALL be allowed to return a final `replace` action immediately without starting the interactive TUI.

Single-candidate fast-path replacements SHALL set `terminal` to `clean` because no interactive TUI rendering or terminal cleanup has occurred.

#### Scenario: Non-interactive single candidate returns clean replace
- **WHEN** `dx menu` is invoked without interactive stdin and initial candidate sourcing yields exactly one candidate with no overflow
- **THEN** the command SHALL return a `replace` action for that candidate with `terminal=clean` instead of `noop`

### Requirement: Completion-Context Interactivity Contract
`dx menu` SHALL remain interactive when invoked from shell completion contexts where stdout is captured, provided input is attached to an interactive TTY.

The command SHALL preserve stdout for JSON action output and SHALL use TTY input/output channels for interactive key handling and rendering.

Interactive replacement actions that occur after TUI rendering SHALL set `terminal` to `dirty` so shell hooks can refresh prompt display after applying the replacement.

#### Scenario: Captured stdout with TTY stdin remains interactive
- **WHEN** `dx menu` is invoked via command substitution with stdout captured and stdin redirected from `/dev/tty`
- **THEN** the menu SHALL remain open for user selection and SHALL NOT immediately return `noop`

#### Scenario: Completion context returns replace after selection
- **WHEN** `dx menu` is invoked from completion context with candidates and the user selects one
- **THEN** stdout SHALL contain a `replace` action JSON payload with `terminal=dirty`
