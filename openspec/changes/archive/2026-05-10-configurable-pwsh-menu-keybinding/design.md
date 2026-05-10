## Context

PowerShell menu integration currently uses `Set-PSReadLineKeyHandler -Key Tab` in generated `dx init pwsh --menu` output. That is necessary for the interactive menu path because `Register-ArgumentCompleter` returns completion candidates, while `dx menu` needs full buffer/cursor access and direct buffer replacement through PSReadLine APIs.

The downside is that the Tab binding is global. Although dx falls back with `TabCompleteNext` for non-matching commands and noop/error paths, installing the handler can still replace a user's existing Tab binding. A configurable key plus previous-function fallback lets users keep the interactive menu while preserving the behavior the configured key had before `dx init pwsh --menu` was evaluated, when PSReadLine exposes that behavior as a named function.

## Goals / Non-Goals

**Goals:**
- Keep `Tab` as the default PowerShell menu key for backward compatibility.
- Allow users to configure the PowerShell menu key before running `dx init pwsh --menu`.
- Capture the configured key in generated hook output at init time.
- Preserve the configured key's previous PSReadLine function for native fallback when possible.
- Document configuration and re-init requirements.

**Non-Goals:**
- Replace PowerShell menu mode with command-scoped `Register-ArgumentCompleter` behavior.
- Add configurable keybindings for Bash, Zsh, or Fish in this change.
- Support multiple PowerShell menu keybindings at once.
- Preserve or chain arbitrary previous user-defined scriptblock handlers for the same key.

## Decisions

### 1) Use an init-time environment variable

Add a PowerShell-specific environment variable consumed during hook generation:

- `DX_PWSH_MENU_KEY=<PSReadLine key name>`
- Default: `Tab`

`dx init pwsh --menu` reads this value and emits `Set-PSReadLineKeyHandler -Key <value> -ScriptBlock { ... }`.

Rationale: this matches existing init-time mapping behavior for menu command mappings. The generated hook is the source of shell behavior, and changing the env var requires regenerating/reloading hooks.

Alternatives considered:
- Runtime lookup inside the generated handler: rejected because the handler is already bound to a key; changing the env var after registration cannot move the binding without re-registering.
- CLI flag on `dx init`: viable later, but env var keeps profile configuration simple and consistent with current menu configuration style.
- Generic `DX_MENU_KEY`: rejected for this change because key syntax and handler semantics are shell-specific.

### 2) Capture and replay the previous PSReadLine function for fallback

Before installing dx's handler, generated PowerShell code should call `Get-PSReadLineKeyHandler -Chord <configured-key>` and capture the returned `Function` name. The dx handler should route every native-fallback path through a small generated helper that invokes `[Microsoft.PowerShell.PSConsoleReadLine]::<Function>($key, $arg)` when the function name is recognized as a safe public PSReadLine function.

Examples:
- If the previous Tab function is `MenuComplete`, dx fallback invokes `MenuComplete`.
- If the previous Tab function is `Complete`, dx fallback invokes `Complete`.
- If the previous Tab function is `TabCompleteNext`, dx fallback invokes `TabCompleteNext`.

If no previous binding exists, or if the previous function is `CustomAction` or another value that cannot be safely mapped to a public PSReadLine static method, fallback should use `TabCompleteNext`.

When the previous function is `CustomAction`, generated hook evaluation should emit a warning to stderr before replacing the key binding. This warning should name the key and explain that dx cannot replay the previous custom handler, so fallback will use `TabCompleteNext`. The warning should be emitted at hook-load time, not on every keypress.

Rationale: fallback should mean "do what this key would have done before dx took it over". `Get-PSReadLineKeyHandler` exposes built-in bindings as named functions but does not expose the body of custom scriptblock handlers, so built-in functions can be preserved while arbitrary custom handlers cannot be replayed safely.

Alternatives considered:
- Always use `TabCompleteNext`: rejected because it loses the user's prior Tab/MenuComplete behavior.
- Add user-configured fallback function env var: rejected for now because PSReadLine can discover the common built-in case automatically.
- Preserve arbitrary custom scriptblocks: rejected because `Get-PSReadLineKeyHandler` exposes `Function=CustomAction` metadata but not the original scriptblock body through the public object.

### 3) Validate minimally but fail generation on unusable values

`DX_PWSH_MENU_KEY` should be trimmed. Unset or empty values resolve to `Tab`. Values containing newlines or quote characters should be rejected to avoid emitting malformed PowerShell. Invalid values should make `dx init pwsh --menu` fail rather than generating a broken handler.

Rationale: dx cannot fully validate every PSReadLine key chord without depending on PSReadLine at generation time, but it can prevent obvious script-generation hazards.

Alternatives considered:
- Pass all values through unvalidated: rejected because generated PowerShell string interpolation would become fragile.
- Maintain a full allowlist of PSReadLine key names: rejected because PSReadLine supports many key forms and may vary by version/platform.

### 4) Scope configuration to PowerShell menu mode only

The setting is used only when shell is `pwsh` and `--menu` is enabled. Non-menu PowerShell init remains unchanged, and POSIX shells ignore this setting.

Rationale: current concern is specifically PSReadLine global Tab interception. Other shells have different keybinding/completion models and should be handled separately if needed.

## Risks / Trade-offs

- **[Risk] User chooses a PSReadLine key name that is syntactically safe but not accepted by PSReadLine** -> Mitigation: generated hook evaluation fails visibly in PowerShell; docs should recommend common values and mention PSReadLine key syntax.
- **[Risk] Warning on CustomAction could be noisy in profiles loaded often** -> Mitigation: emit only once per hook evaluation, only when replacing a prior `CustomAction`, and keep the message concise/actionable.
- **[Risk] Moving dx off Tab can make mapped command menu behavior less discoverable** -> Mitigation: keep Tab default and document explicit examples for custom keys.
- **[Trade-off] Previous built-in key functions can be replayed, but previous custom scriptblock handlers cannot** -> Mitigation: fallback to `TabCompleteNext` for `CustomAction`; users with custom scriptblock handlers can configure dx onto a different key.
- **[Trade-off] Previous bindings for the configured key are still replaced** -> Mitigation: fallback preserves built-in behavior when dx declines to handle the key, but the actual registered handler is still dx's PSReadLine handler.
- **[Trade-off] Env var changes require re-init** -> Mitigation: document the requirement consistently with `DX_MENU_COMMAND_MAPPINGS`.

## Migration Plan

1. Add PowerShell menu key parsing for `DX_PWSH_MENU_KEY` in the init-generation path.
2. Thread the resolved key into PowerShell hook generation.
3. Replace hard-coded `-Key Tab` with generated key output.
4. Add generated PowerShell helper logic that captures the prior key function and invokes it for native fallback paths when it maps to a supported PSReadLine function.
5. Add generated PowerShell warning logic for prior `CustomAction` bindings.
6. Add tests for default `Tab`, custom key output, empty value fallback, invalid value failure, unchanged non-menu output, previous `MenuComplete` fallback capture, `CustomAction` warning, and fallback defaulting for unsupported previous functions.
7. Update docs with configuration examples, previous-function fallback behavior, warning behavior, and re-init guidance.

Rollback strategy: unset `DX_PWSH_MENU_KEY`, re-run `dx init pwsh --menu`, and reload the generated hooks to return to Tab.

## Answered Questions

- Q: Which PSReadLine key chord should be used as the primary documentation example?
  A: Use `F12`, because it is easy to validate in generated hooks and avoids ambiguity in PSReadLine chord syntax.
