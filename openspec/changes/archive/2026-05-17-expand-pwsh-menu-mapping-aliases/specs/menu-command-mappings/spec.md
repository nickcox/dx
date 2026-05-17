## MODIFIED Requirements

### Requirement: Mapping Changes Require Explicit Re-Init
Mapped command seed registrations SHALL be determined when `dx init <shell> --menu` is generated.

Changing `DX_MENU_COMMAND_MAPPINGS` after hook generation SHALL NOT alter existing mapped-command seed registrations until `dx init <shell> --menu` is run again and the regenerated hooks are loaded.

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

## ADDED Requirements

### Requirement: PowerShell One-Way Alias Expansion for Mapped Commands
Generated PowerShell menu hooks SHALL expand each configured mapped command to aliases whose PowerShell alias `Definition` equals the configured command name when the hooks are loaded.

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
