## Purpose
Define expected behavior for session-scoped directory stack operations (`push`, `pop`, `undo`, `redo`) and persistence.

## Requirements

### Requirement: Session Directory Resolution
The system SHALL resolve the session storage directory using the following precedence:
1. `$XDG_RUNTIME_DIR/dx-sessions/` if `XDG_RUNTIME_DIR` is set and non-empty.
2. Otherwise, `std::env::temp_dir().join("dx-sessions")`.

The system SHALL create the `dx-sessions/` directory (and parents) on first use if it does not exist.

#### Scenario: XDG runtime dir is set
- **WHEN** `XDG_RUNTIME_DIR` is set to `/run/user/1000`
- **THEN** the session directory SHALL be `/run/user/1000/dx-sessions/`

#### Scenario: XDG runtime dir is not set
- **WHEN** `XDG_RUNTIME_DIR` is unset or empty
- **THEN** the session directory SHALL be `<temp_dir>/dx-sessions/` where `<temp_dir>` is the value of `std::env::temp_dir()`

#### Scenario: Session directory does not exist yet
- **WHEN** a stack command is invoked and the session directory does not exist
- **THEN** the system SHALL create the directory (including parents) before proceeding

### Requirement: Session Identification
The system SHALL identify sessions by a session ID passed via the `--session <id>` CLI flag or the `DX_SESSION` environment variable. The CLI flag takes precedence over the environment variable.

The system SHALL NOT attempt to auto-detect the parent shell PID.

If neither `--session` nor `DX_SESSION` is provided, the command SHALL fail with a non-zero exit code and a diagnostic on stderr.

#### Scenario: Session ID from CLI flag
- **WHEN** `dx push /foo --session 12345` is invoked
- **THEN** the system SHALL operate on session file `12345.json`

#### Scenario: Session ID from environment variable
- **WHEN** `DX_SESSION=12345` is set and `dx undo` is invoked without `--session`
- **THEN** the system SHALL operate on session file `12345.json`

#### Scenario: CLI flag overrides environment variable
- **WHEN** `DX_SESSION=11111` is set and `dx undo --session 22222` is invoked
- **THEN** the system SHALL operate on session file `22222.json`

#### Scenario: No session ID provided
- **WHEN** `dx push /foo` is invoked with no `--session` flag and `DX_SESSION` is unset
- **THEN** the command SHALL exit with a non-zero code and print a diagnostic to stderr

### Requirement: Session File Schema
Each session file SHALL be a JSON object with the following structure:
```json
{
  "cwd": "<absolute-path>",
  "undo": ["<path>", ...],
  "redo": ["<path>", ...]
}
```

- `cwd`: The current working directory tracked by this session.
- `undo`: A stack of previous directories (last element is the most recent). Empty array when no history exists.
- `redo`: A stack of directories available for redo (last element is the most recent). Empty array when no redo history exists.

#### Scenario: New session with no prior state
- **WHEN** a stack command is invoked for a session ID that has no existing file
- **THEN** the system SHALL treat the session as having empty `undo` and `redo` stacks and no `cwd`

#### Scenario: Corrupt or unparseable session file
- **WHEN** a session file exists but contains invalid JSON
- **THEN** the system SHALL treat the session as empty (as if no file existed) and overwrite it on next write

### Requirement: Push Operation
`dx push <path>` SHALL record a new directory in the session:
1. If the session has an existing `cwd`, push it onto the `undo` stack.
2. Set `cwd` to the provided `<path>`.
3. Clear the `redo` stack (new navigation branch).
4. Print `<path>` to stdout.
5. Exit with code 0.

If `<path>` equals the current `cwd`, the push SHALL be a no-op (no duplicate entry, redo preserved) and still print the path and exit 0.

#### Scenario: Push onto empty session
- **WHEN** `dx push /home/user --session 100` is invoked with no existing session file
- **THEN** session state SHALL be `{ "cwd": "/home/user", "undo": [], "redo": [] }` and stdout SHALL contain `/home/user`

#### Scenario: Push with existing history
- **WHEN** session state is `{ "cwd": "/a", "undo": ["/b"], "redo": ["/c"] }` and `dx push /d --session 100` is invoked
- **THEN** session state SHALL be `{ "cwd": "/d", "undo": ["/b", "/a"], "redo": [] }` and stdout SHALL contain `/d`

#### Scenario: Push duplicate of current directory
- **WHEN** session state is `{ "cwd": "/a", "undo": ["/b"], "redo": ["/c"] }` and `dx push /a --session 100` is invoked
- **THEN** session state SHALL remain `{ "cwd": "/a", "undo": ["/b"], "redo": ["/c"] }` and stdout SHALL contain `/a`

### Requirement: Pop Operation
`dx pop` SHALL destructively remove the top of the undo stack:
1. If the `undo` stack is empty, fail with a non-zero exit code and print a diagnostic to stderr.
2. Otherwise, pop the top entry from `undo` and set it as `cwd`.
3. The `redo` stack is NOT modified.
4. Print the new `cwd` to stdout.
5. Exit with code 0.

#### Scenario: Pop with entries on undo stack
- **WHEN** session state is `{ "cwd": "/a", "undo": ["/b", "/c"], "redo": ["/d"] }` and `dx pop --session 100` is invoked
- **THEN** session state SHALL be `{ "cwd": "/c", "undo": ["/b"], "redo": ["/d"] }` and stdout SHALL contain `/c`

#### Scenario: Pop with empty undo stack
- **WHEN** session state is `{ "cwd": "/a", "undo": [], "redo": [] }` and `dx pop --session 100` is invoked
- **THEN** the command SHALL exit with a non-zero code and print a diagnostic to stderr, and session state SHALL be unchanged

