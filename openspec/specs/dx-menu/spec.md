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

### Requirement: Explicit Mode Invocation for Mapped Commands
When menu mode is active, `dx menu` SHALL support explicit mapped-command invocation via `--mode <mode>` in addition to built-in dx navigation command mappings.

For mapped external commands, `dx menu` SHALL use the explicit mode argument instead of consulting runtime mapping configuration.

If no explicit mode is provided and the command at the cursor is not a built-in supported context, `dx menu` SHALL return `noop`.

#### Scenario: Explicit path mode invokes mapped command handling
- **WHEN** menu mode is active, `dx menu --mode path` is invoked for buffer command `ls`
- **THEN** `dx menu` SHALL build candidates using mapped mode `path`

#### Scenario: Unmapped non-dx command remains noop
- **WHEN** menu mode is active, no explicit mode is provided, and buffer command is `git`
- **THEN** `dx menu` SHALL return `{ "action": "noop" }`

### Requirement: Mapped Command Candidate Dispatch by Mode
For explicit mapped-command invocations, `dx menu` SHALL dispatch candidate sourcing according to the requested mode (`path`, `directory`, `file`) using `dx-smart` candidate behavior.

If no candidate can produce a replace action, `dx menu` SHALL return `noop` so shell hooks can apply native fallback.

#### Scenario: File mode mapped command offers file candidates
- **WHEN** `dx menu --mode file` is invoked and menu context is `cat re`
- **THEN** `dx menu` SHALL produce candidates filtered to files for the active token

#### Scenario: No candidate for mapped command returns noop
- **WHEN** mapped command context produces no valid candidates for the active token
- **THEN** `dx menu` SHALL return `{ "action": "noop" }`

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
When initial candidate sourcing yields exactly one candidate and there are no additional hidden candidates, `dx menu` SHALL be allowed to return a final `replace` action immediately without starting the interactive TUI.

Single-candidate fast-path replacements SHALL set `terminal` to `clean` because no interactive TUI rendering or terminal cleanup has occurred.

#### Scenario: Non-interactive single candidate returns clean replace
- **WHEN** `dx menu` is invoked without interactive stdin and initial candidate sourcing yields exactly one candidate with no overflow
- **THEN** the command SHALL return a `replace` action for that candidate with `terminal=clean` instead of `noop`

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

Interactive replacement actions that occur after TUI rendering SHALL set `terminal` to `dirty` so shell hooks can refresh prompt display after applying the replacement.

#### Scenario: Captured stdout with TTY stdin remains interactive
- **WHEN** `dx menu` is invoked via command substitution with stdout captured and stdin redirected from `/dev/tty`
- **THEN** the menu SHALL remain open for user selection and SHALL NOT immediately return `noop`

#### Scenario: Completion context returns replace after selection
- **WHEN** `dx menu` is invoked from completion context with candidates and the user selects one
- **THEN** stdout SHALL contain a `replace` action JSON payload with `terminal=dirty`

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

### Requirement: Page Key Navigation
The interactive `dx menu` renderer SHALL support PageDown and PageUp keyboard navigation when candidate items are visible.

PageDown SHALL move the selected candidate forward by one visible page. PageUp SHALL move the selected candidate backward by one visible page.

In single-column layout, one visible page SHALL be the current visible candidate row count.

Page key navigation SHALL clamp at the first and last candidate and SHALL NOT wrap around the candidate list.

Page key navigation SHALL NOT change candidate sourcing, filtering, selected candidate identity semantics, status-row rendering rules, JSON action output, or shell replacement text.

#### Scenario: PageDown advances by visible rows
- **WHEN** `dx menu` is rendered in single-column layout with 10 visible candidate rows
- **AND** the selected candidate index is 0
- **AND** the user presses PageDown
- **THEN** the selected candidate index SHALL become 10

#### Scenario: PageUp moves backward by visible rows
- **WHEN** `dx menu` is rendered in single-column layout with 10 visible candidate rows
- **AND** the selected candidate index is 15
- **AND** the user presses PageUp
- **THEN** the selected candidate index SHALL become 5

#### Scenario: Page navigation clamps at boundaries
- **WHEN** the selected candidate is within one page of the end of the candidate list
- **AND** the user presses PageDown
- **THEN** the selected candidate SHALL become the last candidate and SHALL NOT wrap to the beginning

