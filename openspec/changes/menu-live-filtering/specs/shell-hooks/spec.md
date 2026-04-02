## ADDED Requirements

### Requirement: Menu Filtering Final-Action Application
Shell hooks that integrate `dx menu` SHALL continue treating menu filtering as internal interaction and SHALL apply shell buffer changes only from the single final JSON action returned by `dx menu`.

This final action MAY be either a candidate-selection replacement or a typed-filter persistence replacement emitted on cancel after query edits.

#### Scenario: Cancel with typed edits applies returned replace action
- **WHEN** a user opens menu completion, types additional filter characters, and cancels, and `dx menu` returns a `replace` action
- **THEN** the shell hook SHALL apply that returned replacement to the command buffer

#### Scenario: Cancel without typed edits remains noop
- **WHEN** `dx menu` returns `{ "action": "noop" }` after cancel with no query edits
- **THEN** the shell hook SHALL leave the command buffer unchanged

#### Scenario: Final replace action updates shell buffer once
- **WHEN** `dx menu` returns a `replace` action after filtered selection
- **THEN** the shell hook SHALL apply exactly the returned replacement range/value to the current command buffer
