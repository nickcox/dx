## ADDED Requirements

### Requirement: Stack Clear Command
The system SHALL provide `dx stack --clear` to remove stack entries for a session.

#### Scenario: Clear command is available
- **WHEN** a user runs `dx stack --help`
- **THEN** help output SHALL include usage for `dx stack --clear`

### Requirement: Scoped Clear Behavior
`dx stack --clear` SHALL support clearing `undo`, `redo`, or both stacks.

If no scope is provided, the command SHALL clear both stacks.

#### Scenario: Clear undo only
- **WHEN** session state is `{ "cwd": "/a", "undo": ["/b"], "redo": ["/c"] }` and `dx stack --clear --direction undo` is invoked
- **THEN** resulting state SHALL be `{ "cwd": "/a", "undo": [], "redo": ["/c"] }`

#### Scenario: Clear redo only
- **WHEN** session state is `{ "cwd": "/a", "undo": ["/b"], "redo": ["/c"] }` and `dx stack --clear --direction redo` is invoked
- **THEN** resulting state SHALL be `{ "cwd": "/a", "undo": ["/b"], "redo": [] }`

#### Scenario: Clear both by default
- **WHEN** session state is `{ "cwd": "/a", "undo": ["/b"], "redo": ["/c"] }` and `dx stack --clear` is invoked
- **THEN** resulting state SHALL be `{ "cwd": "/a", "undo": [], "redo": [] }`

### Requirement: Preserve Current Directory
`dx stack --clear` SHALL preserve the current `cwd` value in the session file.

#### Scenario: Cwd remains unchanged after clear
- **WHEN** `dx stack --clear` is invoked on a session with `cwd` set
- **THEN** `cwd` SHALL remain unchanged in persisted session state

### Requirement: Success on Already-Empty Stacks
`dx stack --clear` SHALL be idempotent and succeed even when targeted stacks are already empty.

#### Scenario: Clear empty undo stack
- **WHEN** session state has `undo` as an empty array and `dx stack --clear --direction undo` is invoked
- **THEN** command SHALL exit with code 0 and session state SHALL remain valid

### Requirement: Clear Output Contract
On success, `dx stack --clear` SHALL print nothing to stdout and exit with code 0.
On failure, it SHALL print a diagnostic to stderr and exit with a non-zero code.

#### Scenario: Successful clear output
- **WHEN** `dx stack --clear` succeeds
- **THEN** stdout SHALL be empty and exit code SHALL be 0

#### Scenario: Failed clear output
- **WHEN** `dx stack --clear` fails due to missing session identity
- **THEN** stdout SHALL be empty, stderr SHALL contain a diagnostic, and exit code SHALL be non-zero
