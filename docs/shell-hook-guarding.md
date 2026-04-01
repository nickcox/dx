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
