# Shell Hook Guarding Strategy

This document describes the guard and fallback behavior implemented by generated hooks from `dx init`.

For a full list of user configuration options (config file keys, env vars, and per-command overrides), see `./configuration.md`.

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

- Wrappers call native shell directory-change primitives (`builtin cd` / `Set-Location`) and record stack state via `dx stack push`.
- `dx` never changes the shell process directory itself; it only returns paths/state transitions.
- For `/`, `./`, `../`, `~`, and `~/` prefixes, `dx resolve` tries direct filesystem resolution first.
- For leading `/` misses, fallback resolution remains rooted at `/` (root-anchored semantics).
- For `./`, `../`, `~`, and `~/` misses, the prefix is stripped and lookup continues through the standard root-based abbreviation/fallback behavior.
- If `dx resolve` fails, wrappers fall back to native `cd` behavior with original arguments.

## Navigation Wrapper Contract

- Selector resolution for `up|back|forward` is delegated to Rust via `dx navigate`.
- Forward-navigation wrappers seed and record via `dx stack push` around successful navigation.
- Stack-transition wrappers (`back`/`forward`/`cd-`/`cd+`) use `dx stack undo`/`dx stack redo` (and `--target` for selector-based jumps) and must not call extra `dx stack push` operations as part of the transition itself.

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

When menu mode is enabled, pressing Tab on a dx navigation command (`cd`, `up`, `cdf`, `z`, `cdr`, `back`, `forward`, `cd-`, `cd+`) opens an interactive TUI list of candidates. Use arrow keys, Tab, or Shift+Tab to move selection, type characters to filter candidates (including lowercase `j`/`k`), Enter to select, and Esc or Ctrl+C to cancel.

- **Select**: replaces the query in the command buffer with the chosen path
- **Cancel after typing**: preserves typed filter refinement by applying a final `replace` action
- **Cancel without typing**: falls back to native completion / noop semantics

For non-dx commands, Tab behaves normally (native completion).

### Runtime Disable

Set `DX_MENU=0` to disable the menu at runtime without regenerating hooks:

```sh
export DX_MENU=0
```

### Fallback Behavior

The menu boundary contract uses split I/O:

- `dx menu` writes machine-readable JSON actions to stdout (`noop` or `replace` with `replaceStart`/`replaceEnd`/`value`).
- Interactive UI and input handling run through tty/dev-tty/PSReadLine paths depending on shell.

Current runtime behavior:

- **Menu disabled (`DX_MENU=0`)**: hooks use native completion paths.
- **Successful replace/select**: hooks apply the returned replace action.
- **Cancel with query change**: `dx menu` may return replace to preserve typed refinement.
- **No candidates**: `dx menu` returns noop and hooks follow fallback behavior.
- **No TTY / degraded path**: `dx menu` returns noop and hooks follow fallback behavior.
- **Noop/error/non-replace fallback**: Bash and Fish use their native completion fallback; Zsh uses `zle expand-or-complete` (native completion-equivalent); PowerShell falls back to `TabExpansion2` / default completion behavior.
- **dx not found or invalid JSON**: hooks follow fallback behavior.
- **POSIX payload parsing hardening**: Bash/Zsh/Fish wrappers deterministically extract and validate `action`, `replaceStart`, `replaceEnd`, and escaped `value`; invalid payloads (including non-replace actions when replace is required) take native completion fallback paths.
- **PowerShell payload parsing**: remains structured JSON parsing via `ConvertFrom-Json`.
- **Dependencies**: fallback and payload validation behavior is implemented in existing hook/template code paths with no new external runtime dependencies.

### Troubleshooting

If the menu doesn't appear when pressing Tab:

1. Verify `--menu` was passed to `dx init`
2. Check that `dx` is on your PATH: `command -v dx`
3. Verify candidates exist: `dx complete paths <query>`
4. By default, path completion includes the current directory as an implicit root; set `DX_SEARCH_ROOTS` to add or override broader roots
5. Check runtime disable: `echo $DX_MENU` (should not be `0`)
6. Enable debug diagnostics: `export DX_MENU_DEBUG=1` — this emits per-invocation trace on stderr showing buffer, cursor, parsed mode, candidate count, and action taken


### Multicolumn Menu (Optional)

`DX_MENU_ITEM_MAX_LEN` controls optional multicolumn rendering for `dx menu`:

- unset / empty / non-numeric / `< 1`: single-column default behavior
- `>= 1`: multicolumn enabled with dynamic columns computed from terminal width

The selected full path remains visible in the status line even when grid cells are truncated.
