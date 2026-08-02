## MODIFIED Requirements

### Requirement: Ambiguity Handling
When multiple candidates match at the same precedence level, the system MUST fail by default rather than guessing. The system MUST support `--list` and `--json` flags to return ranked candidates instead of failing.

These flags MUST change only how the outcome is presented, never whether the command succeeded. An ambiguous query has not resolved to one directory, so it MUST exit with a non-zero code in every mode.

#### Scenario: Ambiguous match in default mode
- **WHEN** a query matches multiple directories at the same precedence level
- **AND** neither `--list` nor `--json` is specified
- **THEN** the system MUST exit with a non-zero code and output a diagnostic to stderr indicating ambiguity

#### Scenario: Ambiguous match with --list flag
- **WHEN** a query matches multiple directories at the same precedence level
- **AND** `--list` is specified
- **THEN** the system MUST output all matching candidates to stdout (one per line), output nothing to stderr, and exit with a non-zero code

#### Scenario: Ambiguous match with --json flag
- **WHEN** a query matches multiple directories at the same precedence level
- **AND** `--json` is specified
- **THEN** the system MUST output a JSON object containing the status, candidates, and reason to stdout, output nothing to stderr, and exit with a non-zero code

### Requirement: Output and Error Contracts
The `dx resolve` command MUST provide shell-consumable output with strict success and failure semantics.

The command MUST exit with code 0 if and only if the query resolved to exactly one directory. Every other outcome MUST exit with a non-zero code, regardless of output mode.

Exactly one stream MUST carry the outcome. When a machine-readable mode reports a failure on stdout, the system MUST NOT also write a diagnostic to stderr. This makes the streams a reliable discriminator: an ambiguous query leaves stdout non-empty and stderr empty, while a hard failure leaves stdout empty and stderr non-empty.

Only ambiguity and not-found have a JSON representation. Any other failure — an unreadable directory, an unsupported drive-relative query — MUST produce empty stdout and a stderr diagnostic even when `--json` is specified.

#### Scenario: Successful resolution
- **WHEN** a path query is successfully resolved to exactly one directory
- **THEN** the system MUST output exactly one absolute path to stdout (no trailing newline beyond the line terminator) and exit with code 0

#### Scenario: Unsuccessful resolution
- **WHEN** a path query cannot be resolved to any directory
- **THEN** the system MUST output nothing to stdout, output a diagnostic message to stderr, and exit with a non-zero code

#### Scenario: Resolved path does not exist
- **WHEN** a query resolves syntactically (e.g., `~/nonexistent`) but the target directory does not exist on disk
- **THEN** the system MUST treat this as an unsuccessful resolution

#### Scenario: Not-found reported as JSON
- **WHEN** a query cannot be resolved and `--json` is specified
- **THEN** the system MUST output a JSON object with a `not_found` reason to stdout, output nothing to stderr, and exit with a non-zero code

#### Scenario: Failure with no JSON representation
- **WHEN** resolution fails for a reason other than ambiguity or not-found and `--json` is specified
- **THEN** the system MUST output nothing to stdout, output a diagnostic message to stderr, and exit with a non-zero code