#### Scenario: Page navigation preserves output behavior
- **WHEN** the user moves selection with PageDown or PageUp and accepts a candidate
- **THEN** stdout SHALL emit the same replace action schema and replacement formatting used for accepting that candidate after arrow-key navigation

### Requirement: Interactive Status Row Layout
The interactive `dx menu` status row SHALL present selected-item context as the primary left-aligned element.

Selected-item context in the status row SHALL use the full resolved path for the selected candidate by default, not the compact candidate display label.

When hidden-result overflow metadata is present, the status row SHALL treat it as secondary metadata that appears after selected-item context only when sufficient space is available.

When the user has typed refinement characters during the current menu session, the status row SHALL present the typed refinement as a right-aligned element prefixed with `/`.

The status row SHALL NOT display a refinement indicator before the user has typed refinement characters during the current menu session.

The status row SHALL NOT use the literal label `filter:` for the refinement indicator.

Status-row selected-item display SHALL NOT change the selected candidate identity or the shell replacement text emitted when the user accepts a selection.

#### Scenario: Selection shown without refinement
- **WHEN** `dx menu` opens with an initial query parsed from the shell buffer
- **AND** the user has not typed any refinement characters inside the menu
- **THEN** the status row SHALL show selected-item context without a refinement indicator

#### Scenario: Status uses full resolved selected path
- **WHEN** the selected candidate resolves to `/Users/nick/code/personal/dx/src`
- **AND** the candidate list displays that candidate as `./src`
- **THEN** the status row SHALL show `/Users/nick/code/personal/dx/src` as selected-item context

#### Scenario: Typed refinement appears on the right
- **WHEN** `dx menu` opens with initial query `Do`
- **AND** the user types refinement character `w` inside the menu
- **THEN** the status row SHALL show selected-item context on the left and `/w` as a right-aligned refinement indicator

#### Scenario: Initial query is not repeated as refinement
- **WHEN** `dx menu` opens with initial query `Do`
- **AND** the user types refinement character `w` inside the menu
- **THEN** the status row refinement indicator SHALL display `/w` rather than `/Dow`

#### Scenario: Overflow metadata is secondary
- **WHEN** the current candidate source has hidden-result overflow metadata
- **AND** the status row has enough width for selection, overflow metadata, and typed refinement
- **THEN** the status row SHALL place overflow metadata after selected-item context and before any right-aligned refinement indicator

#### Scenario: Status display does not affect replacement text
- **WHEN** the selected candidate resolves to `/Users/nick/code/personal/dx/src`
- **AND** existing replacement formatting would insert `./src/`
- **THEN** accepting the selection SHALL still emit replacement text `./src/`

### Requirement: Status Row Compression Priority
When the terminal width cannot fit all status-row elements, `dx menu` SHALL preserve selected-item context ahead of overflow metadata and refinement visibility.

The status row SHALL drop or omit overflow metadata before truncating selected-item context or typed refinement.

The status row SHALL cap typed-refinement display width so an unusually long refinement cannot reduce selected-item context to zero width.

If the terminal is too narrow to present both useful selected-item context and useful refinement text, the status row SHALL hide the refinement indicator before hiding selected-item context.

Selected-item context and typed-refinement text MAY be truncated independently when both are visible.

#### Scenario: Overflow is omitted before selection or refinement
- **WHEN** selection, overflow metadata, and typed refinement cannot all fit in the status row
- **THEN** `dx menu` SHALL omit overflow metadata before omitting selected-item context or typed refinement

#### Scenario: Long selection preserves refinement when space allows
- **WHEN** the selected-item context is wider than the available left-side status area
- **AND** there is sufficient width to keep useful selected-item context and typed refinement visible
- **THEN** `dx menu` SHALL truncate selected-item context while keeping the refinement indicator visible on the right

#### Scenario: Long refinement cannot consume entire status row
- **WHEN** the typed refinement is unusually long
- **THEN** `dx menu` SHALL cap or truncate the refinement indicator so selected-item context remains visible

#### Scenario: Extremely narrow terminal favors selection
- **WHEN** the terminal is too narrow to show both useful selected-item context and useful typed refinement
- **THEN** `dx menu` SHALL hide the refinement indicator and show selected-item context using the available status-row width

