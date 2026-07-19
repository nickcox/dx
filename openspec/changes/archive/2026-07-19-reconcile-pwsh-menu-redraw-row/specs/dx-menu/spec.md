## MODIFIED Requirements

### Requirement: dx menu Command Contract
The system SHALL provide a `dx menu` subcommand for interactive, context-aware selection. The command SHALL accept shell buffer context inputs and emit a structured JSON action describing how the shell should update its command line.

`dx menu` SHALL support:
- `--buffer <text>`: full current command line buffer
- `--cursor <index>`: zero-based cursor position in the buffer
- `--cwd <path>`: current working directory used for candidate context
- `--session <id>`: session identity for recents/stack contexts

The command SHALL output JSON to stdout with one of:
- `{ "action": "replace", "replaceStart": <int>, "replaceEnd": <int>, "value": <string>, "terminal": "clean" }`
- `{ "action": "replace", "replaceStart": <int>, "replaceEnd": <int>, "value": <string>, "terminal": "dirty", "redrawRow": <int>, "scrollRows": <int> }`
- `{ "action": "cancel", "terminal": "dirty", "redrawRow": <int>, "scrollRows": <int> }`
- `{ "action": "noop" }`

For `replace` actions, `terminal` SHALL describe whether terminal presentation is clean or dirty after `dx menu` returns. `terminal=clean` SHALL mean shell hooks can apply the replacement without redrawing the prompt. `terminal=dirty` SHALL mean shell hooks need to redraw or repaint the prompt after applying the replacement.

Dirty interactive outcomes SHALL include `redrawRow`, the zero-based viewport row containing the command-line anchor after menu space reservation, and `scrollRows`, the number of rows by which dx scrolled the terminal during reservation. Clean and noop outcomes SHALL NOT require terminal geometry.

If `--cursor` exceeds the buffer length, the command SHALL clamp it to the end of the provided buffer before parsing command context.

#### Scenario: Replace action returned after interactive selection
- **WHEN** the user selects a candidate after the interactive menu rendered
- **THEN** stdout SHALL contain `action=replace`, replacement bounds, replacement value, `terminal=dirty`, `redrawRow`, and `scrollRows`, and exit code SHALL be 0

#### Scenario: Clean replacement omits required redraw geometry
- **WHEN** the single-candidate fast path returns without touching terminal presentation
- **THEN** stdout SHALL contain a replacement with `terminal=clean` and SHALL NOT require `redrawRow` or `scrollRows`

#### Scenario: Cancel action returned on explicit cancellation
- **WHEN** a user explicitly cancels after the interactive menu rendered
- **THEN** stdout SHALL contain `action=cancel`, `terminal=dirty`, `redrawRow`, and `scrollRows`, and exit code SHALL be 0

#### Scenario: Out-of-range cursor is clamped
- **WHEN** `dx menu --buffer "cd proj" --cursor 999` is invoked
- **THEN** the command SHALL treat the cursor as if it were positioned at the end of `cd proj`

### Requirement: Completion-Context Interactivity Contract
`dx menu` SHALL remain interactive when invoked from shell completion contexts where stdout is captured, provided input is attached to an interactive TTY.

The command SHALL preserve stdout for JSON action output and SHALL use TTY input/output channels for interactive key handling and rendering.

Interactive replacement and cancellation actions that occur after TUI rendering SHALL set `terminal` to `dirty` and SHALL include the terminal redraw geometry produced by initial space reservation so shell hooks can refresh prompt display at the post-scroll location.

#### Scenario: Captured stdout with TTY stdin remains interactive
- **WHEN** `dx menu` is invoked via command substitution with stdout captured and stdin redirected from `/dev/tty`
- **THEN** the menu SHALL remain open for user selection and SHALL NOT immediately return `noop`

#### Scenario: Completion context returns dirty geometry after selection
- **WHEN** `dx menu` returns a replacement after interactive selection
- **THEN** stdout SHALL contain `terminal=dirty`, the post-scroll `redrawRow`, and the applied `scrollRows`

#### Scenario: Completion context returns dirty geometry after cancellation
- **WHEN** `dx menu` returns cancellation after interactive rendering
- **THEN** stdout SHALL contain `terminal=dirty`, the post-scroll `redrawRow`, and the applied `scrollRows`
