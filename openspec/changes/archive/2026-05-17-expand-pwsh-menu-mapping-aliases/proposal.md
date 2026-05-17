## Why

PowerShell users often invoke filesystem-oriented commands through aliases such as `gci`, `dir`, `ii`, or custom aliases like `open`. `DX_MENU_COMMAND_MAPPINGS` currently matches only the literal first token, so users must duplicate mappings for every alias even when PowerShell already knows those aliases point at the same command.

## What Changes

- Expand PowerShell mapped-command registrations from configured command names to aliases whose `Definition` matches those commands.
- Use one-way alias lookup: users configure the canonical command they want to support, and aliases of that command are added automatically.
- Keep the configured command mapped directly even when no aliases exist.
- Freeze the expanded PowerShell mapping set when generated hooks are loaded; aliases created later require reloading the hooks.
- Leave Bash, Zsh, Fish, and `dx menu` runtime candidate behavior unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `menu-command-mappings`: PowerShell mapped commands also match aliases of configured command definitions.

## Impact

- Affects generated PowerShell hook code in `src/hooks/pwsh.rs` and shared mapping rendering in `src/hooks/common.rs`.
- Requires tests for generated PowerShell hook output and mapping precedence rules.
- No CLI flag or environment schema change is expected.
