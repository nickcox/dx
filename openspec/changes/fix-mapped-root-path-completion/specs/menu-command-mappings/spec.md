## ADDED Requirements

### Requirement: Rooted Path Queries for Mapped Commands
Mapped external command candidate sourcing SHALL treat active-token queries beginning with `/` as rooted filesystem path queries.

For mapped `path`, `directory`, and `file` modes, a query of `/` SHALL list candidates from the filesystem root `/` according to the mapped mode filter and SHALL NOT include children of the current working directory solely because the query parent is empty after slash parsing.

For mapped `path`, `directory`, and `file` modes, a query of `/<prefix>` SHALL list matching children under `/` whose basenames match `<prefix>` according to the mapped mode filter and SHALL NOT include current-working-directory children solely due to the rooted query form.

Empty active-token queries and bare relative active-token queries SHALL continue to use the current working directory as their filesystem parent.

#### Scenario: Root slash query lists root children only
- **WHEN** a mapped command uses `path` mode with active-token query `/`
- **AND** the current working directory contains unrelated children
- **THEN** candidate sourcing SHALL include root children according to `path` mode
- **AND** candidate sourcing SHALL NOT include current-working-directory children solely because of the `/` query

#### Scenario: Rooted prefix query filters root children
- **WHEN** a mapped command uses `path` mode with active-token query `/U`
- **THEN** candidate sourcing SHALL consider children under `/` with basenames matching `U`
- **AND** candidate sourcing SHALL NOT include current-working-directory children solely because of the `/U` query

#### Scenario: Empty mapped query still uses cwd
- **WHEN** a mapped command uses `path` mode with an empty active-token query
- **THEN** candidate sourcing SHALL use the current working directory as the filesystem parent

#### Scenario: Bare relative mapped query still uses cwd
- **WHEN** a mapped command uses `path` mode with active-token query `src`
- **THEN** candidate sourcing SHALL use the current working directory as the filesystem parent with `src` as the leaf prefix
