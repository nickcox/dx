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

If `terminal` is absent or not exactly `clean` or `dirty`, shell hooks SHALL treat the action payload as invalid and fall back to native completion behavior.

#### Scenario: Replace action updates the shell buffer
- **WHEN** Zsh, Fish, or PowerShell receives a `replace` action with `replaceStart`, `replaceEnd`, and `value`
- **THEN** the hook SHALL replace exactly that span in the shell buffer with the returned value

#### Scenario: Clean replace skips prompt redraw
- **WHEN** Zsh, Fish, or PowerShell receives a valid `replace` action with `terminal=clean`
- **THEN** the hook SHALL apply the replacement without invoking its prompt redraw or repaint function

#### Scenario: Dirty replace refreshes prompt
- **WHEN** Zsh, Fish, or PowerShell receives a valid `replace` action with `terminal=dirty`
- **THEN** the hook SHALL apply the replacement and invoke its prompt redraw or repaint function

#### Scenario: Invalid terminal field falls back natively
- **WHEN** Zsh, Fish, or PowerShell receives a `replace` action without a valid `terminal` value
- **THEN** the hook SHALL invoke native completion fallback instead of applying a replacement

#### Scenario: Bash uses value-oriented completion insertion
- **WHEN** Bash receives a valid `replace` action from `dx menu`
- **THEN** the hook SHALL validate the returned range fields and use the returned `value` as the completion insertion payload, relying on Readline to perform the replacement

#### Scenario: Cancel leaves shell buffer unchanged without native fallback
- **WHEN** `dx menu` returns `{ "action": "cancel" }`
- **THEN** the hook SHALL leave the current buffer content unchanged and SHALL NOT invoke native completion fallback

#### Scenario: Noop or invalid payload falls back natively
- **WHEN** `dx menu` returns `noop`, invalid payload data, or exits non-zero
- **THEN** the hook SHALL invoke the shell's native completion fallback instead of applying a replacement

#### Scenario: PowerShell menu fallback remains PSReadLine-native for noop/error paths
- **WHEN** menu-enabled PowerShell hook execution receives `noop`, invalid JSON, missing JSON, or a non-`replace` action other than explicit `cancel`
- **THEN** the hook SHALL parse payloads with `ConvertFrom-Json` when present and SHALL fall back via PSReadLine native completion behavior
