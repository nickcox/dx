## Purpose
Define expected behavior for `dx resolve` path interpretation and output.

## Requirements

### Requirement: Traditional Traversal
The system MUST resolve standard shell path indicators including absolute paths, relative paths, parent directories (`..`), and home directory (`~`) by normalizing them to absolute directory paths.

#### Scenario: Resolve home directory
- **WHEN** the user queries `~` or `~/folder`
- **THEN** the system MUST resolve to the absolute path of the user's home directory or the specified subfolder within it

#### Scenario: Resolve parent directory
- **WHEN** the user queries `..` or `../folder`
- **THEN** the system MUST resolve to the absolute path of the parent directory or the specified subfolder within it

#### Scenario: Resolve absolute path
- **WHEN** the user queries an absolute path like `/usr/local/bin`
- **THEN** the system MUST verify the directory exists and return the normalized absolute path

#### Scenario: Resolve relative path
- **WHEN** the user queries a relative path like `./src` or `src`
- **THEN** the system MUST resolve it relative to the current working directory and return the absolute path

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
The system MUST resolve abbreviated path segments (e.g., `f/b/b`) by matching each query segment as a prefix against directory names within configured search roots. Matching MUST be segment-aware: each query segment maps to one directory segment in order.

#### Scenario: Unambiguous prefix match
- **WHEN** the user queries an abbreviated path like `src/c/b` and only one directory tree matches
- **THEN** the system MUST resolve to the single matching path (e.g., `src/components/button`)

#### Scenario: Ambiguous prefix match
- **WHEN** the user queries an abbreviated path that matches multiple directories
- **THEN** the system MUST fail with a non-zero exit code indicating ambiguity, and MUST NOT silently pick a winner

### Requirement: Fallback Roots
The system MUST support configured fallback search roots (analogous to `CD_PATH`) that are searched when a query does not match as a direct, step-up, or relative path. Both exact name matches and abbreviated segment matches SHALL be attempted against fallback roots.

#### Scenario: Match in fallback root
- **WHEN** the user queries `myproject` and it does not exist relative to the current directory
- **AND** a configured fallback root contains a directory named `myproject`
- **THEN** the system MUST resolve to the absolute path of that directory within the fallback root

#### Scenario: No match in any root
- **WHEN** the user queries a name that does not match in the current directory or any configured fallback root
- **THEN** the system MUST fail with a non-zero exit code

### Requirement: Resolution Precedence
The system MUST evaluate resolution strategies in a fixed, deterministic order. The precedence MUST be:
1. Direct paths (absolute, relative, `~`, `..`)
2. Step-up aliases (multi-dot patterns)
3. Abbreviated segment matching against configured roots
4. Fallback root matching (CD_PATH-style)
5. Failure

The system MUST return the result from the first strategy that produces a match and MUST NOT continue to lower-precedence strategies.

#### Scenario: Direct path takes precedence over abbreviation
- **WHEN** the user queries `src` and a subdirectory `./src` exists in the current directory
- **AND** a fallback root also contains an `src` directory
- **THEN** the system MUST resolve to `./src` (direct relative path) without consulting fallback roots

#### Scenario: Step-up alias takes precedence over abbreviation
- **WHEN** the user queries `...`
- **THEN** the system MUST resolve it as a step-up alias (two levels up) regardless of whether a directory named `...` exists in any search root

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
