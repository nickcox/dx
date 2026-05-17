## ADDED Requirements

### Requirement: Query-Style Candidate Labels
For filesystem path menu modes, `dx menu` SHALL render candidate item labels using the path style implied by the user's active query.

Filesystem path menu modes SHALL include `paths`, `path`, `directory`, and `file` modes.

When the active query is empty or a bare relative token without an explicit path prefix, cwd-local candidates SHALL be displayed as bare relative labels without a leading `./`.

When the active query starts with `./`, cwd-local candidates SHALL be displayed with a leading `./`.

When the active query starts with one or more `../` path segments, candidates representable relative to cwd SHALL be displayed using parent-relative labels that preserve the typed parent-relative style.

When the active query starts with `~` or `~/`, candidates under the user's home directory SHALL be displayed with a leading `~/`.

When the active query starts with `/`, candidates SHALL be displayed as absolute paths.

Candidate label style SHALL NOT change candidate sourcing, filtering, ranking, selected candidate identity, status-row selected path display, JSON action shape, or accepted replacement text.

#### Scenario: Empty cd query shows bare cwd children
- **WHEN** `dx menu` opens for `cd <tab>` in `/Users/nick/project`
- **AND** a candidate resolves to `/Users/nick/project/src`
- **THEN** the candidate item label SHALL be `src` rather than `./src`

#### Scenario: Bare relative query shows bare cwd child
- **WHEN** `dx menu` opens for `cd s<tab>` in `/Users/nick/project`
- **AND** a candidate resolves to `/Users/nick/project/src`
- **THEN** the candidate item label SHALL be `src`

#### Scenario: Dot-relative query preserves dot prefix
- **WHEN** `dx menu` opens for `cd ./<tab>` in `/Users/nick/project`
- **AND** a candidate resolves to `/Users/nick/project/src`
- **THEN** the candidate item label SHALL be `./src`

#### Scenario: Parent-relative query preserves parent prefix
- **WHEN** `dx menu` opens for `cd ../<tab>` in `/Users/nick/project`
- **AND** a candidate resolves to `/Users/nick/sibling`
- **THEN** the candidate item label SHALL be `../sibling`

#### Scenario: Home query preserves home prefix
- **WHEN** `dx menu` opens for `cd ~/<tab>`
- **AND** a candidate resolves under the user's home directory as `/Users/nick/code`
- **THEN** the candidate item label SHALL be `~/code`

#### Scenario: Absolute query preserves absolute path
- **WHEN** `dx menu` opens for `cd /Users/nick/<tab>`
- **AND** a candidate resolves to `/Users/nick/code`
- **THEN** the candidate item label SHALL be `/Users/nick/code`

#### Scenario: Label style does not affect replacement
- **WHEN** `dx menu` opens for `cd s<tab>` in `/Users/nick/project`
- **AND** a candidate item label is displayed as `src`
- **THEN** accepting the candidate SHALL preserve existing replacement formatting behavior
