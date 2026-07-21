## Purpose
Define configuration and behavior for opt-in menu-backed command mappings that let external commands use the Rust TUI or native PowerShell completion with explicit filesystem candidate modes.

## Requirements

### Requirement: Command Mapping Environment Schema for Init Generation
`dx init <shell> --menu` and `dx init pwsh --native-menu` SHALL accept command-to-mode mappings from `DX_MENU_COMMAND_MAPPINGS` using a comma-separated grammar: `<command>=<mode>`.

`<mode>` SHALL be one of `path`, `directory`, or `file`.

#### Scenario: Parse valid mappings during init generation
- **WHEN** `DX_MENU_COMMAND_MAPPINGS="ls=path,open=path,cat=file"` and `dx init bash --menu` is run
- **THEN** init generation SHALL accept three mappings using only their configured command and mode values

#### Scenario: Invalid mapping causes init failure
- **WHEN** `DX_MENU_COMMAND_MAPPINGS="ls=path,badentry,cat=unknown"` and `dx init zsh --menu` is run
- **THEN** init generation SHALL fail and SHALL NOT emit partial mapped-command registrations

### Requirement: Mapping Changes Require Explicit Re-Init
Mapped command seed registrations SHALL be determined when `dx init <shell> --menu` or `dx init pwsh --native-menu` is generated.

Changing `DX_MENU_COMMAND_MAPPINGS` after hook generation SHALL NOT alter existing mapped-command seed registrations until the corresponding menu-enabled init command is run again and the regenerated hooks are loaded.

For PowerShell, alias-expanded registrations SHALL be determined when the generated hooks are loaded from the generated seed registrations and the aliases visible in the current PowerShell session.

Changing PowerShell aliases after hook load SHALL NOT alter existing alias-expanded mapped-command registrations until the hooks are loaded again.

#### Scenario: Existing hooks keep prior mapping set
- **WHEN** hooks were generated with `DX_MENU_COMMAND_MAPPINGS="ls=path"` and the environment is later changed to `DX_MENU_COMMAND_MAPPINGS="cat=file"` without re-running `dx init`
- **THEN** mapped-command behavior SHALL continue using the previously generated `ls=path` seed registration set

#### Scenario: Re-running init applies new mapping set
- **WHEN** `DX_MENU_COMMAND_MAPPINGS` changes and the user re-runs `dx init fish --menu`
- **THEN** newly generated hooks SHALL reflect the updated mapping seed set

#### Scenario: PowerShell hook load freezes visible aliases
- **WHEN** generated PowerShell hooks are loaded while alias `gci` points to `Get-ChildItem`
- **AND** the generated mapping seed set contains `Get-ChildItem=path`
- **THEN** mapped-command behavior SHALL match `gci` in `path` mode
- **AND** later changes to alias `gci` SHALL NOT alter mapped-command behavior until the hooks are loaded again

### Requirement: PowerShell One-Way Alias Expansion for Mapped Commands
Generated PowerShell TUI and native menu hooks SHALL expand each configured mapped command to aliases whose PowerShell alias `Definition` equals the configured command name when the hooks are loaded.

The configured command itself SHALL remain mapped even when no aliases are found.

Alias expansion SHALL be one-way from configured command name to alias names. Configuring an alias name SHALL NOT require the system to infer the alias definition or sibling aliases.

Explicit configured command mappings SHALL take precedence over derived alias mappings. If multiple derived alias mappings collide, the first configured seed mapping that derives the alias SHALL win.

Bash, Zsh, and Fish mappings SHALL continue to use only the configured command names.

#### Scenario: Canonical PowerShell command maps aliases
- **WHEN** generated PowerShell hooks are loaded with mapping seed `Get-ChildItem=path`
- **AND** aliases `gci` and `dir` have alias definition `Get-ChildItem`
- **THEN** mapped-command behavior SHALL match `Get-ChildItem`, `gci`, and `dir` in `path` mode

#### Scenario: Configured alias does not expand back to canonical command
- **WHEN** generated PowerShell hooks are loaded with mapping seed `gci=path`
- **AND** alias `gci` has alias definition `Get-ChildItem`
- **THEN** mapped-command behavior SHALL match `gci` in `path` mode
- **AND** mapped-command behavior SHALL NOT match `Get-ChildItem` solely because `gci` is an alias of it

#### Scenario: Explicit mapping wins over derived alias
- **WHEN** generated PowerShell hooks are loaded with mapping seeds `Get-ChildItem=path,gci=file`
- **AND** alias `gci` has alias definition `Get-ChildItem`
- **THEN** mapped-command behavior SHALL match `gci` in `file` mode

#### Scenario: Non-PowerShell shells do not expand aliases
- **WHEN** `DX_MENU_COMMAND_MAPPINGS="Get-ChildItem=path"` and `dx init bash --menu` is run
- **THEN** generated Bash hooks SHALL register only the configured `Get-ChildItem` mapped command

### Requirement: Native PowerShell Mapping Registration
Native PowerShell mappings SHALL use `Register-ArgumentCompleter` and the shared filesystem completion modes. Native applications SHALL receive native command completers. PowerShell commands SHALL use `Path`, then `LiteralPath`, then the first positional string parameter.

When no suitable PowerShell parameter exists, hook loading SHALL emit a warning rather than registering a completer that cannot run.

#### Scenario: Native mapping uses a PowerShell path parameter
- **WHEN** `DX_MENU_COMMAND_MAPPINGS="Get-Content=file"` and `dx init pwsh --native-menu` hooks are loaded
- **THEN** `Get-Content` and its derived aliases SHALL receive a file-only completer for their `Path` parameter

#### Scenario: Native application mapping uses native registration
- **WHEN** a configured mapped command resolves to a native application while `dx init pwsh --native-menu` hooks are loaded
- **THEN** the command SHALL receive a native argument completer for its configured filesystem mode

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
