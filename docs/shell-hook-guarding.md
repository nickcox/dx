# Shell Hook Guarding Strategy

This document describes the guard and fallback behavior implemented by generated hooks from `dx init`.

## Goals

- Preserve native shell behavior for non-path-like command typos.
- Attempt `dx resolve` only when the token looks path-like.
- Prevent command-not-found recursion loops.
- Keep directory-changing semantics in shell wrappers, not in `dx`.

## Path-Like Heuristic

Command-not-found handlers only attempt `dx resolve` when the command token:

- contains `/`
- starts with `.`
- starts with `~`
- starts with `...` (multi-dot alias)

For non-path-like tokens, handlers immediately return native command-not-found behavior (Bash/Zsh/Fish exit 127 semantics preserved).

## Recursion Guard

- Handlers check `DX_RESOLVE_GUARD` first.
- If already set, they do not call `dx resolve` and return native command-not-found behavior.
- For guarded resolve calls, handlers set `DX_RESOLVE_GUARD=1` only for the nested resolve invocation, then clear it.

## cd Wrapper Contract

- Wrappers call native shell directory-change primitives (`builtin cd` / `Set-Location`) and record stack state via `dx push`.
- `dx` never changes the shell process directory itself; it only returns paths/state transitions.
- If `dx resolve` fails, wrappers fall back to native `cd` behavior with original arguments.

## Navigation Wrapper Contract

- Selector resolution for `up|back|forward` is delegated to Rust via `dx navigate`.
- Forward-navigation wrappers seed and record via `dx push` around successful navigation.
- Stack-transition wrappers (`back`/`forward`/`cd-`/`cd+`) use `dx undo`/`dx redo` (and `--target` for selector-based jumps) and must not call `dx push`.

## Source of Truth

Current behavior is implemented in generated hook templates:

- `src/hooks/bash.rs`
- `src/hooks/zsh.rs`
- `src/hooks/fish.rs`
- `src/hooks/pwsh.rs`

Legacy prototype scripts under `scripts/hooks/` are not authoritative.

## Menu Integration

The `dx menu` command provides an interactive TUI selection menu for directory navigation. It is **opt-in** and disabled by default.

### Enabling

Pass `--menu` to `dx init` to generate hooks with menu support:

```sh
# Zsh
eval "$(dx init zsh --menu)"

# Bash
eval "$(dx init bash --menu)"

# Fish
dx init fish --menu | source

# PowerShell
Invoke-Expression ((& dx init pwsh --menu | Out-String))
```

### How It Works

When menu mode is enabled, pressing Tab on a dx navigation command (`cd`, `up`, `cdf`, `z`, `cdr`, `back`, `forward`, `cd-`, `cd+`) opens an interactive TUI list of candidates. Use arrow keys or `j`/`k` to navigate, Enter to select, Esc or Ctrl+C to cancel.

- **Select**: replaces the query in the command buffer with the chosen path
- **Cancel**: falls back to the shell's native completion

For non-dx commands, Tab behaves normally (native completion).

### Runtime Disable

Set `DX_MENU=0` to disable the menu at runtime without regenerating hooks:

```sh
export DX_MENU=0
```

### Fallback Behavior

The menu gracefully degrades in these cases:

- **No TTY available**: outputs `{"action":"noop"}` and hooks fall back to native completion
- **No candidates**: outputs noop
- **dx not found**: hooks fall back to native completion
- **Command failure or invalid JSON**: hooks fall back to native completion

### Troubleshooting

If the menu doesn't appear when pressing Tab:

1. Verify `--menu` was passed to `dx init`
2. Check that `dx` is on your PATH: `command -v dx`
3. Verify candidates exist: `dx complete paths <query>`
4. For path completions, ensure `DX_SEARCH_ROOTS` is configured
5. Check runtime disable: `echo $DX_MENU` (should not be `0`)
6. Enable debug diagnostics: `export DX_MENU_DEBUG=1` — this emits per-invocation trace on stderr showing buffer, cursor, parsed mode, candidate count, and action taken
