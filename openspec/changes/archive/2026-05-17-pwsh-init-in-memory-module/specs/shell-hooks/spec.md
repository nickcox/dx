## ADDED Requirements

### Requirement: PowerShell Init Module Lifecycle
Generated PowerShell init output SHALL load dx shell integration as an in-memory PowerShell module named `dx`.

The module SHALL keep dx helper functions and saved cleanup state in module scope rather than defining all implementation details as loose caller-scope functions.

The module SHALL register an `OnRemove` cleanup handler so `Remove-Module dx` restores or removes the session state that dx replaced during module import where feasible.

The generated output SHALL remain self-contained and SHALL NOT require a cached `.psm1`, packaged module, external download, or filesystem write.

#### Scenario: PowerShell init imports dx module
- **WHEN** `dx init pwsh` output is evaluated as one script block
- **THEN** PowerShell SHALL load an in-memory module named `dx`
- **AND** `Get-Module dx` SHALL be able to identify the loaded integration as a module

#### Scenario: Module unload restores replaced cd alias
- **WHEN** a PowerShell session has an existing `cd` alias before evaluating `dx init pwsh` output
- **AND** the generated output is evaluated and then `Remove-Module dx` is invoked
- **THEN** the prior `cd` alias target SHALL be restored where feasible

#### Scenario: Module unload removes dx-owned aliases without prior state
- **WHEN** dx installs a PowerShell alias that did not exist before module import
- **AND** `Remove-Module dx` is invoked
- **THEN** the dx-installed alias SHALL be removed where feasible

#### Scenario: PowerShell init remains self-contained
- **WHEN** `dx init pwsh` output is evaluated in a PowerShell session
- **THEN** it SHALL load the dx module without reading a generated module file from disk

## MODIFIED Requirements

### Requirement: Init Subcommand
The `dx init <shell>` subcommand SHALL accept a shell identifier and print shell-specific hook code to stdout. The supported shell identifiers SHALL be `bash`, `zsh`, `fish`, and `pwsh`.

If an unsupported shell identifier is provided, the command SHALL exit with a non-zero exit code and print a diagnostic to stderr.

The output SHALL be self-contained: evaluating it in the target shell SHALL define all hooks without requiring additional files or downloads.

#### Scenario: Generate Bash hooks
- **WHEN** `dx init bash` is invoked
- **THEN** the command SHALL print valid Bash code to stdout that, when evaluated, defines a `cd` wrapper function and exports `DX_SESSION`

#### Scenario: Generate Zsh hooks
- **WHEN** `dx init zsh` is invoked
- **THEN** the command SHALL print valid Zsh code to stdout that, when evaluated, defines a `cd` wrapper function and exports `DX_SESSION`

#### Scenario: Generate Fish hooks
- **WHEN** `dx init fish` is invoked
- **THEN** the command SHALL print valid Fish code to stdout that, when evaluated, defines a `cd` wrapper function and sets `DX_SESSION` as a universal or exported variable

#### Scenario: Generate PowerShell hooks
- **WHEN** `dx init pwsh` is invoked
- **THEN** the command SHALL print valid PowerShell code to stdout that, when evaluated via `Invoke-Expression`, imports an in-memory `dx` module, installs the PowerShell navigation aliases, and sets `$env:DX_SESSION`

#### Scenario: Unsupported shell
- **WHEN** `dx init unknown` is invoked
- **THEN** the command SHALL exit with a non-zero exit code and print a diagnostic to stderr listing the supported shells

### Requirement: PowerShell Set-Location Wrapper
The PowerShell hook code SHALL define a real dx location wrapper command that wraps `Set-Location` (PowerShell has no `builtin cd`). The wrapper SHALL follow the same resolve-then-navigate-then-push flow as POSIX shells but using `Set-Location` as the native navigation command.

The generated hook code SHALL install `cd` as an alias to the dx location wrapper command rather than defining a function literally named `cd`.

#### Scenario: PowerShell cd wrapper uses Set-Location
- **WHEN** `cd pr/dx` is invoked in a PowerShell session with dx hooks
- **THEN** the hook SHALL call `dx resolve`, and on success call `Set-Location` with the resolved path, then `dx stack push`

#### Scenario: PowerShell CommandNotFoundAction feature detection
- **WHEN** `dx init pwsh --command-not-found` output is evaluated and `$ExecutionContext.InvokeCommand` has a `CommandNotFoundAction` member
- **THEN** the hook code SHALL register a handler via `CommandNotFoundAction`

#### Scenario: PowerShell without CommandNotFoundAction
- **WHEN** `dx init pwsh --command-not-found` output is evaluated and `CommandNotFoundAction` member does not exist
- **THEN** the hook code SHALL skip command_not_found registration gracefully without errors

#### Scenario: PowerShell aliases cd to dx location wrapper
- **WHEN** `dx init pwsh` output is evaluated
- **THEN** the script SHALL install `Alias:cd` pointing at the dx location wrapper command
- **AND** it SHALL NOT rely on defining a function literally named `cd`
