## ADDED Requirements

### Requirement: Path Mode Directory Replacement Uses Trailing Slash
When `dx menu` is invoked in explicit mapped `path` mode and the selected candidate is a directory, the replace action value SHALL include a trailing `/`.

When `dx menu` is invoked in explicit mapped `path` mode and the selected candidate is a file, the replace action value SHALL NOT append a trailing `/` solely because the mode is `path`.

Existing replacement formatting rules SHALL remain unchanged for built-in `paths` mode, mapped `directory` mode, mapped `file` mode, and non-filesystem completion modes.

When a mapped `path` mode directory replacement requires shell quoting, the trailing `/` SHALL be included inside the quoted path token.

#### Scenario: Path mode directory selection appends slash
- **WHEN** `dx menu --mode path` selects a directory candidate
- **THEN** the replace action value SHALL end with `/`

#### Scenario: Path mode file selection does not append slash
- **WHEN** `dx menu --mode path` selects a file candidate
- **THEN** the replace action value SHALL NOT append `/` solely because the selection came from `path` mode

#### Scenario: Path mode quoted directory keeps slash inside quotes
- **WHEN** `dx menu --mode path` selects a directory candidate whose replacement value requires shell quoting
- **THEN** the replace action value SHALL include the trailing `/` inside the quoted path token
