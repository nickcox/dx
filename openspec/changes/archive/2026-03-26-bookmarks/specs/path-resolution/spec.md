## MODIFIED Requirements

### Requirement: Resolution Precedence
The system MUST evaluate resolution strategies in a fixed, deterministic order. The precedence MUST be:
1. Direct paths (absolute, relative, `~`, `..`)
2. Step-up aliases (multi-dot patterns)
3. Abbreviated segment matching against configured roots
4. Fallback root matching (CD_PATH-style)
5. Bookmark lookup (exact name match against persistent bookmarks)
6. Failure

The system MUST return the result from the first strategy that produces a match and MUST NOT continue to lower-precedence strategies.

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
