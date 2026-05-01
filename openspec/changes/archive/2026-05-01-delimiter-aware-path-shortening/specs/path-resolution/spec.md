## ADDED Requirements

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

## MODIFIED Requirements

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
