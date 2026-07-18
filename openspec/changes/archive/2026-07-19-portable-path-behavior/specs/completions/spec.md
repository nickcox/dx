## ADDED Requirements

### Requirement: Portable Path Candidate Handling
Path completion, ancestor filtering, session filtering, and navigation selector matching SHALL preserve the native path identity of each candidate through collection, deduplication, ranking, and selection.

Path-aware filtering SHALL recognize native filesystem roots and separators. A root-only query SHALL remain a root selector and SHALL NOT normalize to an empty match-all query. Human-readable labels MAY be lossy, but identical labels SHALL not merge candidates or alter index-based selection.

#### Scenario: Root selector remains specific
- **WHEN** path-aware filtering runs with a native filesystem root as its query
- **THEN** it SHALL not return unrelated candidates solely because the normalized query is empty

#### Scenario: Colliding labels retain distinct selected paths
- **WHEN** two distinct candidate paths render to the same human-readable label
- **THEN** both candidates SHALL remain available and selecting either index SHALL return its original path

#### Scenario: Windows trailing separator lists directory children
- **WHEN** `dx complete paths` runs on Windows with an existing directory query ending in `\` or `/`
- **THEN** it SHALL return matching child directories using the native directory root

### Requirement: Significant Query and Selector Whitespace
Completion queries and navigation selectors SHALL preserve leading and trailing whitespace. Only an absent selector or a selector with zero characters SHALL select the first candidate by default.

#### Scenario: Whitespace-bearing selector is matched literally
- **WHEN** a navigation selector contains leading or trailing whitespace
- **THEN** the system SHALL use that whitespace as part of path matching rather than trimming it

#### Scenario: Whitespace-bearing completion query remains distinct
- **WHEN** a completion query names a directory with leading or trailing whitespace
- **THEN** candidate collection and filtering SHALL retain that whitespace in the query

### Requirement: Best-Effort Interactive Filesystem Discovery
Interactive completion candidate collection SHALL skip unreadable directories, entries whose metadata cannot be read, and entries that disappear during enumeration. It SHALL return any candidates that remain available and SHALL not emit resolver diagnostics for those skipped entries.

#### Scenario: Completion skips unreadable sibling
- **WHEN** completion enumerates a directory containing an unreadable sibling entry and another matching readable directory
- **THEN** it SHALL return the readable candidate without failing the completion command
