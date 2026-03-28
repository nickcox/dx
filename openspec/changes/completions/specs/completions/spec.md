## ADDED Requirements

### Requirement: Complete Subcommand Tree
The `dx complete` subcommand SHALL expose mode-specific subcommands: `paths`, `ancestors`, `frecents`, `recents`, and `stack`. Each subcommand SHALL accept an optional positional `query` argument and an optional `--json` flag. The `stack` subcommand SHALL additionally require a `--direction` flag with value `back` or `forward`.

Invoking `dx complete` without a mode subcommand SHALL print usage help and exit with a non-zero exit code.

#### Scenario: List available modes
- **WHEN** `dx complete` is invoked without a subcommand
- **THEN** the command SHALL print usage help listing the available modes and exit non-zero

#### Scenario: Invalid mode
- **WHEN** `dx complete bogus` is invoked
- **THEN** the command SHALL print an error diagnostic and exit non-zero

#### Scenario: Stack mode requires direction
- **WHEN** `dx complete stack` is invoked without `--direction`
- **THEN** the command SHALL print an error diagnostic and exit non-zero

### Requirement: Paths Mode
`dx complete paths <query>` SHALL return directory candidates by running the query through the resolver's candidate-collection strategy. This SHALL include abbreviated segment matches, fallback root matches, and bookmark name matches — the same sources as `dx resolve`, but collecting all candidates instead of failing on ambiguity.

Candidates SHALL be deduplicated by absolute path and ordered by resolution precedence (direct > step-up > abbreviated > fallback > bookmark).

#### Scenario: Abbreviated query returns multiple candidates
- **WHEN** `dx complete paths pr` is invoked and abbreviation expansion finds `/home/user/projects` and `/home/user/presentations`
- **THEN** the output SHALL contain both paths, one per line

#### Scenario: Bookmark name matches
- **WHEN** `dx complete paths work` is invoked and a bookmark named `work` exists pointing to `/home/user/work`
- **THEN** `/home/user/work` SHALL appear in the output

#### Scenario: No matches
- **WHEN** `dx complete paths zzz` is invoked and no candidates are found
- **THEN** the command SHALL produce no output (empty stdout) and exit with code 0

### Requirement: Ancestors Mode
`dx complete ancestors [query]` SHALL return the parent directories from the current working directory up to the filesystem root, ordered nearest-first. If a query is provided, candidates SHALL be filtered using path-aware matching (see Query Filtering requirement).

#### Scenario: Full ancestor list from nested directory
- **WHEN** `dx complete ancestors` is invoked from `/home/user/code/projects/dx`
- **THEN** the output SHALL contain, in order: `/home/user/code/projects`, `/home/user/code`, `/home/user`, `/home`, `/`

#### Scenario: Filtered ancestor list
- **WHEN** `dx complete ancestors code` is invoked from `/home/user/code/projects/dx`
- **THEN** the output SHALL contain `/home/user/code` (basename matches) and exclude non-matching ancestors

#### Scenario: Root directory has no ancestors
- **WHEN** `dx complete ancestors` is invoked from `/`
- **THEN** the command SHALL produce no output and exit with code 0

### Requirement: Frecents Mode
`dx complete frecents [query]` SHALL return frecency-ranked directory candidates by querying the configured `FrecencyProvider`. If a query is provided, it SHALL be passed to the provider as a filter argument.

#### Scenario: Zoxide provides frecency candidates
- **WHEN** `dx complete frecents proj` is invoked and zoxide is installed with matching entries
- **THEN** the output SHALL contain the paths returned by `zoxide query --list proj`, one per line

#### Scenario: Zoxide not installed
- **WHEN** `dx complete frecents` is invoked and zoxide is not installed
- **THEN** the command SHALL produce no output and exit with code 0

#### Scenario: Zoxide returns no matches
- **WHEN** `dx complete frecents nonexistent` is invoked and zoxide returns no matches
- **THEN** the command SHALL produce no output and exit with code 0

### Requirement: Recents Mode
`dx complete recents [query]` SHALL return recently visited directories from the current session's stack undo history, ordered most-recent-first. If a query is provided, candidates SHALL be filtered using path-aware matching. The session SHALL be identified by `--session` flag or `DX_SESSION` environment variable.

