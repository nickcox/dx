## ADDED Requirements

### Requirement: Configurable PowerShell Menu Key Binding
When `dx init pwsh --menu` is invoked, generated PowerShell hooks SHALL bind the dx menu PSReadLine handler to the key named by `DX_PWSH_MENU_KEY`.

If `DX_PWSH_MENU_KEY` is unset or empty, generated PowerShell hooks SHALL bind the handler to `Tab`.

The configured key value SHALL be captured when hook output is generated. Changing `DX_PWSH_MENU_KEY` after hook generation SHALL NOT alter the active binding until `dx init pwsh --menu` is run again and the regenerated hooks are loaded.

#### Scenario: Default PowerShell menu key remains Tab
- **WHEN** `DX_PWSH_MENU_KEY` is unset and `dx init pwsh --menu` is invoked
- **THEN** generated PowerShell output SHALL contain `Set-PSReadLineKeyHandler -Key Tab`

#### Scenario: Custom PowerShell menu key is emitted
- **WHEN** `DX_PWSH_MENU_KEY="F12"` and `dx init pwsh --menu` is invoked
- **THEN** generated PowerShell output SHALL contain `Set-PSReadLineKeyHandler -Key F12`

#### Scenario: Empty PowerShell menu key falls back to Tab
- **WHEN** `DX_PWSH_MENU_KEY` is set to an empty or whitespace-only value and `dx init pwsh --menu` is invoked
- **THEN** generated PowerShell output SHALL contain `Set-PSReadLineKeyHandler -Key Tab`

#### Scenario: PowerShell menu key changes require re-init
- **WHEN** hooks were generated with `DX_PWSH_MENU_KEY="F12"` and the environment later changes to `DX_PWSH_MENU_KEY="Tab"` without re-running `dx init pwsh --menu`
- **THEN** the existing hook behavior SHALL continue using the previously generated `F12` binding

### Requirement: Invalid PowerShell Menu Key Values Fail Init Generation
If `DX_PWSH_MENU_KEY` contains a value that cannot be safely emitted into generated PowerShell hook code, `dx init pwsh --menu` SHALL fail rather than emitting a malformed menu binding.

#### Scenario: Unsafe key value prevents hook emission
- **WHEN** `DX_PWSH_MENU_KEY` contains a newline or quote character and `dx init pwsh --menu` is invoked
- **THEN** init generation SHALL fail and SHALL NOT emit a partial PowerShell menu handler

### Requirement: PowerShell Menu Key Configuration Is Menu-Scoped
`DX_PWSH_MENU_KEY` SHALL affect only menu-enabled PowerShell hook generation.

PowerShell init without `--menu` SHALL NOT emit a PSReadLine menu handler regardless of `DX_PWSH_MENU_KEY`.

Bash, Zsh, and Fish hook generation SHALL ignore `DX_PWSH_MENU_KEY`.

#### Scenario: Non-menu PowerShell init ignores configured menu key
- **WHEN** `DX_PWSH_MENU_KEY="F12"` and `dx init pwsh` is invoked without `--menu`
- **THEN** generated PowerShell output SHALL NOT contain `Set-PSReadLineKeyHandler`

#### Scenario: POSIX shell init ignores PowerShell menu key
- **WHEN** `DX_PWSH_MENU_KEY="F12"` and `dx init bash --menu` is invoked
- **THEN** generated Bash output SHALL NOT include the configured PowerShell key value

### Requirement: PowerShell Menu Fallback Preserves Previous Key Function
The generated PowerShell menu hook SHALL capture the configured key's previous PSReadLine function before registering dx's menu handler.

For disabled menu, non-matching command, noop, error, invalid JSON, missing JSON, and non-`replace` action paths, the generated handler SHALL invoke the captured previous PSReadLine function when it can be safely mapped to a public PSReadLine function.

If no previous function is available or the previous function cannot be safely replayed, the generated handler SHALL fall back via `TabCompleteNext`.

#### Scenario: Prior MenuComplete binding is preserved on fallback
- **WHEN** the configured key is bound to PSReadLine `MenuComplete` before `dx init pwsh --menu` output is evaluated
- **THEN** generated hook behavior SHALL invoke `MenuComplete` for native fallback paths

#### Scenario: Prior TabCompleteNext binding is preserved on fallback
- **WHEN** the configured key is bound to PSReadLine `TabCompleteNext` before `dx init pwsh --menu` output is evaluated
- **THEN** generated hook behavior SHALL invoke `TabCompleteNext` for native fallback paths

#### Scenario: Unsupported prior custom action falls back safely
- **WHEN** the configured key is bound to a user-defined PSReadLine scriptblock before `dx init pwsh --menu` output is evaluated
- **THEN** generated hook behavior SHALL use `TabCompleteNext` for native fallback paths instead of attempting to replay the unavailable scriptblock

#### Scenario: Custom menu key still preserves native fallback helper
- **WHEN** `DX_PWSH_MENU_KEY="F12"` and `dx init pwsh --menu` is invoked
- **THEN** generated PowerShell output SHALL bind the handler to `F12` and SHALL include logic to capture and invoke the previous key function for fallback paths

#### Scenario: Re-evaluating hooks preserves original fallback function
- **WHEN** generated `dx init pwsh --menu` output is evaluated more than once in the same PowerShell session
- **THEN** the generated hook SHALL NOT capture its own dx menu handler as the previous key function

#### Scenario: Changing configured key removes prior dx binding
- **WHEN** generated hooks were loaded with `DX_PWSH_MENU_KEY="F11"` and are then regenerated and loaded with `DX_PWSH_MENU_KEY="F12"` in the same PowerShell session
- **THEN** the previous `F11` dx menu binding SHALL be removed or restored to its original replayable PSReadLine function

### Requirement: PowerShell CustomAction Overwrite Warning
When generated PowerShell menu hooks replace a configured key whose previous PSReadLine function is `CustomAction`, the generated hook code SHALL emit a warning to stderr during hook evaluation.

The warning SHALL identify the configured key and explain that dx cannot replay the previous custom handler, so native fallback will use `TabCompleteNext`.

The warning SHALL NOT be emitted on every keypress.

#### Scenario: CustomAction overwrite warns during init evaluation
- **WHEN** the configured key is bound to a user-defined PSReadLine scriptblock and generated `dx init pwsh --menu` output is evaluated
- **THEN** the generated hook SHALL write a warning to stderr before replacing the key binding

#### Scenario: Built-in previous function does not warn
- **WHEN** the configured key is bound to PSReadLine `MenuComplete` before generated `dx init pwsh --menu` output is evaluated
- **THEN** the generated hook SHALL NOT emit the CustomAction overwrite warning
