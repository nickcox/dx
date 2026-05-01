## ADDED Requirements

### Requirement: Delimiter-Aware Paths Completion Matching
`dx complete paths` SHALL use the resolver's delimiter-aware abbreviated-segment matching rules when collecting candidates.

This SHALL include support for word-delimiter shortening (`.`, `_`, `-`) and doubled-period gap operators (`..`) wherever the resolver would consider abbreviated or fallback-root matches.

#### Scenario: Hyphen-delimited query returns matching path candidate
- **WHEN** `dx complete paths cd-e` is invoked
- **AND** resolver candidate collection finds `/home/user/projects/cd-extras`
- **THEN** the output SHALL include `/home/user/projects/cd-extras`

#### Scenario: Dot-delimited query returns matching suffix-style candidate
- **WHEN** `dx complete paths .sdk` is invoked
- **AND** resolver candidate collection finds `/home/user/src/Microsoft.PowerShell.SDK`
- **THEN** the output SHALL include `/home/user/src/Microsoft.PowerShell.SDK`

#### Scenario: Doubled-period query returns matching candidate
- **WHEN** `dx complete paths p..shell` is invoked
- **AND** resolver candidate collection finds `/home/user/projects/PowerShell`
- **THEN** the output SHALL include `/home/user/projects/PowerShell`

## MODIFIED Requirements

### Requirement: Paths Mode
`dx complete paths <query>` SHALL return directory candidates by running the query through the resolver's candidate-collection strategy. This SHALL include abbreviated segment matches, delimiter-aware abbreviated matches, doubled-period gap matches, fallback root matches, and bookmark name matches - the same sources as `dx resolve`, but collecting all candidates instead of failing on ambiguity.

Candidates SHALL be deduplicated by absolute path and ordered by resolution precedence (direct > step-up > abbreviated > fallback > bookmark).

#### Scenario: Abbreviated query returns multiple candidates
- **WHEN** `dx complete paths pr` is invoked and abbreviation expansion finds `/home/user/projects` and `/home/user/presentations`
- **THEN** the output SHALL contain both paths, one per line

#### Scenario: Bookmark name matches
- **WHEN** `dx complete paths work` is invoked and a bookmark named `work` exists pointing to `/home/user/work`
- **THEN** `/home/user/work` SHALL appear in the output

#### Scenario: No matches
- **WHEN** `dx complete paths zzz` is invoked and no candidates are found
- **THEN** the command SHALL produce no output (empty stdout) and exit with code 0

#### Scenario: Delimiter-aware query returns multiple candidates
- **WHEN** `dx complete paths cd-e` is invoked and delimiter-aware matching finds both `/home/user/projects/cd-extras` and `/tmp/tools/cd-editor`
- **THEN** the output SHALL contain both paths, one per line
