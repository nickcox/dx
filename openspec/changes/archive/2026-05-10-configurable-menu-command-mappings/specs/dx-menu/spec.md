## ADDED Requirements

### Requirement: Explicit Mode Invocation for Mapped Commands
When menu mode is active, `dx menu` SHALL support explicit mapped-command invocation via `--mode <mode>` in addition to built-in dx navigation command mappings.

For mapped external commands, `dx menu` SHALL use the explicit mode argument instead of consulting runtime mapping configuration.

If no explicit mode is provided and the command at the cursor is not a built-in supported context, `dx menu` SHALL return `noop`.

#### Scenario: Explicit path mode invokes mapped command handling
- **WHEN** menu mode is active, `dx menu --mode path` is invoked for buffer command `ls`
- **THEN** `dx menu` SHALL build candidates using mapped mode `path`

#### Scenario: Unmapped non-dx command remains noop
- **WHEN** menu mode is active, no explicit mode is provided, and buffer command is `git`
- **THEN** `dx menu` SHALL return `{ "action": "noop" }`

### Requirement: Mapped Command Candidate Dispatch by Mode
For explicit mapped-command invocations, `dx menu` SHALL dispatch candidate sourcing according to the requested mode (`path`, `directory`, `file`) using `dx-smart` candidate behavior.

If no candidate can produce a replace action, `dx menu` SHALL return `noop` so shell hooks can apply native fallback.

#### Scenario: File mode mapped command offers file candidates
- **WHEN** `dx menu --mode file` is invoked and menu context is `cat re`
- **THEN** `dx menu` SHALL produce candidates filtered to files for the active token

#### Scenario: No candidate for mapped command returns noop
- **WHEN** mapped command context produces no valid candidates for the active token
- **THEN** `dx menu` SHALL return `{ "action": "noop" }`
