## ADDED Requirements

### Requirement: PowerShell Filesystem Stack Recording
The PowerShell location wrapper SHALL record a location transition in the dx session stack only after native `Set-Location` completes and the effective location changed to a FileSystem provider location.

For a transition between FileSystem locations, the wrapper SHALL push the starting path followed by the destination path so a previously uninitialized dx session retains the origin. For a transition from a non-FileSystem provider to a FileSystem location, the wrapper SHALL push only the destination. Stack recording SHALL remain fire-and-forget and SHALL NOT alter native output or the result of a successful location change.

The wrapper SHALL NOT store non-FileSystem provider locations in the dx session stack.

#### Scenario: Successful filesystem transition records origin and destination
- **WHEN** `Set-DxLocation` successfully changes from one FileSystem location to a different FileSystem location
- **THEN** the wrapper SHALL push the starting path and then the destination path to the dx session stack

#### Scenario: Failed location change does not update stack
- **WHEN** native `Set-Location` fails without changing the effective location
- **THEN** the wrapper SHALL NOT call `dx stack push`

#### Scenario: Provider destination is not recorded
- **WHEN** `Set-DxLocation` successfully changes from a FileSystem location to a non-FileSystem provider location
- **THEN** the wrapper SHALL NOT store the provider location in the dx session stack

#### Scenario: Return from provider records filesystem destination
- **WHEN** `Set-DxLocation` successfully changes from a non-FileSystem provider location to a FileSystem location
- **THEN** the wrapper SHALL push only the FileSystem destination to the dx session stack

#### Scenario: Stack push failure does not change native result
- **WHEN** native `Set-Location` succeeds but a subsequent `dx stack push` fails
- **THEN** the location change SHALL remain successful and native `Set-Location` output SHALL remain unchanged

## MODIFIED Requirements

### Requirement: cd Wrapper — Flag Passthrough
The Bash, Zsh, and Fish cd wrappers SHALL pass shell-specific flags supported by their native cd command through unchanged. The wrapper SHALL only resolve the path argument, not flags.

PowerShell SHALL use native `Set-Location` parameter binding instead of treating `-L`, `-P`, or other POSIX cd flags as PowerShell parameters.

#### Scenario: Physical flag passthrough
- **WHEN** `cd -P /some/symlink` is invoked in Bash or Zsh
- **THEN** the `-P` flag SHALL be passed to native `cd` and `dx resolve` SHALL receive only the path argument

#### Scenario: PowerShell does not emulate POSIX flags
- **WHEN** the PowerShell location wrapper is invoked
- **THEN** it SHALL bind `Set-Location` parameters and SHALL NOT parse POSIX `cd` flags

### Requirement: PowerShell Set-Location Wrapper
The PowerShell hook code SHALL define a `Set-DxLocation` advanced function that wraps the fully qualified native `Microsoft.PowerShell.Management\Set-Location` cmdlet. The function SHALL expose the native `Path`, `LiteralPath`, and `StackName` parameter sets, including `PassThru`, native pipeline binding, property-name binding, parameter aliases, and common parameters.

For a directly supplied `Path` argument, the wrapper SHALL attempt `dx resolve` only when the current provider is FileSystem and the argument is eligible for dx filesystem resolution. Native history tokens (`-` and `+`), provider-qualified paths, wildcard paths, pipeline input, `LiteralPath`, and `StackName` SHALL bypass dx resolution. If dx resolution succeeds, the wrapper SHALL pass the resolved path to native `Set-Location`; otherwise it SHALL pass the original argument unchanged.

The wrapper SHALL delegate no-argument home navigation, location history, provider semantics, wildcard handling, literal-path handling, named-stack selection, output, and errors to native `Set-Location`. The generated hook code SHALL install `cd` as an alias to `Set-DxLocation` rather than defining a function literally named `cd`.

#### Scenario: PowerShell cd wrapper resolves eligible filesystem path
- **WHEN** `cd pr/dx` is invoked from a FileSystem location and `dx resolve "pr/dx"` succeeds
- **THEN** the wrapper SHALL call native `Set-Location` with the resolved path

#### Scenario: PowerShell resolution failure preserves native fallback
- **WHEN** an eligible direct `Path` argument cannot be resolved by dx
- **THEN** the wrapper SHALL call native `Set-Location` with the original argument unchanged

#### Scenario: No-argument invocation delegates home navigation
- **WHEN** `Set-DxLocation` is invoked without arguments
- **THEN** the wrapper SHALL invoke native `Set-Location` without a path and navigate according to native home-directory behavior

#### Scenario: Native history tokens bypass dx resolution
- **WHEN** `cd -` or `cd +` is invoked
- **THEN** the wrapper SHALL pass the history token to native `Set-Location` without invoking `dx resolve`

#### Scenario: Literal path preserves native semantics
- **WHEN** `Set-DxLocation -LiteralPath <path>` is invoked for a path containing wildcard characters
- **THEN** the wrapper SHALL forward `LiteralPath` without invoking `dx resolve` or interpreting wildcard characters

#### Scenario: PassThru preserves native output
- **WHEN** `Set-DxLocation -Path <path> -PassThru` succeeds
- **THEN** the wrapper SHALL emit the native `PathInfo` output without additional stack-command output

#### Scenario: Pipeline input uses native binding
- **WHEN** one or more path values are piped to `Set-DxLocation`
- **THEN** the wrapper SHALL process them through native `Set-Location` pipeline semantics without invoking `dx resolve`

#### Scenario: Provider-qualified path bypasses dx
- **WHEN** a provider-qualified path is supplied to `Set-DxLocation`
- **THEN** the wrapper SHALL pass it directly to native `Set-Location` without invoking `dx resolve`

#### Scenario: Named stack selection preserves native behavior
- **WHEN** `Set-DxLocation -StackName <name>` is invoked
- **THEN** the wrapper SHALL select the native PowerShell location stack without invoking `dx resolve`

#### Scenario: PowerShell CommandNotFoundAction feature detection
- **WHEN** `dx init pwsh --command-not-found` output is evaluated and `$ExecutionContext.InvokeCommand` has a `CommandNotFoundAction` member
- **THEN** the hook code SHALL register a handler via `CommandNotFoundAction`

#### Scenario: PowerShell without CommandNotFoundAction
- **WHEN** `dx init pwsh --command-not-found` output is evaluated and `CommandNotFoundAction` member does not exist
- **THEN** the hook code SHALL skip command_not_found registration gracefully without errors

#### Scenario: PowerShell aliases cd to dx location wrapper
- **WHEN** `dx init pwsh` output is evaluated
- **THEN** the script SHALL install `Alias:cd` pointing at `Set-DxLocation`
- **AND** it SHALL NOT rely on defining a function literally named `cd`