#### Scenario: Session with navigation history
- **WHEN** `dx complete recents` is invoked and the session stack has undo entries `[/a, /b, /c]` (most recent last in storage)
- **THEN** the output SHALL contain `/c`, `/b`, `/a` in that order (most recent first)

#### Scenario: Empty session history
- **WHEN** `dx complete recents` is invoked and the session stack has no undo entries
- **THEN** the command SHALL produce no output and exit with code 0

#### Scenario: Filtered recents
- **WHEN** `dx complete recents proj` is invoked and the session stack contains `/home/user/projects/dx` and `/tmp/scratch`
- **THEN** the output SHALL contain only `/home/user/projects/dx`

#### Scenario: Missing session identity
- **WHEN** `dx complete recents` is invoked without `--session` and `DX_SESSION` is unset
- **THEN** the command SHALL produce no output and exit with code 0

### Requirement: Stack Mode
`dx complete stack --direction back|forward [query]` SHALL return directory candidates from the session stack's undo entries (when direction is `back`) or redo entries (when direction is `forward`), ordered by stack proximity (top of stack first). If a query is provided, candidates SHALL be filtered using path-aware matching.

#### Scenario: Back direction returns undo entries
- **WHEN** `dx complete stack --direction back` is invoked and the session stack has undo entries `[/a, /b, /c]`
- **THEN** the output SHALL contain `/c`, `/b`, `/a` in that order (stack top first)

#### Scenario: Forward direction returns redo entries
- **WHEN** `dx complete stack --direction forward` is invoked and the session stack has redo entries `[/x, /y]`
- **THEN** the output SHALL contain `/y`, `/x` in that order (stack top first)

#### Scenario: Empty stack direction
- **WHEN** `dx complete stack --direction back` is invoked and the session stack has no undo entries
- **THEN** the command SHALL produce no output and exit with code 0

#### Scenario: Filtered stack candidates
- **WHEN** `dx complete stack --direction back proj` is invoked and the undo stack contains `/home/user/projects/dx` and `/tmp/scratch`
- **THEN** the output SHALL contain only `/home/user/projects/dx`

### Requirement: Plain Output Format
In plain mode (no `--json` flag), all completion modes SHALL output one absolute path per line to stdout. No indexes, labels, or metadata SHALL be included. Empty results SHALL produce no output. The command SHALL always exit with code 0.

#### Scenario: Multiple candidates in plain mode
- **WHEN** `dx complete ancestors` is invoked without `--json` from `/a/b/c`
- **THEN** stdout SHALL contain `/a/b` and `/a` and `/` each on separate lines, and the exit code SHALL be 0

#### Scenario: Empty results in plain mode
- **WHEN** a completion mode produces no candidates and `--json` is not specified
- **THEN** stdout SHALL be empty and the exit code SHALL be 0

### Requirement: JSON Output Format
When `--json` is specified, all completion modes SHALL output a JSON array of candidate objects to stdout. Each object SHALL contain a `path` field (absolute path string), a `label` field (human-readable display string), and a `rank` field (1-based integer reflecting position in the candidate list). Empty results SHALL produce `[]`. The command SHALL always exit with code 0.

No `mode` or `direction` fields SHALL be included in the JSON output; the caller already knows its invocation context.

#### Scenario: JSON output with candidates
- **WHEN** `dx complete ancestors --json` is invoked from `/a/b/c`
- **THEN** stdout SHALL contain a JSON array where each element has `path`, `label`, and `rank` fields, and `rank` values are sequential starting from 1

#### Scenario: JSON label for ancestors
- **WHEN** `dx complete ancestors --json` is invoked from `/home/user/code`
- **THEN** the `label` for `/home/user` SHALL be a human-readable representation (e.g., `user` or `home/user`)

#### Scenario: Empty JSON results
- **WHEN** a completion mode produces no candidates and `--json` is specified
- **THEN** stdout SHALL contain `[]` and the exit code SHALL be 0

