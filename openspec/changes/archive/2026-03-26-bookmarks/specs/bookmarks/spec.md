## Purpose

Define expected behavior for persistent named directory bookmarks — creating, removing, listing, resolving, and storing named path aliases.

## ADDED Requirements

### Requirement: Bookmark Store Location
The system MUST store bookmarks in a TOML file resolved in the following order: (1) `DX_BOOKMARKS_FILE` environment variable if set, (2) `$XDG_DATA_HOME/dx/bookmarks.toml`, (3) platform-specific data directory via `dirs::data_dir()` joined with `dx/bookmarks.toml`. If the file does not exist, read operations MUST return an empty bookmark set without error.

#### Scenario: DX_BOOKMARKS_FILE override
- **WHEN** the `DX_BOOKMARKS_FILE` environment variable is set to a path
- **THEN** the system MUST use that path as the bookmark store file, ignoring XDG and platform defaults

#### Scenario: XDG_DATA_HOME resolution
- **WHEN** `DX_BOOKMARKS_FILE` is not set and `XDG_DATA_HOME` is set
- **THEN** the system MUST use `$XDG_DATA_HOME/dx/bookmarks.toml` as the bookmark store file

#### Scenario: Platform default fallback
- **WHEN** neither `DX_BOOKMARKS_FILE` nor `XDG_DATA_HOME` is set
- **THEN** the system MUST use the platform data directory (e.g., `~/.local/share/dx/bookmarks.toml`) as the bookmark store file

#### Scenario: Missing store file on read
- **WHEN** the bookmark store file does not exist and a read operation is performed
- **THEN** the system MUST return an empty bookmark set and MUST NOT produce an error

### Requirement: Bookmark Name Validation
The system MUST validate that bookmark names are non-empty and contain only alphanumeric characters, hyphens, and underscores (matching `^[a-zA-Z0-9_-]+$`). Names containing path-like characters (`/`, `.`, `~`, whitespace) MUST be rejected.

#### Scenario: Valid bookmark name
- **WHEN** a user provides a bookmark name like `my-project` or `docs_v2`
- **THEN** the system MUST accept the name

#### Scenario: Invalid bookmark name with path characters
- **WHEN** a user provides a bookmark name containing `/`, `.`, `~`, or whitespace
- **THEN** the system MUST reject the name with a diagnostic message to stderr and exit with a non-zero code

#### Scenario: Empty bookmark name
- **WHEN** a user provides an empty string as a bookmark name
- **THEN** the system MUST reject it with a diagnostic message to stderr and exit with a non-zero code

### Requirement: Mark Operation
The `dx mark <name> [<path>]` command MUST save a bookmark associating the given name with an absolute directory path. If `<path>` is omitted, the current working directory MUST be used. The path MUST be canonicalized to an absolute path before storing. The parent directory of the store file MUST be created automatically if it does not exist.

#### Scenario: Mark current directory
- **WHEN** the user runs `dx mark proj` without specifying a path
- **THEN** the system MUST save a bookmark named `proj` pointing to the current working directory

#### Scenario: Mark explicit path
- **WHEN** the user runs `dx mark proj /home/user/code/project`
- **THEN** the system MUST save a bookmark named `proj` pointing to `/home/user/code/project`

#### Scenario: Overwrite existing bookmark
- **WHEN** a bookmark named `proj` already exists and the user runs `dx mark proj /new/path`
- **THEN** the system MUST replace the existing path with the new one without error

#### Scenario: Auto-create store directory
- **WHEN** the parent directory of the bookmark store file does not exist
- **THEN** the system MUST create the directory hierarchy before writing the file

#### Scenario: Mark nonexistent path
- **WHEN** the user runs `dx mark proj /does/not/exist`
- **THEN** the system MUST reject the operation with a diagnostic message to stderr and exit with a non-zero code

### Requirement: Unmark Operation
The `dx unmark <name>` command MUST remove the bookmark with the given name. If the named bookmark does not exist, the command MUST fail with a diagnostic message to stderr and a non-zero exit code.

#### Scenario: Remove existing bookmark
- **WHEN** a bookmark named `proj` exists and the user runs `dx unmark proj`
- **THEN** the system MUST remove the bookmark and exit with code 0

#### Scenario: Remove nonexistent bookmark
- **WHEN** no bookmark named `proj` exists and the user runs `dx unmark proj`
- **THEN** the system MUST output a diagnostic to stderr and exit with a non-zero code

### Requirement: List Operation
The `dx bookmarks` command MUST list all saved bookmarks. Default output MUST print one `name = path` line per entry to stdout, sorted alphabetically by name. The `--json` flag MUST output a JSON object mapping names to paths.

#### Scenario: List bookmarks in default mode
- **WHEN** the user runs `dx bookmarks` and bookmarks exist
- **THEN** the system MUST output one line per bookmark in `name = path` format, sorted alphabetically by name, to stdout

#### Scenario: List bookmarks with --json flag
- **WHEN** the user runs `dx bookmarks --json`
- **THEN** the system MUST output a JSON object where keys are bookmark names and values are path strings

#### Scenario: List empty bookmark set
- **WHEN** the user runs `dx bookmarks` and no bookmarks exist
- **THEN** the system MUST produce no output to stdout and exit with code 0

### Requirement: Bookmark Resolve Lookup
When invoked during path resolution, the system MUST perform an exact-match lookup of the query against bookmark names. The bookmarked path MUST exist on disk for the lookup to succeed. If the path does not exist, the lookup MUST return no match (treating the bookmark as stale).

#### Scenario: Resolve existing bookmark
- **WHEN** `dx resolve proj` is called and a bookmark named `proj` exists pointing to an existing directory
- **AND** no filesystem-based resolution strategy matched the query
- **THEN** the system MUST return the bookmarked directory path

#### Scenario: Resolve bookmark with stale path
- **WHEN** `dx resolve proj` is called and a bookmark named `proj` exists but its target directory has been deleted
- **THEN** the bookmark lookup MUST return no match, and resolution MUST continue to failure

#### Scenario: Resolve query that is not a bookmark name
- **WHEN** `dx resolve somename` is called and no bookmark named `somename` exists
- **THEN** the bookmark lookup MUST return no match

### Requirement: Atomic File Writes
All write operations to the bookmark store MUST use atomic write semantics: write to a temporary file in the same directory, then rename over the target. This MUST prevent data loss from interrupted writes or crashes.

#### Scenario: Crash during write
- **WHEN** a write to the bookmark store is interrupted (process killed, power loss)
- **THEN** the previous version of the store file MUST remain intact

#### Scenario: Normal write completion
- **WHEN** a mark or unmark operation completes
- **THEN** the store file MUST contain the updated bookmark set and no temporary files MUST remain

### Requirement: Output Contract
All bookmark CLI commands MUST follow the dx output contract: success outputs result to stdout and exits with code 0; failure outputs diagnostic to stderr and exits with a non-zero code.

#### Scenario: Mark success output
- **WHEN** `dx mark proj` succeeds
- **THEN** the system MUST exit with code 0

#### Scenario: Unmark failure output
- **WHEN** `dx unmark nonexistent` fails because the bookmark does not exist
- **THEN** the system MUST output a diagnostic message to stderr and exit with a non-zero code
