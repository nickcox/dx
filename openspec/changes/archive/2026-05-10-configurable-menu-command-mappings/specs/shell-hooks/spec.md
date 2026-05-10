## ADDED Requirements

### Requirement: Menu-Enabled Registration for Configured Mapped Commands
When `dx init <shell> --menu` is used and `DX_MENU_COMMAND_MAPPINGS` contains valid mappings, generated hooks SHALL register mapped command names for menu-backed completion in that shell.

Registration SHALL preserve existing native fallback behavior for noop/error/invalid payload paths.
Generated mapped-command registrations SHALL invoke `dx menu --mode <mode>` with the explicit mode captured at init time.

#### Scenario: Bash registers mapped commands
- **WHEN** `dx init bash --menu` is run with `DX_MENU_COMMAND_MAPPINGS="ls=path,cat=file"`
- **THEN** generated Bash completion registration SHALL bind `ls` and `cat` to the shared menu completion handler with explicit `--mode path` and `--mode file` invocation respectively

#### Scenario: Zsh registers mapped commands
- **WHEN** `dx init zsh --menu` is run with `DX_MENU_COMMAND_MAPPINGS="open=path"`
- **THEN** generated Zsh hook output SHALL route `open` through the shared menu widget with explicit `--mode path`

#### Scenario: Fish registers mapped commands
- **WHEN** `dx init fish --menu` is run with `DX_MENU_COMMAND_MAPPINGS="ls=path"`
- **THEN** generated Fish hook output SHALL route `ls` through the shared menu helper with explicit `--mode path`

#### Scenario: PowerShell registers mapped commands
- **WHEN** `dx init pwsh --menu` is run with `DX_MENU_COMMAND_MAPPINGS="cat=file"`
- **THEN** generated PowerShell output SHALL route `cat` through the shared PSReadLine menu handler with explicit `--mode file`

### Requirement: Invalid Mappings Fail Init Generation
If `DX_MENU_COMMAND_MAPPINGS` contains an invalid entry, `dx init <shell> --menu` SHALL fail rather than emitting partial mapped-command registrations.

#### Scenario: Invalid mapping prevents partial hook emission
- **WHEN** `dx init bash --menu` is run with `DX_MENU_COMMAND_MAPPINGS="ls=path,badentry"`
- **THEN** init generation SHALL fail and SHALL NOT emit partial mapped-command registrations

### Requirement: Global Menu Enablement Gate for Mapped Commands
Configured mapped command registrations SHALL apply only when global menu integration is enabled via `dx init <shell> --menu`.

Without `--menu`, mapped commands SHALL NOT be registered for menu handling.

#### Scenario: Mappings ignored when menu flag is not enabled
- **WHEN** `dx init bash` is used without `--menu` and mappings are present in environment
- **THEN** generated hooks SHALL NOT install mapped-command menu completion bindings

### Requirement: Mapping Changes Require Re-Running Init
Changing `DX_MENU_COMMAND_MAPPINGS` after hooks are generated SHALL NOT change mapped-command behavior until `dx init <shell> --menu` is re-run and the regenerated hooks are loaded.

#### Scenario: Runtime env change does not update existing hook behavior
- **WHEN** hooks were generated from `DX_MENU_COMMAND_MAPPINGS="ls=path"` and the environment later changes without re-running `dx init`
- **THEN** the existing hook behavior SHALL continue using the previously generated mapped-command registrations

### Requirement: Token-Scoped Buffer Replacement for Mapped Commands
Shell integrations for mapped commands SHALL preserve existing replace action semantics and SHALL apply replacement only to the active token span reported by `dx menu`.

#### Scenario: PowerShell replaces only mapped command active token
- **WHEN** mapped command buffer is `open src/readme.md --wait` and replace action targets token `src/readme.md`
- **THEN** hook logic SHALL replace only that token span and preserve remaining buffer text
