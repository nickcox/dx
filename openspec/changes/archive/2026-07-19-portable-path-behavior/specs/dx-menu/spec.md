## ADDED Requirements

### Requirement: Portable Mapped Filesystem Completion
For mapped filesystem menu modes (`path`, `directory`, and `file`), `dx menu` SHALL use the same native path-query interpretation as `dx complete paths` for roots, home expansion, explicit-relative paths, separators, parent extraction, and trailing separators.

Mapped filesystem completion SHALL preserve significant leading and trailing query whitespace and candidate `PathBuf` identity. Unreadable or disappearing entries SHALL be skipped without preventing available candidates from being offered.

#### Scenario: Windows mapped path completion accepts backslash input
- **WHEN** `dx menu --mode path` runs on Windows with an active query using `\` separators
- **THEN** it SHALL source matching candidates from the corresponding native parent directory

#### Scenario: Windows mapped UNC query retains its share root
- **WHEN** `dx menu --mode directory` runs on Windows with an active UNC query
- **THEN** it SHALL derive parents and candidates under that UNC share root rather than under cwd

#### Scenario: Whitespace-bearing mapped query is preserved
- **WHEN** an active mapped filesystem query contains leading or trailing whitespace that is part of a directory name
- **THEN** `dx menu` SHALL use the whitespace-bearing query without trimming it

### Requirement: Native Shell Replacement Paths
For filesystem menu selections, the replacement formatter SHALL preserve native drive and UNC prefixes, use the native separator when adding a directory suffix, and preserve the selected path's identity until the shell-string output boundary.

If a selected path cannot be represented safely by the shell action's UTF-8 string value, `dx menu` SHALL return `noop` rather than emitting a lossy replacement that could target a different path.

#### Scenario: Windows directory replacement uses a native separator
- **WHEN** `dx menu` runs on Windows and selects a directory in a mode requiring a trailing directory separator
- **THEN** the replacement value SHALL end with `\` inside any required shell quoting

#### Scenario: Drive-qualified absolute replacement preserves its prefix
- **WHEN** `dx menu` runs on Windows with an explicitly drive-qualified input and selects a candidate
- **THEN** the replacement value SHALL remain drive-qualified and absolute

#### Scenario: Unrepresentable selected path returns noop
- **WHEN** a selected filesystem path cannot be represented safely in the JSON shell action string
- **THEN** `dx menu` SHALL return `{ "action": "noop" }` and SHALL not emit a lossy replacement
