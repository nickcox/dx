## Why

`dx` currently relies on shell-native completion lists, which work but provide inconsistent UX and limited in-line filtering across Bash, Zsh, and Fish. A first-class interactive menu mode is needed to make directory selection fast, discoverable, and consistent across shells while preserving low-latency navigation.

## What Changes

- Add a new `dx menu` command that opens an interactive selector for navigation candidates based on command-buffer context.
- Define a shell-agnostic buffer protocol so shell hooks can pass `buffer`, `cursor`, `cwd`, and `session` context to `dx menu` and apply a structured replacement result.
- Add shell hook integration points for invoking `dx menu` from an opt-in keybinding path while preserving default completion behavior outside dx navigation contexts.
- Add graceful no-TUI fallback behavior for non-interactive terminals and cancellation paths.

## Capabilities

### New Capabilities
- `dx-menu`: Interactive TUI menu selection, buffer-aware candidate filtering, and structured replacement output for shell hooks.

### Modified Capabilities
- `shell-hooks`: Add menu invocation wiring and context-aware keybinding integration for Bash/Zsh/Fish/PowerShell.

## Impact

- Affected code: `src/cli`, `src/hooks`, new `src/menu` module(s), and integration tests for hook/menu contracts.
- APIs/CLI: Introduces `dx menu` command and structured output contract used by shell wrappers.
- Dependencies: Adds terminal UI dependencies (for example `ratatui`/`crossterm`) and test fixtures for buffer/selection flows.
- Systems: Hook generation templates and shell profile eval output for supported shells.
