## MODIFIED Requirements

### Requirement: command_not_found Handler — Path-Like Heuristic
When the opt-in `command_not_found` handler receives an unrecognized command, it SHALL only invoke `dx resolve` if the input matches a path-like heuristic: contains `/`, starts with `.` or `~`, matches a multi-dot pattern (e.g., `...`, `....`), contains a supported word delimiter (`.`, `_`, `-`), or contains an in-segment doubled-period sequence (`..`) that is not a pure multi-dot step-up token. If the input does not match the heuristic, the handler SHALL immediately produce the shell's standard "command not found" error without invoking dx.

#### Scenario: Slash-containing input triggers resolve
- **WHEN** the command `pr/dx` is not found and the command_not_found handler is active
- **THEN** the handler SHALL invoke `dx resolve "pr/dx"`

#### Scenario: Dot-prefixed input triggers resolve
- **WHEN** the command `./foo` is not found and the command_not_found handler is active
- **THEN** the handler SHALL invoke `dx resolve "./foo"`

#### Scenario: Multi-dot input triggers resolve
- **WHEN** the command `...` is not found and the command_not_found handler is active
- **THEN** the handler SHALL invoke `dx resolve "..."`

#### Scenario: Delimiter-shortened input triggers resolve
- **WHEN** the command `cd-e` is not found and the command_not_found handler is active
- **THEN** the handler SHALL invoke `dx resolve "cd-e"`

#### Scenario: Doubled-period shortening input triggers resolve
- **WHEN** the command `p..shell` is not found and the command_not_found handler is active
- **THEN** the handler SHALL invoke `dx resolve "p..shell"`

#### Scenario: Plain word does not trigger resolve
- **WHEN** the command `gti` is not found and the command_not_found handler is active
- **THEN** the handler SHALL NOT invoke `dx resolve` and SHALL produce the standard "command not found" error

### Requirement: Fish Auto-cd Cooperation
The Fish hook code SHALL cooperate with Fish's built-in auto-cd feature. The `fish_command_not_found` handler (when enabled) SHALL only attempt `dx resolve` for inputs that Fish's native auto-cd would not handle (abbreviated paths, delimiter-shortened paths, multi-dot patterns, or bookmark names). If the input is a literal existing directory, Fish's auto-cd SHALL take precedence.

#### Scenario: Fish auto-cd handles literal directory
- **WHEN** a user types a literal directory name that exists on disk in a Fish shell with dx hooks
- **THEN** Fish's native auto-cd SHALL handle the navigation (the dx command_not_found handler is never reached)

#### Scenario: Fish handler resolves abbreviated path
- **WHEN** a user types an abbreviated path like `pr/dx` that is not a literal directory, and the Fish command_not_found handler is active
- **THEN** the handler SHALL invoke `dx resolve "pr/dx"` and navigate on success

#### Scenario: Fish handler resolves delimiter-shortened path
- **WHEN** a user types a delimiter-shortened query like `cd-e` that is not a literal directory, and the Fish command_not_found handler is active
- **THEN** the handler SHALL invoke `dx resolve "cd-e"` and navigate on success
