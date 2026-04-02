## ADDED Requirements

### Requirement: Implicit Current Directory Root in Paths Completion
`dx complete paths` SHALL include the current working directory as an implicit search root when collecting root-based candidates, even if no explicit search roots are configured.

If explicit roots are configured, cwd SHALL still be included in effective root evaluation unless it is already represented after normalization.

#### Scenario: Paths completion works without configured roots
- **WHEN** `DX_SEARCH_ROOTS` is unset and `dx complete paths src` is invoked from a directory containing `./src`
- **THEN** completion candidate collection SHALL consider cwd as a root and include matching cwd-based candidates

#### Scenario: Cwd root is deduplicated when already configured
- **WHEN** cwd is already present in configured search roots
- **THEN** effective root evaluation SHALL include cwd only once after normalization/deduplication
