## Purpose
Define expected behavior for `dx resolve` path interpretation and output.

## Requirements

### Requirement: Traditional Traversal
The system MUST resolve standard shell path indicators including absolute paths, relative paths, parent directories (`..`), and home directory (`~`) by normalizing them to absolute directory paths using native platform path semantics.

On Windows, absolute paths include drive-qualified and UNC paths. A root-relative path SHALL be resolved against the root of the current drive or UNC share. Lexical normalization SHALL preserve native path prefixes and roots and SHALL NOT traverse above them.

#### Scenario: Resolve home directory
- **WHEN** the user queries `~` or `~/folder`
- **THEN** the system MUST resolve to the absolute path of the user's home directory or the specified subfolder within it

#### Scenario: Resolve parent directory
- **WHEN** the user queries `..` or `../folder`
- **THEN** the system MUST resolve to the absolute path of the parent directory or the specified subfolder within it

#### Scenario: Resolve absolute path
- **WHEN** the user queries an existing native absolute path
- **THEN** the system MUST verify the directory exists and return the normalized absolute path

#### Scenario: Resolve relative path
- **WHEN** the user queries a relative path like `./src` or `src`
- **THEN** the system MUST resolve it relative to the current working directory and return the absolute path

#### Scenario: Resolve Windows root-relative path
- **WHEN** `dx resolve` runs on Windows from drive `C:` with query `\work\project`
- **THEN** the system MUST resolve the query under `C:\` rather than treating it as a relative path

### Requirement: Portable Filesystem Query Forms
The system SHALL interpret filesystem queries using the native platform path model. On Unix, `/` is the only query separator and `\` remains an ordinary filename character. On Windows, the system SHALL accept both `\` and `/` separators for drive-absolute, UNC, root-relative, and explicit-relative queries.

The system SHALL expand `~` and `~/...` using the platform home directory and SHALL construct paths without string-concatenating separators. The system SHALL reject Windows drive-relative queries such as `C:work` with a diagnostic rather than resolving them against implicit per-drive state.

#### Scenario: Windows drive-absolute query accepts either separator
- **WHEN** `dx resolve` runs on Windows with a query for an existing directory expressed as `C:\work\project` or `C:/work/project`
- **THEN** it SHALL resolve the directory to its normalized absolute path

#### Scenario: Unix backslash remains a filename character
- **WHEN** `dx resolve` runs on Unix and a directory name contains `\`
- **THEN** the system SHALL not split that directory name into path segments

#### Scenario: Windows drive-relative query is rejected
- **WHEN** `dx resolve` runs on Windows with query `C:work`
- **THEN** it SHALL write a diagnostic to stderr, produce no stdout path, and exit non-zero

### Requirement: Filesystem Access Error Reporting
For direct filesystem queries, the system SHALL distinguish a missing path from other filesystem access failures. A missing target SHALL retain applicable fallback behavior. Permission denied, invalid path syntax, and other non-not-found I/O failures encountered while resolving an exact query SHALL produce a diagnostic and a non-zero exit code.

Configured search roots that are unavailable SHALL not prevent independent configured roots or cwd-rooted resolution from producing a result.

#### Scenario: Direct unreadable path reports its filesystem failure
- **WHEN** a direct filesystem query reaches a path whose metadata cannot be read for a reason other than not found
- **THEN** `dx resolve` SHALL produce a diagnostic identifying the filesystem failure and exit non-zero

#### Scenario: Unavailable configured root does not block another root
- **WHEN** one configured search root cannot be read
- **AND** another configured search root contains an unambiguous matching directory
- **THEN** `dx resolve` SHALL resolve the directory from the available root

### Requirement: Step-up Aliases
The system MUST support step-up aliases using multiple dots (e.g., `...` for `../../`, `....` for `../../../`) to traverse multiple parent directories quickly.

#### Scenario: Resolve three dots
- **WHEN** the user queries `...`
- **THEN** the system MUST resolve to the absolute path two levels up from the current directory

#### Scenario: Resolve four dots
- **WHEN** the user queries `....`
- **THEN** the system MUST resolve to the absolute path three levels up from the current directory

#### Scenario: Resolve N dots
- **WHEN** the user queries a string of N dots where N > 2
- **THEN** the system MUST resolve to the absolute path (N-1) levels up from the current directory

### Requirement: Abbreviated Segments
The system MUST resolve abbreviated path segments (e.g., `f/b/b`) by matching each query segment against directory names within configured search roots. Matching MUST be segment-aware: each query segment maps to one directory segment in order.

For query segments without delimiter-aware operators, matching MUST continue to behave as a segment-start prefix match.

For query segments containing supported word delimiters (`.`, `_`, `-`) or doubled-period gap operators (`..`), matching SHALL use the delimiter-aware abbreviated-segment rules and doubled-period gap rules defined for this capability.

#### Scenario: Unambiguous prefix match
- **WHEN** the user queries an abbreviated path like `src/c/b` and only one directory tree matches
- **THEN** the system MUST resolve to the single matching path (e.g., `src/components/button`)

#### Scenario: Ambiguous prefix match
- **WHEN** the user queries an abbreviated path that matches multiple directories
- **THEN** the system MUST fail with a non-zero exit code indicating ambiguity, and MUST NOT silently pick a winner

#### Scenario: Delimiter-aware segment participates in multi-segment abbreviation
- **WHEN** the user queries an abbreviated path like `proj/p..shell/s/.sdk`
- **AND** only one directory tree matches those segment rules
- **THEN** the system MUST resolve to the single matching path

### Requirement: Delimiter-Aware Abbreviated Segment Matching
Abbreviated segment matching SHALL support delimiter-aware shortening within a single query segment.

When a query segment contains `.`, `_`, or `-`, the resolver SHALL treat those characters as significant delimiter boundaries and SHALL match the surrounding fragments in order against a candidate directory name while allowing omitted characters around each delimiter boundary.

This matching SHALL honor the existing resolver case-sensitivity setting.

#### Scenario: Hyphen-delimited fragment matches directory name
- **WHEN** a search root contains a directory named `cd-extras`
- **AND** the user queries `cd-e`
- **THEN** abbreviated matching SHALL treat the query as a match for `cd-extras`

#### Scenario: Dot-delimited fragment matches interior suffix
- **WHEN** a search root contains a directory named `Microsoft.PowerShell.SDK`
- **AND** the user queries `.sdk`
- **THEN** abbreviated matching SHALL treat the query as a match for `Microsoft.PowerShell.SDK`

#### Scenario: Underscore-delimited fragment preserves delimiter identity
- **WHEN** a search root contains directories named `foo_bar` and `foo-bar`
- **AND** the user queries `foo_bar`
- **THEN** abbreviated matching SHALL match `foo_bar`
- **AND** SHALL NOT treat `foo-bar` as the same delimiter pattern solely because the surrounding fragments match

### Requirement: Doubled-Period Gap Matching Within Abbreviated Segments
Abbreviated segment matching SHALL treat each `..` sequence inside a query segment as an in-segment gap operator.

The gap operator SHALL match zero or more characters within the candidate directory segment while preserving the left-to-right order of surrounding fragments.

If a query segment contains both `..` and single `.` characters, tokenization SHALL interpret `..` before single-dot delimiter parsing.

This gap operator SHALL apply only during abbreviated segment matching and SHALL honor the existing resolver case-sensitivity setting.

#### Scenario: Doubled periods bridge omitted interior text
- **WHEN** a search root contains a directory named `PowerShell`
- **AND** the user queries `p..shell`
- **THEN** abbreviated matching SHALL treat the query as a match for `PowerShell`

#### Scenario: Doubled periods match numeric suffix path
- **WHEN** a search root contains a directory named `System32`
- **AND** the user queries `s..32`
- **THEN** abbreviated matching SHALL treat the query as a match for `System32`

#### Scenario: Doubled periods can bridge delimiter characters
- **WHEN** a search root contains a directory named `foo-bar`
- **AND** the user queries `f..bar`
- **THEN** abbreviated matching SHALL treat the query as a match for `foo-bar`

#### Scenario: Doubled periods are parsed before single-dot delimiters
- **WHEN** a search root contains a directory named `alphaBeta.core`
- **AND** the user queries `a..b.c`
- **THEN** abbreviated matching SHALL interpret `..` as a gap operator before interpreting the remaining single `.` as a delimiter boundary

### Requirement: Fallback Roots
The system MUST support configured fallback search roots (analogous to `CD_PATH`) that are searched when a query does not match as a direct, step-up, or relative path. Both exact name matches and abbreviated segment matches SHALL be attempted against fallback roots.

#### Scenario: Match in fallback root
- **WHEN** the user queries `myproject` and it does not exist relative to the current directory
- **AND** a configured fallback root contains a directory named `myproject`
- **THEN** the system MUST resolve to the absolute path of that directory within the fallback root

#### Scenario: No match in any root
- **WHEN** the user queries a name that does not match in the current directory or any configured fallback root
- **THEN** the system MUST fail with a non-zero exit code

### Requirement: Root-Anchored Fallback for Leading Root Misses
When a query begins with a native filesystem root and direct filesystem resolution does not find an existing directory, any subsequent root-based fallback matching SHALL remain anchored at that query's filesystem root.

The system SHALL NOT reinterpret that miss relative to the current working directory or configured search roots. Queries beginning with `./`, `../`, `~`, or `~/` SHALL continue into the standard root-based fallback flow after stripping the leading traversal or home prefix from the abbreviation query.


#### Scenario: Leading slash miss stays rooted at filesystem root
- **WHEN** the user queries `/proj` and `/proj` does not exist as a direct path
- **AND** `/projects` exists under `/`
- **AND** configured search roots or cwd also contain unrelated `proj*` matches
- **THEN** fallback matching SHALL consider only the filesystem-root-anchored branch and SHALL NOT return cwd-rooted or configured-root candidates

#### Scenario: Windows drive-root miss stays on its drive
- **WHEN** `dx resolve` runs on Windows with a direct miss under `C:\`
- **AND** an abbreviated match exists below `C:\`
- **AND** cwd or configured roots contain unrelated matches
- **THEN** fallback matching SHALL consider only the `C:\`-anchored branch

#### Scenario: UNC-root miss stays on its share
- **WHEN** `dx resolve` runs on Windows with a direct miss below a UNC share root
- **THEN** fallback matching SHALL retain that UNC share root as its only root anchor

#### Scenario: Dot and home misses continue standard fallback behavior
- **WHEN** the user queries `~/proj` or `../proj` and direct filesystem resolution misses
- **THEN** the system SHALL continue with the standard root-based abbreviation and fallback-root stages for the stripped query text

### Requirement: Implicit Current Directory Root in Root-Based Resolution
Root-based resolution stages (abbreviated segment matching and fallback-root matching) SHALL include the current working directory as an implicit root by default.

This implicit cwd root SHALL participate only in root-based stages and SHALL NOT alter precedence of direct paths or step-up aliases.

#### Scenario: Abbreviation resolves using cwd when no roots configured
- **WHEN** no explicit search roots are configured and a query requires root-based abbreviation matching
- **THEN** the resolver SHALL evaluate cwd as an implicit root for abbreviation matching

#### Scenario: Direct path precedence remains unchanged
- **WHEN** a direct relative path match exists in cwd and root-based matches also exist
- **THEN** resolution SHALL still return the direct-path result before consulting root-based stages

#### Scenario: Implicit cwd root is deduplicated with configured roots
- **WHEN** configured roots already include cwd (or its normalized equivalent)
- **THEN** the effective root set SHALL contain only one cwd entry

### Requirement: Resolution Precedence
The system MUST evaluate resolution strategies in a fixed, deterministic order. The precedence MUST be:
1. Direct paths (absolute, relative, `~`, `..`)
2. Step-up aliases (multi-dot patterns)
3. Abbreviated segment matching against configured roots
4. Fallback root matching (CD_PATH-style)
5. Bookmark lookup (exact name match against persistent bookmarks)
6. Failure

The system MUST return the result from the first strategy that produces a match and MUST NOT continue to lower-precedence strategies.

Delimiter-aware abbreviated matching and doubled-period gap matching SHALL be evaluated only within the abbreviated segment and fallback-root stages. They SHALL NOT alter the precedence of direct paths or step-up aliases.

#### Scenario: Direct path takes precedence over abbreviation
- **WHEN** the user queries `src` and a subdirectory `./src` exists in the current directory
- **AND** a fallback root also contains an `src` directory
- **THEN** the system MUST resolve to `./src` (direct relative path) without consulting fallback roots

#### Scenario: Step-up alias takes precedence over abbreviation
- **WHEN** the user queries `...`
- **THEN** the system MUST resolve it as a step-up alias (two levels up) regardless of whether a directory named `...` exists in any search root

#### Scenario: Fallback root takes precedence over bookmark
- **WHEN** the user queries `proj` and a configured fallback root contains a directory named `proj`
- **AND** a bookmark named `proj` also exists
- **THEN** the system MUST resolve to the fallback root match without consulting bookmarks

#### Scenario: Bookmark resolves when no filesystem match exists
- **WHEN** the user queries `proj` and no direct, step-up, abbreviated, or fallback root match is found
- **AND** a bookmark named `proj` exists pointing to an existing directory
- **THEN** the system MUST resolve to the bookmarked path

#### Scenario: Multi-dot step-up alias is not reinterpreted as gap syntax
- **WHEN** the user queries `...`
- **THEN** the system MUST resolve it using step-up alias handling rather than delimiter-aware abbreviated matching

### Requirement: Ambiguity Handling
When multiple candidates match at the same precedence level, the system MUST fail by default rather than guessing. The system MUST support `--list` and `--json` flags to return ranked candidates instead of failing.

#### Scenario: Ambiguous match in default mode
- **WHEN** a query matches multiple directories at the same precedence level
- **AND** neither `--list` nor `--json` is specified
- **THEN** the system MUST exit with a non-zero code and output a diagnostic to stderr indicating ambiguity

#### Scenario: Ambiguous match with --list flag
- **WHEN** a query matches multiple directories at the same precedence level
- **AND** `--list` is specified
- **THEN** the system MUST output all matching candidates to stdout (one per line) and exit with code 0

#### Scenario: Ambiguous match with --json flag
- **WHEN** a query matches multiple directories at the same precedence level
- **AND** `--json` is specified
- **THEN** the system MUST output a JSON object containing the status, candidates, and reason, and exit with code 0

### Requirement: Output and Error Contracts
The `dx resolve` command MUST provide shell-consumable output with strict success and failure semantics.

#### Scenario: Successful resolution
- **WHEN** a path query is successfully resolved to exactly one directory
- **THEN** the system MUST output exactly one absolute path to stdout (no trailing newline beyond the line terminator) and exit with code 0

#### Scenario: Unsuccessful resolution
- **WHEN** a path query cannot be resolved to any directory
- **THEN** the system MUST output nothing to stdout, output a diagnostic message to stderr, and exit with a non-zero code

#### Scenario: Resolved path does not exist
- **WHEN** a query resolves syntactically (e.g., `~/nonexistent`) but the target directory does not exist on disk
- **THEN** the system MUST treat this as an unsuccessful resolution

### Requirement: Performance and Safety
The system MUST ensure low-latency responses suitable for interactive shell usage and MUST NOT cause recursion loops when invoked from shell handlers.

#### Scenario: Interactive latency
- **WHEN** a resolution query is executed against a typical configuration (fewer than 10 search roots, each with fewer than 1000 immediate children)
- **THEN** the system MUST return the result in under 50ms

#### Scenario: Recursion safety
- **WHEN** `dx resolve` is invoked from within a shell `cd` wrapper or `command_not_found` handler
- **THEN** the system MUST NOT trigger further invocations of itself (e.g., by calling `cd` internally or producing output that the shell hook would re-intercept)
