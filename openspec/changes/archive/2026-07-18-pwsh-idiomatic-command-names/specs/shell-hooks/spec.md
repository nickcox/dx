## ADDED Requirements

### Requirement: PowerShell Module Uses Idiomatic Primary Command Names
Generated PowerShell init output SHALL define and export idiomatic primary functions for dx navigation operations inside the `dx` module.

The primary function names SHALL be:
- `Set-DxLocation` for `cd` integration
- `Step-Up` for ancestor navigation
- `Undo-Location` for backward stack traversal
- `Redo-Location` for forward stack traversal
- `Set-FrecentLocation` for frecent jumps
- `Set-RecentLocation` for recent jumps

Generated PowerShell init output SHALL preserve short interactive spellings by installing aliases to those primary functions.

#### Scenario: Module exposes primary PowerShell command names
- **WHEN** `dx init pwsh` output is evaluated
- **THEN** `Get-Command -Module dx` SHALL show primary functions named `Set-DxLocation`, `Step-Up`, `Undo-Location`, `Redo-Location`, `Set-FrecentLocation`, and `Set-RecentLocation`

#### Scenario: Short names remain aliases
- **WHEN** `dx init pwsh` output is evaluated
- **THEN** the command `up` SHALL be an alias for `Step-Up`
- **AND** `back` and `cd-` SHALL be aliases for `Undo-Location`
- **AND** `forward` and `cd+` SHALL be aliases for `Redo-Location`
- **AND** `cdf` and `z` SHALL be aliases for `Set-FrecentLocation`
- **AND** `cdr` SHALL be an alias for `Set-RecentLocation`

#### Scenario: Dot-dot alias steps up
- **WHEN** `dx init pwsh` output is evaluated
- **THEN** `..` SHALL be an alias for `Step-Up`

#### Scenario: Alias behavior remains unchanged
- **WHEN** a user invokes a short alias such as `back`, `forward`, `cdf`, `cdr`, `up`, `cd-`, `cd+`, or `z`
- **THEN** the generated PowerShell hook SHALL perform the same navigation behavior as before the primary function rename

### Requirement: PowerShell Module Cleanup Covers Navigation Aliases
Generated PowerShell init output SHALL capture prior alias targets for every navigation alias it installs.

When `Remove-Module dx` is invoked, the module cleanup handler SHALL restore prior alias targets where they existed before import and SHALL remove dx-installed aliases that had no prior target.

The cleanup set SHALL include `cd`, `up`, `..`, `back`, `forward`, `cd-`, `cd+`, `cdf`, `cdr`, and `z`.

#### Scenario: Remove module restores prior dot-dot alias
- **WHEN** a PowerShell session has an existing `..` alias before evaluating `dx init pwsh` output
- **AND** the generated output is evaluated and then `Remove-Module dx` is invoked
- **THEN** the prior `..` alias target SHALL be restored where feasible

#### Scenario: Remove module removes dx-created aliases
- **WHEN** a PowerShell session has no prior alias for `up`, `back`, `forward`, `cdf`, `cdr`, or `..`
- **AND** generated `dx init pwsh` output installs those aliases
- **AND** `Remove-Module dx` is invoked
- **THEN** those dx-created aliases SHALL be removed where feasible
