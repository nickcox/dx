## ADDED Requirements

### Requirement: Stack List Command
The system SHALL provide `dx stack --list` to inspect session stack entries without mutating stack state.

#### Scenario: List command is available
- **WHEN** a user runs `dx stack --help`
- **THEN** help output SHALL include usage for `dx stack --list`

### Requirement: Direction Selection
`dx stack --list` SHALL support selecting which stack entries to list: undo, redo, or both.

If `--direction` is omitted, the command SHALL default to `both`.

#### Scenario: List undo only
- **WHEN** the session state has `undo` entries and `dx stack --list --direction undo` is invoked
- **THEN** output SHALL include only undo entries

#### Scenario: List redo only
- **WHEN** the session state has `redo` entries and `dx stack --list --direction redo` is invoked
- **THEN** output SHALL include only redo entries

#### Scenario: List both stacks
- **WHEN** the session state has both undo and redo entries and `dx stack --list --direction both` is invoked
- **THEN** output SHALL include entries from both stacks

#### Scenario: Direction omitted defaults to both
- **WHEN** the session state has both undo and redo entries and `dx stack --list` is invoked without `--direction`
- **THEN** output SHALL be equivalent to `dx stack --list --direction both`

### Requirement: Plain Output Contract
In plain mode, `dx stack --list` SHALL print one absolute path per line in deterministic order and exit with code 0.

For `undo`, entries SHALL be listed nearest-first (most recent undo target first, corresponding to the last element of stored `undo`). For `redo`, entries SHALL be listed nearest-first (most recent redo target first, corresponding to the last element of stored `redo`).

For `both`, output SHALL list all undo entries first (nearest-first) followed by all redo entries (nearest-first).

#### Scenario: Plain output with undo entries
- **WHEN** session state is `{ "cwd": "/a", "undo": ["/b", "/c"], "redo": [] }` and `dx stack --list --direction undo` is invoked
- **THEN** stdout SHALL contain exactly `/c\n/b\n` and exit code SHALL be 0

#### Scenario: Plain output with empty stack
- **WHEN** `dx stack --list --direction undo` is invoked and `undo` is empty
- **THEN** stdout SHALL be empty and exit code SHALL be 0

#### Scenario: Plain output ordering with both directions
- **WHEN** session state is `{ "cwd": "/x", "undo": ["/a", "/b"], "redo": ["/c", "/d"] }` and `dx stack --list --direction both` is invoked
- **THEN** stdout SHALL contain exactly `/b\n/a\n/d\n/c\n` and exit code SHALL be 0

### Requirement: JSON Output Contract
`dx stack --list --json` SHALL emit valid JSON containing stack entries and metadata sufficient for machine parsing.

Each entry SHALL include at least:
- `path`: absolute path string
- `label`: display label derived from the path
- `rank`: 1-based index in displayed order

#### Scenario: JSON output includes rank
- **WHEN** session state is `{ "cwd": "/a", "undo": ["/b", "/c"], "redo": [] }` and `dx stack --list --direction undo --json` is invoked
- **THEN** stdout SHALL be valid JSON with first entry `{ "path": "/c", "label": "c", "rank": 1 }`

#### Scenario: JSON output for empty result
- **WHEN** no entries match the selected direction and `dx stack --list --json` is invoked
- **THEN** stdout SHALL be a valid JSON representation of an empty result and exit code SHALL be 0

### Requirement: Read-Only Behavior
`dx stack --list` SHALL NOT modify session state.

#### Scenario: Listing does not mutate session
- **WHEN** `dx stack --list` is invoked for a session with existing stacks
- **THEN** the session file contents SHALL remain byte-for-byte equivalent after command completion