### Requirement: FrecencyProvider Abstraction
The system SHALL define a `FrecencyProvider` trait with methods `query(filter) -> Vec<PathBuf>` and `is_available() -> bool`. The initial implementation SHALL be `ZoxideProvider`, which invokes `zoxide query --list [filter]` via `std::process::Command` and parses one-path-per-line output.

`ZoxideProvider::is_available()` SHALL check for the `zoxide` binary on the system PATH. If the binary is not found, `query()` SHALL return an empty vector without producing any error output.

No new crate dependencies SHALL be added for frecency functionality.

#### Scenario: ZoxideProvider queries zoxide
- **WHEN** `ZoxideProvider::query("proj")` is called and zoxide is installed
- **THEN** it SHALL execute `zoxide query --list proj` and return the paths from stdout

#### Scenario: ZoxideProvider when zoxide is absent
- **WHEN** `ZoxideProvider::is_available()` is called and `zoxide` is not on PATH
- **THEN** it SHALL return `false`

#### Scenario: ZoxideProvider query when unavailable
- **WHEN** `ZoxideProvider::query("anything")` is called and zoxide is not installed
- **THEN** it SHALL return an empty vector without writing to stderr

### Requirement: Query Filtering
When a query argument is provided to `ancestors`, `recents`, or `stack` modes, candidates SHALL be filtered using path-aware matching with the following priority:

1. Exact absolute path match
2. Exact basename match
3. Absolute-path prefix match
4. Basename prefix match
5. Substring match (against full path)

Matching SHALL be case-insensitive. All candidates that match at any level SHALL be included in the output. Within a match level, the mode's native ordering SHALL be preserved (nearest-first for ancestors, most-recent-first for recents/stack).

The `frecents` mode SHALL pass the query directly to the FrecencyProvider without applying this filtering. The `paths` mode SHALL pass the query to the resolver as abbreviation input.

#### Scenario: Exact basename match ranks first
- **WHEN** `dx complete ancestors code` is invoked from `/home/user/code/projects/dx` and both `/home/user/code` and `/home/user/code-review` are ancestors
- **THEN** `/home/user/code` (exact basename match) SHALL appear before `/home/user/code-review` (prefix match)

#### Scenario: Case-insensitive matching
- **WHEN** `dx complete recents Proj` is invoked and the session contains `/home/user/projects/dx`
- **THEN** `/home/user/projects/dx` SHALL be included (case-insensitive substring match on `projects`)

#### Scenario: No filter matches
- **WHEN** `dx complete ancestors zzz` is invoked and no ancestor basename or path contains `zzz`
- **THEN** the command SHALL produce no output and exit with code 0

### Requirement: Navigation Selector Resolution
The system SHALL provide a mechanism to resolve a navigation selector (used by shell wrappers for `up`, `back`, `forward`) to a single target path. The selector SHALL be interpreted as follows:

1. Empty/absent selector: return the first candidate from the corresponding completion mode.
2. Integer selector (`N`): return the Nth candidate (1-based) from the mode's candidate list.
3. Non-integer selector: treat as a path query and return the best match using the query filtering rules, selecting only the top result.

If the selector does not match any candidate, the system SHALL print a diagnostic to stderr and exit with a non-zero exit code.

#### Scenario: No selector returns first candidate
- **WHEN** a navigation command is invoked with no selector and the mode has candidates
- **THEN** the system SHALL return the first candidate (e.g., immediate parent for `up`, stack top for `back`)

#### Scenario: Numeric selector picks Nth candidate
- **WHEN** a navigation command is invoked with selector `3` and the mode has at least 3 candidates
- **THEN** the system SHALL return the 3rd candidate from the mode's list

#### Scenario: Numeric selector out of range
- **WHEN** a navigation command is invoked with selector `99` and the mode has fewer than 99 candidates
- **THEN** the system SHALL print a diagnostic to stderr and exit non-zero

#### Scenario: Path selector matches closest candidate
- **WHEN** `up` is invoked with selector `code` from `/home/user/code/projects/dx`
- **THEN** the system SHALL return `/home/user/code` (exact basename match among ancestors)

#### Scenario: Path selector with no match
- **WHEN** `back` is invoked with selector `nonexistent` and no stack entry matches
- **THEN** the system SHALL print a diagnostic to stderr and exit non-zero
