## Why

`dx menu` currently handles only built-in dx navigation command contexts, so users cannot reuse the same interactive menu experience for common external commands (for example `ls`, `open`, and `cat`). Adding configurable command-to-mode mappings at shell-init time enables one consistent completion UX while preserving existing shell-native fallback behavior and keeping `dx menu` free of runtime mapping-policy concerns.

## What Changes

- Add environment-variable-driven command mappings consumed by `dx init <shell> --menu` when hooks are generated.
- Introduce explicit completion modes for mapped commands: `path`, `directory`, and `file`.
- Make generated shell hooks capture mapped command→mode bindings at init time and invoke `dx menu` with an explicit mode for matched mapped commands.
- Use `dx-smart` resolution for all mapped external commands.
- Scope v1 mapped behavior to the current token under the cursor; no command-specific multi-argument parsing in this phase.
- Require re-running `dx init` after mapping changes so regenerated hooks capture the new mapping set.
- Keep an extensible init-time mapping source boundary so a future config-file layer can be added without making `dx menu` read configuration at runtime.

## Capabilities

### New Capabilities
- `menu-command-mappings`: Configurable init-time command mappings and mode semantics for external command completion.

### Modified Capabilities
- `dx-menu`: Add explicit mode-directed handling and token-scoped replacement behavior for mapped external command invocations.
- `shell-hooks`: Add shell-specific registration behavior for mapped commands under menu-enabled init output.

## Impact

- Affected areas: `src/menu/*` (explicit mode dispatch, file-aware candidate sourcing, token replacement), shell init generation modules/templates, and completion/menu integration tests.
- Behavioral impact: menu mode can drive completions for configured external commands while preserving native fallback on noop/error paths.
- Configuration impact: introduces env-var schema for command-to-mode mappings consumed at init time; mapping changes take effect only after re-running `dx init`.
