## ADDED Requirements

### Requirement: Implicit Current Directory Root in Root-Based Resolution
Root-based resolution stages (abbreviated segment matching and fallback-root matching) SHALL include the current working directory as an implicit root by default.

This implicit cwd root SHALL participate only in root-based stages and SHALL NOT alter precedence of direct paths or step-up aliases.

#### Scenario: Abbreviation resolves using cwd when no roots configured
- **WHEN** no explicit search roots are configured and a query requires root-based abbreviation matching
- **THEN** the resolver SHALL evaluate cwd as an implicit root for abbreviation matching

#### Scenario: Direct path precedence remains unchanged
- **WHEN** a direct relative path match exists in cwd and root-based matches also exist
- **THEN** resolution SHALL still return the direct-path result before consulting root-based stages

#### Scenario: Implicit cwd root is deduplicated with configured roots
- **WHEN** configured roots already include cwd (or its normalized equivalent)
- **THEN** the effective root set SHALL contain only one cwd entry
