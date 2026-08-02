## ADDED Requirements

### Requirement: Bookmark Prefix Completion
When collecting completion candidates, the system MUST match bookmark names by prefix and emit each matching bookmark's target directory. Candidates MUST be ordered after filesystem-derived candidates. Bookmarks whose target is no longer a directory MUST be excluded. Prefix matching MUST honor the configured case sensitivity.

Prefix matching MUST apply only to plain queries. A query with an explicit filesystem prefix MUST keep exact-name matching, and a root-anchored query MUST NOT produce bookmark candidates at all.

Resolution MUST remain exact-match-only, so a bookmark name prefix that completes MUST NOT resolve.

#### Scenario: Prefix of a bookmark name offers its target
- **WHEN** a bookmark named `work` points to an existing directory and `dx complete paths wo` is invoked
- **THEN** the output MUST contain the `work` bookmark's target path

#### Scenario: Stale bookmarks are excluded from prefix matches
- **WHEN** a bookmark named `work` exists but its target directory has been deleted, and `dx complete paths wo` is invoked
- **THEN** the output MUST NOT contain the bookmark's target path

#### Scenario: Filesystem candidates rank before bookmarks
- **WHEN** a query matches both a directory on disk and a bookmark name prefix
- **THEN** the on-disk directory MUST appear before the bookmark target in the candidate list

#### Scenario: Explicit filesystem prefix keeps exact matching
- **WHEN** a bookmark named `work` exists and `dx complete paths ./wo` is invoked with no matching directory on disk
- **THEN** the output MUST NOT contain the bookmark's target path

#### Scenario: Root-anchored query offers no bookmarks
- **WHEN** a bookmark named `work` exists and a query is anchored to a nonexistent absolute path such as `/missing/wo`
- **THEN** the output MUST NOT contain the bookmark's target path

#### Scenario: A bookmark name prefix does not resolve
- **WHEN** a bookmark named `work` exists and `dx resolve wo` is invoked
- **THEN** resolution MUST fail rather than returning the bookmarked directory

### Requirement: Prune Operation
The `dx bookmarks prune` command MUST remove every bookmark whose target is no longer a directory. It MUST print each removed bookmark rather than deleting silently, and MUST exit with code 0 when nothing is stale. The `--json` flag MUST output the removed bookmarks in the same array form as the list operation. Pruning MUST NOT occur automatically during read or write operations.

#### Scenario: Prune removes stale bookmarks
- **WHEN** a bookmark's target directory has been deleted and the user runs `dx bookmarks prune`
- **THEN** the system MUST remove that bookmark from the store and print it to stdout

#### Scenario: Prune retains live bookmarks
- **WHEN** the store contains both a live and a stale bookmark and the user runs `dx bookmarks prune`
- **THEN** the live bookmark MUST remain in the store and MUST NOT be printed

#### Scenario: Prune with nothing stale
- **WHEN** every bookmark target exists and the user runs `dx bookmarks prune`
- **THEN** the system MUST produce no output to stdout and exit with code 0

#### Scenario: Prune with --json flag
- **WHEN** the user runs `dx bookmarks prune --json` and a stale bookmark is removed
- **THEN** the system MUST output a JSON array whose entries contain `name`, `path`, and `exists` keys

### Requirement: Bookmark Store Read Resilience
Resolution and completion MUST read the bookmark store at most once per invocation. An unreadable or malformed store MUST yield no bookmarks without emitting a diagnostic, so that ordinary completion is never interrupted. The `dx bookmarks` commands MUST still report a parse failure with the offending path.

#### Scenario: Corrupt store during completion
- **WHEN** the bookmark store file contains malformed TOML and a completion query is collected
- **THEN** the system MUST return no bookmark candidates, MUST NOT emit a diagnostic, and MUST NOT fail the query

#### Scenario: Corrupt store during an explicit bookmark command
- **WHEN** the bookmark store file contains malformed TOML and the user runs `dx bookmarks`
- **THEN** the system MUST output a diagnostic naming the store file to stderr and exit with a non-zero code

## MODIFIED Requirements

### Requirement: List Operation
The `dx bookmarks` command MUST list all saved bookmarks. Default output MUST print one `name = path` line per entry to stdout, sorted alphabetically by name. A bookmark whose target is no longer a directory MUST be marked with a trailing ` (missing)` on its line. The `--json` flag MUST output a JSON array of objects, each containing a `name` string, a `path` string, and an `exists` boolean reporting whether the target is still a directory.

#### Scenario: List bookmarks in default mode
- **WHEN** the user runs `dx bookmarks` and bookmarks exist
- **THEN** the system MUST output one line per bookmark in `name = path` format, sorted alphabetically by name, to stdout

#### Scenario: List marks a stale bookmark
- **WHEN** the user runs `dx bookmarks` and a bookmark's target directory has been deleted
- **THEN** that bookmark's line MUST end with ` (missing)`

#### Scenario: List bookmarks with --json flag
- **WHEN** the user runs `dx bookmarks --json`
- **THEN** the system MUST output a JSON array whose entries each contain `name`, `path`, and `exists` keys

#### Scenario: List empty bookmark set
- **WHEN** the user runs `dx bookmarks` and no bookmarks exist
- **THEN** the system MUST produce no output to stdout and exit with code 0

### Requirement: Output Contract
All bookmark CLI commands MUST follow the dx output contract: success outputs result to stdout and exits with code 0; failure outputs diagnostic to stderr and exits with a non-zero code.

A successful `dx bookmarks add` MUST print the canonical absolute path that was stored, so that path canonicalization — which resolves symlinks — is visible. A successful `dx bookmarks remove` MUST print the absolute path of the bookmark that was removed.

#### Scenario: Add success output
- **WHEN** `dx bookmarks add proj` succeeds
- **THEN** the system MUST print the stored canonical absolute path to stdout and exit with code 0

#### Scenario: Add reports the target behind a symlink
- **WHEN** `dx bookmarks add proj <symlink>` succeeds and the given path is a symlink to a directory
- **THEN** the printed path MUST be the canonical target rather than the symlink

#### Scenario: Remove success output
- **WHEN** `dx bookmarks remove proj` succeeds
- **THEN** the system MUST print the removed bookmark's absolute path to stdout and exit with code 0

#### Scenario: Remove failure output
- **WHEN** `dx bookmarks remove nonexistent` fails because the bookmark does not exist
- **THEN** the system MUST output a diagnostic message to stderr and exit with a non-zero code