### Requirement: Query-Style Candidate Labels
For filesystem path menu modes, `dx menu` SHALL render candidate item labels using the path style implied by the user's active query.

Filesystem path menu modes SHALL include `paths`, `path`, `directory`, and `file` modes.

When the active query is empty or a bare relative token without an explicit path prefix, cwd-local candidates SHALL be displayed as bare relative labels without a leading `./`.

When the active query starts with `./`, cwd-local candidates SHALL be displayed with a leading `./`.

When the active query starts with one or more `../` path segments, candidates representable relative to cwd SHALL be displayed using parent-relative labels that preserve the typed parent-relative style.

When the active query starts with `~` or `~/`, candidates under the user's home directory SHALL be displayed with a leading `~/`.

When the active query starts with `/`, candidates SHALL be displayed as absolute paths.

Candidate label style SHALL NOT change candidate sourcing, filtering, ranking, selected candidate identity, status-row selected path display, JSON action shape, or accepted replacement text.

#### Scenario: Empty cd query shows bare cwd children
- **WHEN** `dx menu` opens for `cd <tab>` in `/Users/nick/project`
- **AND** a candidate resolves to `/Users/nick/project/src`
- **THEN** the candidate item label SHALL be `src` rather than `./src`

#### Scenario: Bare relative query shows bare cwd child
- **WHEN** `dx menu` opens for `cd s<tab>` in `/Users/nick/project`
- **AND** a candidate resolves to `/Users/nick/project/src`
- **THEN** the candidate item label SHALL be `src`

#### Scenario: Dot-relative query preserves dot prefix
- **WHEN** `dx menu` opens for `cd ./<tab>` in `/Users/nick/project`
- **AND** a candidate resolves to `/Users/nick/project/src`
- **THEN** the candidate item label SHALL be `./src`

#### Scenario: Parent-relative query preserves parent prefix
- **WHEN** `dx menu` opens for `cd ../<tab>` in `/Users/nick/project`
- **AND** a candidate resolves to `/Users/nick/sibling`
- **THEN** the candidate item label SHALL be `../sibling`

#### Scenario: Home query preserves home prefix
- **WHEN** `dx menu` opens for `cd ~/<tab>`
- **AND** a candidate resolves under the user's home directory as `/Users/nick/code`
- **THEN** the candidate item label SHALL be `~/code`

#### Scenario: Absolute query preserves absolute path
- **WHEN** `dx menu` opens for `cd /Users/nick/<tab>`
- **AND** a candidate resolves to `/Users/nick/code`
- **THEN** the candidate item label SHALL be `/Users/nick/code`

#### Scenario: Label style does not affect replacement
- **WHEN** `dx menu` opens for `cd s<tab>` in `/Users/nick/project`
- **AND** a candidate item label is displayed as `src`
- **THEN** accepting the candidate SHALL preserve existing replacement formatting behavior

### Requirement: Opt-In LS_COLORS Candidate Styling
The interactive `dx menu` renderer SHALL support opt-in candidate label styling from `LS_COLORS` when `DX_MENU_LS_COLORS=1` and `LS_COLORS` is present.

When LS_COLORS candidate styling is not enabled, candidate labels SHALL retain the existing monochrome rendering behavior.

LS_COLORS candidate styling SHALL apply only to candidate item labels during interactive rendering and SHALL NOT change candidate sourcing, ordering, filtering, selected candidate identity, status-row text, JSON action output, terminal cleanliness semantics, or shell replacement text.

Selected candidates SHALL use the existing selected-item highlight instead of LS_COLORS-derived styling.

#### Scenario: Enabled with DX_MENU_LS_COLORS and LS_COLORS
- **WHEN** `dx menu` enters interactive rendering with `DX_MENU_LS_COLORS=1`
- **AND** `LS_COLORS` is present
- **THEN** non-selected candidate item labels SHALL be styled according to each candidate path's LS_COLORS match

#### Scenario: Missing feature flag stays monochrome
- **WHEN** `dx menu` enters interactive rendering with `LS_COLORS` present
- **AND** `DX_MENU_LS_COLORS` is unset or not equal to `1`
- **THEN** candidate item labels SHALL use the existing monochrome rendering behavior

