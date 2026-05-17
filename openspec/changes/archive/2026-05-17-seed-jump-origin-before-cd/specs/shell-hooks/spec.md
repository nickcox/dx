## ADDED Requirements

### Requirement: Jump Wrappers Seed Origin Before Directory Change
Generated jump wrappers for `cdf`, `z`, and `cdr` SHALL record the current shell working directory in the session stack before changing to a resolved jump target.

After a successful jump directory change, generated jump wrappers SHALL record the destination working directory in the session stack.

If no jump target is resolved or the directory change fails, generated jump wrappers SHALL NOT record the destination as a successful navigation.

Generated stack traversal wrappers for `back`, `forward`, `cd-`, and `cd+` SHALL NOT seed a new origin before performing undo/redo traversal.

#### Scenario: First jump in fresh session can be undone
- **WHEN** generated hooks are loaded in a fresh session whose stack has no current directory
- **AND** the user runs `z project` from `/start` and the jump target resolves to `/project`
- **THEN** the hook SHALL record `/start` before changing directory
- **AND** after the successful change, the hook SHALL record `/project` as the current stack directory
- **AND** a subsequent `cd-` SHALL have `/start` available as an undo destination

#### Scenario: Failed jump does not record destination
- **WHEN** a generated jump wrapper cannot resolve a target or fails to change directory
- **THEN** the hook SHALL NOT record the failed destination as the current stack directory

#### Scenario: Stack traversal does not seed new origin
- **WHEN** the user runs `cd-` or `cd+`
- **THEN** generated hooks SHALL delegate to stack undo/redo behavior without pre-pushing the current directory as a new navigation origin
