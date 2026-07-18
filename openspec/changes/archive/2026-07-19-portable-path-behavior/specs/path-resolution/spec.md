## ADDED Requirements

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

## MODIFIED Requirements

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
