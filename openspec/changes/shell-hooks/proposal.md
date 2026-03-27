## Why

The core dx features (path resolution, session stacks, bookmarks) exist as standalone CLI subcommands, but using them requires manual invocation. Shell hooks are the glue that makes dx a daily-driver: intercepting `cd`, pushing to session stacks automatically, forwarding unrecognized path-like commands through `dx resolve` for auto-cd, and providing a single `eval "$(dx init <shell>)"` onboarding line. Without hooks, every feature requires explicit `dx resolve`, `dx push`, etc. calls. Prototype hooks exist at `scripts/hooks/dx.{bash,zsh}` but they lack session stack integration, `dx init` generation, Fish support, and PowerShell support.

## What Changes

- **New `dx init <shell>` subcommand**: Outputs shell-specific hook code to stdout for Bash, Zsh, Fish, and PowerShell. Users add a single `eval` line (or `Invoke-Expression` in PowerShell) to their shell profile.
- **cd wrapper generation**: Generated hooks define a `cd` wrapper that calls `dx resolve` for path expansion, performs `builtin cd`, and calls `dx push` to record the directory change in the session stack.
- **command_not_found handler generation**: Generated hooks register a handler that forwards path-like unrecognized commands through `dx resolve` for auto-cd, with recursion guard (`DX_RESOLVE_GUARD`).
- **Session stack integration**: Hooks automatically call `dx push <resolved-path> --session <id>` after every successful cd, making undo/redo work without explicit user action.
- **Replaces prototype hooks**: The static `scripts/hooks/dx.{bash,zsh}` files are superseded by generated output from `dx init`.

## Capabilities

### New Capabilities
- `shell-hooks`: Shell-specific hook code generation via `dx init <shell>`, including cd wrappers, command_not_found handlers, session identity management, recursion guards, and session stack recording. Covers Bash, Zsh, Fish, and PowerShell.

### Modified Capabilities
_(none)_

## Impact

- **New CLI subcommand**: `dx init bash|zsh|fish|pwsh` added to `src/cli/`.
- **New module**: `src/hooks/` containing hook script templates for each shell.
- **Deprecates prototypes**: `scripts/hooks/dx.bash` and `scripts/hooks/dx.zsh` superseded (can be removed or left as reference).
- **User onboarding**: Users add `eval "$(dx init bash)"` (or equivalent `Invoke-Expression (dx init pwsh)` for PowerShell) to their shell profile.
- **No new crate dependencies**: Hook generation is string templating, no external crates needed.
