## 1. PowerShell Mapping Expansion

- [x] 1.1 Replace the generated PowerShell mapped-command array with seed mappings plus a hook-load lookup table.
- [x] 1.2 Add one-way alias expansion using `Get-Alias -Definition <configured command>` while preserving direct configured command mappings.
- [x] 1.3 Enforce precedence so explicit configured command mappings override derived alias mappings and first derived alias mapping wins on derived collisions.
- [x] 1.4 Update the PowerShell menu key handler to use the expanded lookup table for first-token mode routing.

## 2. Tests And Validation

- [x] 2.1 Add Rust tests for generated PowerShell hook structure, including `Get-Alias -Definition` alias expansion and lookup-table routing.
- [x] 2.2 Add tests covering explicit mapping precedence over derived alias mappings.
- [x] 2.3 Confirm Bash, Zsh, and Fish generated mappings remain literal command registrations only.
- [x] 2.4 Run the relevant menu/init hook tests and full test suite if feasible.
