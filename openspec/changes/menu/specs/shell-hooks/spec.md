## ADDED Requirements

### Requirement: Menu Integration Toggle
Generated shell hooks SHALL support an opt-in toggle for menu integration. By default, existing completion and wrapper behavior SHALL remain unchanged unless menu integration is explicitly enabled.

The enablement mechanism SHALL be available at shell init/runtime through a documented configuration path (for example environment variable and/or init flag).

When disabled, hook scripts SHALL not invoke `dx menu` during normal completion flows.

#### Scenario: Menu disabled preserves existing behavior
- **WHEN** shell hooks are loaded with menu integration disabled
- **THEN** completions and navigation wrappers SHALL use current non-menu behavior and SHALL not call `dx menu`

#### Scenario: Menu enabled allows invocation
- **WHEN** shell hooks are loaded with menu integration enabled
- **THEN** configured keybindings or helper functions SHALL invoke `dx menu` for supported dx navigation contexts

### Requirement: Buffer Context Passing
When invoking `dx menu`, shell hooks SHALL pass the current command line buffer text, cursor position, current working directory, and session identity so menu logic can determine mode and replacement bounds.

Hook integrations SHALL pass session identity via `DX_SESSION` (or equivalent shell environment variable projection).

#### Scenario: Hook passes full context to dx menu
- **WHEN** the user triggers menu on buffer `cd pr` with cursor at token end
- **THEN** the hook SHALL invoke `dx menu` with buffer, cursor, cwd, and session inputs matching shell state

#### Scenario: Missing session in shell environment
- **WHEN** menu is triggered and `DX_SESSION` is unset
- **THEN** hook behavior SHALL follow current session resolution defaults and still invoke `dx menu` with available context

### Requirement: Completion-Context TTY Wiring
In completion contexts where stdin is non-interactive, shell hooks SHALL invoke `dx menu` with stdin redirected from the controlling TTY (for example `</dev/tty`) while keeping stdout captured for JSON parsing.

Shell hooks SHALL NOT treat captured stdout as a reason to bypass menu invocation.

#### Scenario: Hook uses /dev/tty for menu stdin
- **WHEN** zsh or bash invokes `dx menu` from completion function via command substitution
- **THEN** hook invocation SHALL redirect stdin from `/dev/tty` so interactive key input is available

#### Scenario: JSON action still parsed from captured stdout
- **WHEN** `dx menu` returns a `replace` action while stdout is captured
- **THEN** hook logic SHALL parse the JSON payload and apply replacement behavior

### Requirement: Menu Action Application
Shell hooks SHALL parse `dx menu` JSON output and apply actions deterministically:

- `action=replace`: update only the indicated buffer range with `value` and place cursor at end of replacement.
- `action=noop`: leave buffer and cursor unchanged.

If JSON parsing fails or action payload is invalid, hooks SHALL leave buffer unchanged and fall back to existing non-menu behavior.

#### Scenario: Replace action updates command line
- **WHEN** `dx menu` returns `{ "action": "replace", "replaceStart": 3, "replaceEnd": 5, "value": "/tmp" }` for buffer `cd pr`
- **THEN** shell hook SHALL update command line to `cd /tmp` and move cursor to end of `/tmp`

#### Scenario: Noop action keeps buffer
- **WHEN** `dx menu` returns `{ "action": "noop" }`
- **THEN** shell hook SHALL keep command line and cursor unchanged

#### Scenario: Invalid action falls back safely
- **WHEN** `dx menu` returns malformed JSON
- **THEN** shell hook SHALL not mutate command buffer and SHALL use existing completion behavior

### Requirement: Mode-Scoped Invocation
Shell hooks SHALL invoke `dx menu` only for supported dx navigation command contexts:

- `cd`, `up`, `cdf`, `z`, `cdr`, `back`, `forward`, `cd-`, `cd+`

For non-dx commands, menu integration SHALL not intercept behavior.

#### Scenario: Supported command invokes menu
- **WHEN** user triggers menu while editing `cdf proj`
- **THEN** hook SHALL invoke `dx menu`

#### Scenario: Non-dx command bypasses menu
- **WHEN** user triggers completion while editing `git checkout`
- **THEN** hook SHALL bypass `dx menu` and preserve native shell behavior

### Requirement: Cancellation and Non-TTY Fallback
When `dx menu` returns `noop` (including cancellation or non-interactive fallback), shell hooks SHALL gracefully continue with prior behavior and SHALL NOT emit errors.

If `dx menu` exits non-zero, shell hooks SHALL not mutate command buffer and SHALL use existing completion behavior.

#### Scenario: User cancels menu
- **WHEN** menu is invoked and user cancels
- **THEN** hook SHALL observe noop and preserve existing buffer content without errors

#### Scenario: Menu command fails
- **WHEN** menu invocation exits non-zero
- **THEN** hook SHALL preserve existing buffer and use non-menu completion flow
