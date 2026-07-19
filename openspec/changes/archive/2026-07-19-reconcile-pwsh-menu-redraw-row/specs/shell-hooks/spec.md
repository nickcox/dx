## MODIFIED Requirements

### Requirement: Menu Action Boundary and Native Fallback
When menu mode is enabled, hooks SHALL treat stdout from `dx menu` as a structured action payload channel.

Successful replace actions SHALL update the shell buffer using the returned `value`.

Shells with direct buffer-editing APIs (Zsh, Fish, and PowerShell) SHALL apply `replaceStart`, `replaceEnd`, and `value` exactly to the shell buffer.

Bash SHALL validate the replace payload fields, but MAY rely on Readline completion semantics to insert `value` rather than directly applying `replaceStart` and `replaceEnd` itself.

Explicit `cancel` actions SHALL leave the shell buffer unchanged and SHALL NOT trigger native completion fallback.

`noop`, invalid payloads, non-replace actions where replacement is required other than explicit cancel, command failure, no candidates, and non-interactive execution paths SHALL all fall back to native completion behavior for the current shell.

POSIX hooks SHALL keep deterministic dependency-free payload validation. PowerShell SHALL continue structured parsing via `ConvertFrom-Json` and native completion fallback via PSReadLine.

For successful replace actions, shell hooks SHALL use the action payload's `terminal` value to determine whether to redraw or repaint the prompt after applying the replacement. `terminal=clean` SHALL skip prompt redraw. `terminal=dirty` SHALL redraw or repaint the prompt after replacement.

PowerShell SHALL require integer `redrawRow` and `scrollRows` geometry for dirty interactive replacement and cancellation actions. It SHALL use the geometry to invoke PSReadLine prompt redraw at an explicit post-scroll Y coordinate rather than using the no-argument redraw path with cached pre-scroll coordinates. Bash, Zsh, and Fish MAY ignore the geometry fields.

If `terminal` is absent or not exactly `clean` or `dirty`, hooks SHALL treat a replace action as invalid and fall back to native completion behavior. PowerShell SHALL also reject dirty replace geometry that is missing, non-integer, negative, outside the console, or inconsistent with the pre-menu cursor position. Invalid dirty cancellation geometry SHALL leave the buffer unchanged and use the safest available prompt redraw fallback without triggering native completion.

#### Scenario: Replace action updates the shell buffer
- **WHEN** Zsh, Fish, or PowerShell receives a `replace` action with `replaceStart`, `replaceEnd`, and `value`
- **THEN** the hook SHALL replace exactly that span in the shell buffer with the returned value

#### Scenario: Clean replace skips prompt redraw
- **WHEN** Zsh, Fish, or PowerShell receives a valid `replace` action with `terminal=clean`
- **THEN** the hook SHALL apply the replacement without invoking its prompt redraw or repaint function

#### Scenario: Dirty replace refreshes prompt
- **WHEN** Zsh or Fish receives a valid `replace` action with `terminal=dirty`
- **THEN** the hook SHALL apply the replacement and invoke its prompt redraw or repaint function

#### Scenario: PowerShell dirty replace uses explicit redraw row
- **WHEN** PowerShell receives a valid dirty replacement with post-scroll geometry
- **THEN** the hook SHALL apply the replacement and invoke PSReadLine prompt redraw with an explicit Y coordinate reconciled from the pre-menu console position and returned geometry

#### Scenario: PowerShell dirty cancellation uses explicit redraw row
- **WHEN** PowerShell receives a valid dirty cancellation with post-scroll geometry
- **THEN** the hook SHALL leave the buffer unchanged and invoke PSReadLine prompt redraw with the reconciled explicit Y coordinate without native completion fallback

#### Scenario: PowerShell redraw preserves wrapped or multiline input
- **WHEN** a dirty PowerShell menu outcome occurs while the prompt or input spans multiple physical rows
- **THEN** prompt reconciliation SHALL preserve the complete PSReadLine buffer and logical cursor position without accumulating blank menu-reservation rows

#### Scenario: Invalid terminal field falls back natively
- **WHEN** Zsh, Fish, or PowerShell receives a `replace` action without a valid `terminal` value
- **THEN** the hook SHALL invoke native completion fallback instead of applying a replacement

#### Scenario: Invalid PowerShell dirty replace geometry falls back natively
- **WHEN** PowerShell receives `terminal=dirty` replacement data with invalid redraw geometry
- **THEN** the hook SHALL invoke native completion fallback instead of applying the replacement

#### Scenario: Bash uses value-oriented completion insertion
- **WHEN** Bash receives a valid `replace` action from `dx menu`
- **THEN** the hook SHALL validate the returned range fields and use the returned `value` as the completion insertion payload, relying on Readline to perform the replacement

#### Scenario: Cancel leaves shell buffer unchanged without native fallback
- **WHEN** `dx menu` returns a valid explicit cancel action
- **THEN** the hook SHALL leave the current buffer content unchanged and SHALL NOT invoke native completion fallback

#### Scenario: Noop or invalid payload falls back natively
- **WHEN** `dx menu` returns `noop`, invalid payload data, or exits non-zero
- **THEN** the hook SHALL invoke the shell's native completion fallback instead of applying a replacement

#### Scenario: PowerShell menu fallback remains PSReadLine-native for noop/error paths
- **WHEN** menu-enabled PowerShell hook execution receives `noop`, invalid JSON, missing JSON, or a non-`replace` action other than explicit `cancel`
- **THEN** the hook SHALL parse payloads with `ConvertFrom-Json` when present and SHALL fall back via PSReadLine native completion behavior