#### Scenario: Missing LS_COLORS stays monochrome
- **WHEN** `dx menu` enters interactive rendering with `DX_MENU_LS_COLORS=1`
- **AND** `LS_COLORS` is not present
- **THEN** candidate item labels SHALL use the existing monochrome rendering behavior

#### Scenario: Selection highlight overrides LS_COLORS
- **WHEN** LS_COLORS candidate styling is enabled
- **AND** a candidate item is selected
- **THEN** the selected candidate SHALL render with the existing selected-item highlight rather than its LS_COLORS-derived style

#### Scenario: Candidate styling does not affect action output
- **WHEN** LS_COLORS candidate styling is enabled
- **AND** the user accepts a selected candidate
- **THEN** stdout SHALL emit the same replace action schema and replacement value that would be emitted with monochrome candidate rendering

#### Scenario: Candidate styling works in multicolumn rendering
- **WHEN** LS_COLORS candidate styling is enabled
- **AND** the menu renders candidates in multicolumn layout
- **THEN** non-selected candidate cells SHALL apply the LS_COLORS-derived style for their corresponding candidate paths while preserving existing grid ordering, truncation, and navigation behavior

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

When a replacement value needs both shell quoting and an appended trailing directory slash, the trailing slash SHALL be included inside the quoted path token.

The parser SHALL continue to accept previously emitted replacements that placed an appended trailing directory slash outside the quoted path token.

For non-`paths` modes, replacements SHALL identify the selected destination without a trailing slash and MAY include shell quoting when needed.

Replacement bounds SHALL only target the active query token under the cursor and SHALL NOT modify unrelated buffer segments.

#### Scenario: Relative query preserves dot-slash style
- **WHEN** buffer is `cd b`, cwd contains `./benches`, and the selected candidate is that child directory
- **THEN** the returned replacement value MAY be `./benches/`

#### Scenario: Explicit absolute query preserves absolute replacement
- **WHEN** buffer is `cd /tmp/b`, the selected candidate is `/tmp/benches`, and `paths` mode is active
- **THEN** the returned replacement value SHALL be `/tmp/benches/`

#### Scenario: Quoted directory replacement keeps slash inside quotes
- **WHEN** the selected directory replacement requires shell quoting
- **THEN** the returned replacement value SHALL include the trailing `/` inside the quoted path token

#### Scenario: Previously emitted outside-slash quoted replacement remains parseable
- **WHEN** the shell buffer contains a previously emitted quoted replacement with the trailing `/` outside the quoted path token
- **THEN** `dx menu` SHALL parse it as the same raw query with the trailing slash preserved

#### Scenario: Replace only query token
- **WHEN** buffer is `cd pr --flag` and the selected replacement token is `./projects/`
- **THEN** replacement bounds SHALL cover only `pr`, and resulting buffer SHALL be `cd ./projects/ --flag`

#### Scenario: Preserve command prefix
- **WHEN** buffer is `up co` and user selects `/home/user/code`
- **THEN** replacement SHALL preserve `up ` prefix and update only selector token

### Requirement: Path Mode Directory Replacement Uses Trailing Slash
When `dx menu` is invoked in explicit mapped `path` mode and the selected candidate is a directory, the replace action value SHALL include a trailing `/`.

When `dx menu` is invoked in explicit mapped `path` mode and the selected candidate is a file, the replace action value SHALL NOT append a trailing `/` solely because the mode is `path`.

Existing replacement formatting rules SHALL remain unchanged for built-in `paths` mode, mapped `directory` mode, mapped `file` mode, and non-filesystem completion modes.

When a mapped `path` mode directory replacement requires shell quoting, the trailing `/` SHALL be included inside the quoted path token.

#### Scenario: Path mode directory selection appends slash
- **WHEN** `dx menu --mode path` selects a directory candidate
- **THEN** the replace action value SHALL end with `/`

#### Scenario: Path mode file selection does not append slash
- **WHEN** `dx menu --mode path` selects a file candidate
- **THEN** the replace action value SHALL NOT append `/` solely because the selection came from `path` mode

#### Scenario: Path mode quoted directory keeps slash inside quotes
- **WHEN** `dx menu --mode path` selects a directory candidate whose replacement value requires shell quoting
- **THEN** the replace action value SHALL include the trailing `/` inside the quoted path token