### Requirement: Undo Operation
`dx undo` SHALL non-destructively navigate backward:
1. If the `undo` stack is empty, fail with a non-zero exit code and print a diagnostic to stderr.
2. Otherwise, push the current `cwd` onto the `redo` stack.
3. Pop the top entry from `undo` and set it as `cwd`.
4. Print the new `cwd` to stdout.
5. Exit with code 0.

#### Scenario: Undo with history
- **WHEN** session state is `{ "cwd": "/a", "undo": ["/b", "/c"], "redo": [] }` and `dx undo --session 100` is invoked
- **THEN** session state SHALL be `{ "cwd": "/c", "undo": ["/b"], "redo": ["/a"] }` and stdout SHALL contain `/c`

#### Scenario: Multiple consecutive undos
- **WHEN** session state is `{ "cwd": "/a", "undo": ["/b", "/c"], "redo": [] }` and `dx undo` is invoked twice
- **THEN** after first undo: `{ "cwd": "/c", "undo": ["/b"], "redo": ["/a"] }` and after second undo: `{ "cwd": "/b", "undo": [], "redo": ["/a", "/c"] }`

#### Scenario: Undo with empty undo stack
- **WHEN** session state is `{ "cwd": "/a", "undo": [], "redo": ["/b"] }` and `dx undo --session 100` is invoked
- **THEN** the command SHALL exit with a non-zero code and print a diagnostic to stderr, and session state SHALL be unchanged

### Requirement: Redo Operation
`dx redo` SHALL non-destructively navigate forward:
1. If the `redo` stack is empty, fail with a non-zero exit code and print a diagnostic to stderr.
2. Otherwise, push the current `cwd` onto the `undo` stack.
3. Pop the top entry from `redo` and set it as `cwd`.
4. Print the new `cwd` to stdout.
5. Exit with code 0.

#### Scenario: Redo after undo
- **WHEN** session state is `{ "cwd": "/c", "undo": ["/b"], "redo": ["/a"] }` and `dx redo --session 100` is invoked
- **THEN** session state SHALL be `{ "cwd": "/a", "undo": ["/b", "/c"], "redo": [] }` and stdout SHALL contain `/a`

#### Scenario: Redo with empty redo stack
- **WHEN** session state is `{ "cwd": "/a", "undo": ["/b"], "redo": [] }` and `dx redo --session 100` is invoked
- **THEN** the command SHALL exit with a non-zero code and print a diagnostic to stderr, and session state SHALL be unchanged

### Requirement: Redo Cleared on Push
When `dx push` records a new directory (that is not a duplicate of `cwd`), the `redo` stack SHALL be cleared. This enforces standard undo/redo branch semantics - navigating to a new location discards the forward history.

#### Scenario: Push after undo clears redo
- **WHEN** session state is `{ "cwd": "/c", "undo": ["/b"], "redo": ["/a"] }` and `dx push /d --session 100` is invoked
- **THEN** session state SHALL be `{ "cwd": "/d", "undo": ["/b", "/c"], "redo": [] }`

### Requirement: Atomic File Writes
The system SHALL write session state using a write-to-temp-then-rename pattern:
1. Write the new JSON to a temporary file in the same directory as the target session file.
2. Rename the temporary file over the target session file.

This ensures readers never observe a partially-written file.

#### Scenario: Crash during write
- **WHEN** the process is interrupted after writing the temp file but before renaming
- **THEN** the original session file SHALL remain intact and the orphaned temp file SHALL not affect subsequent operations

#### Scenario: Normal write succeeds
- **WHEN** any stack command modifies session state
- **THEN** the session file SHALL contain valid, complete JSON after the operation

### Requirement: Output Contract
All stack subcommands (`push`, `pop`, `undo`, `redo`) SHALL follow a consistent output contract:
- **On success**: Print exactly one absolute path to stdout, followed by a newline. Exit with code 0.
- **On failure**: Print nothing to stdout. Print a human-readable diagnostic to stderr. Exit with a non-zero code.

#### Scenario: Successful operation output
- **WHEN** `dx undo --session 100` succeeds and the restored path is `/home/user`
- **THEN** stdout SHALL contain exactly `/home/user\n` and the exit code SHALL be 0

#### Scenario: Failed operation output
- **WHEN** `dx undo --session 100` fails because the undo stack is empty
- **THEN** stdout SHALL be empty, stderr SHALL contain a diagnostic message, and the exit code SHALL be non-zero

### Requirement: Stale Session Cleanup
The system SHALL perform best-effort cleanup of stale session files:
1. On each session file read or write, optionally scan sibling `*.json` files in the session directory.
2. Remove files whose modification time is older than a configured TTL (default: 7 days).
3. Only delete files matching the expected session filename pattern.

Cleanup SHALL be best effort:
- Errors reading metadata or deleting files SHALL be silently ignored.
- The active command SHALL never fail due to cleanup errors.

#### Scenario: Stale files are pruned
- **WHEN** a stack command runs and the session directory contains a session file last modified 10 days ago
- **THEN** the stale file SHALL be deleted (best effort) and the active command SHALL succeed normally

#### Scenario: Cleanup errors are ignored
- **WHEN** a stale file cannot be deleted (e.g., permission denied)
- **THEN** the error SHALL be silently ignored and the active command SHALL succeed normally

#### Scenario: Only session files are pruned
- **WHEN** the session directory contains non-session files (e.g., `.lock`, `.tmp`)
- **THEN** those files SHALL NOT be deleted by the cleanup process
