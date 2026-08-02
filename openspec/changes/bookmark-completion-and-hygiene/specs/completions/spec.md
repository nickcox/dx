## MODIFIED Requirements

### Requirement: Paths Mode
`dx complete paths <query>` SHALL return directory candidates by running the query through the resolver's candidate-collection strategy. This SHALL include abbreviated segment matches, delimiter-aware abbreviated matches, doubled-period gap matches, fallback root matches, and bookmark name prefix matches - the same sources as `dx resolve`, but collecting all candidates instead of failing on ambiguity.

Bookmark names SHALL be matched by prefix rather than only exactly, and a bookmark whose target is no longer a directory SHALL be excluded. Because `dx resolve` matches bookmark names exactly, a bookmark name prefix SHALL complete without resolving.

Candidates SHALL be deduplicated by absolute path and ordered by resolution precedence (direct > step-up > abbreviated > fallback > bookmark).

#### Scenario: Abbreviated query returns multiple candidates
- **WHEN** `dx complete paths pr` is invoked and abbreviation expansion finds `/home/user/projects` and `/home/user/presentations`
- **THEN** the output SHALL contain both paths, one per line

#### Scenario: Bookmark name prefix matches
- **WHEN** `dx complete paths wo` is invoked and a bookmark named `work` exists pointing to an existing `/home/user/work`
- **THEN** the output SHALL contain `/home/user/work`, ordered after any filesystem-derived candidates
