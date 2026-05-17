## Purpose
Define configuration and behavior for opt-in menu-backed command mappings that let external commands use `dx menu` with explicit filesystem candidate modes.

## Requirements

### Requirement: Command Mapping Environment Schema for Init Generation
`dx init <shell> --menu` SHALL accept command-to-mode mappings from `DX_MENU_COMMAND_MAPPINGS` using a comma-separated grammar: `<command>=<mode>`.

`<mode>` SHALL be one of `path`, `directory`, or `file`.

#### Scenario: Parse valid mappings during init generation
- **WHEN** `DX_MENU_COMMAND_MAPPINGS="ls=path,open=path,cat=file"` and `dx init bash --menu` is run
- **THEN** init generation SHALL accept three mappings using only their configured command and mode values

#### Scenario: Invalid mapping causes init failure
- **WHEN** `DX_MENU_COMMAND_MAPPINGS="ls=path,badentry,cat=unknown"` and `dx init zsh --menu` is run
- **THEN** init generation SHALL fail and SHALL NOT emit partial mapped-command registrations

### Requirement: Mapping Changes Require Explicit Re-Init
Mapped command registrations SHALL be determined when `dx init <shell> --menu` is generated.

Changing `DX_MENU_COMMAND_MAPPINGS` after hook generation SHALL NOT alter existing mapped-command registrations until `dx init <shell> --menu` is run again and the regenerated hooks are loaded.

#### Scenario: Existing hooks keep prior mapping set
- **WHEN** hooks were generated with `DX_MENU_COMMAND_MAPPINGS="ls=path"` and the environment is later changed to `DX_MENU_COMMAND_MAPPINGS="cat=file"` without re-running `dx init`
- **THEN** mapped-command behavior SHALL continue using the previously generated `ls=path` registration set

#### Scenario: Re-running init applies new mapping set
- **WHEN** `DX_MENU_COMMAND_MAPPINGS` changes and the user re-runs `dx init fish --menu`
- **THEN** newly generated hooks SHALL reflect the updated mapping set

### Requirement: Explicit Mode Semantics
Mapped commands SHALL use explicit candidate-mode semantics:
- `path` SHALL allow file and directory candidates
- `directory` SHALL allow only directory candidates
- `file` SHALL allow only regular file candidates

Mapped-command candidate sourcing SHALL use a filesystem candidate source capable of surfacing both files and directories before mode filtering is applied.

Mode filtering SHALL be applied before emitting a replace action value.

#### Scenario: Directory mode excludes files
- **WHEN** a mapped command uses `directory` mode and candidate discovery finds both files and directories
- **THEN** only directory candidates SHALL remain selectable

#### Scenario: File mode excludes directories
- **WHEN** a mapped command uses `file` mode and candidate discovery finds both files and directories
- **THEN** only file candidates SHALL remain selectable

#### Scenario: Path mode includes files and directories
- **WHEN** a mapped command uses `path` mode and candidate discovery finds both files and directories
- **THEN** both file and directory candidates SHALL remain selectable

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

### Requirement: dx-smart Resolution for Mapped External Commands
For mapped external commands, completion candidate discovery SHALL use `dx-smart` behavior before action output is produced.

#### Scenario: Mapped external command uses dx-smart by default
- **WHEN** a mapped command is registered in `path` mode and menu handling runs for that command
- **THEN** candidate resolution SHALL use `dx-smart` behavior

### Requirement: Current-Token Scope for V1
Mapped command processing in v1 SHALL operate only on the current token under the cursor.

The system SHALL NOT require command-specific parsing of non-active tokens to generate replacement bounds or values.

#### Scenario: Replace only active token under cursor
- **WHEN** buffer is `ls src --color` and cursor is inside token `src`
- **THEN** replacement bounds SHALL target only `src` and SHALL NOT alter `ls ` or ` --color`

#### Scenario: Non-active tokens are ignored for mapping semantics
- **WHEN** buffer contains multiple arguments for a mapped command
- **THEN** mapped completion behavior SHALL be determined without command-specific parsing of non-active tokens
