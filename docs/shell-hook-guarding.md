# Shell Hook Guarding Strategy

This document describes how shell hooks should invoke `dx resolve` safely from `cd` wrappers and command-not-found handlers.

## Goals

- Preserve native shell behavior for regular commands.
- Only call `dx resolve` for path-like inputs.
- Prevent recursion loops when hooks call back into shell functions.

## Guard Rules

1. **Path-like filter first**
   - Only attempt `dx resolve` when token resembles a path shortcut:
     - contains `/`
     - contains `.`
     - starts with `~`
     - equals known alias (e.g., `up`)
     - starts with multi-dot alias (`...` and longer)

2. **Recursion guard environment variable**
   - Set `DX_RESOLVE_GUARD=1` for the nested resolve call from command-not-found handlers.
   - If the guard is already set, bail out to native shell "command not found" behavior.

3. **No shell-internal `cd` inside `dx` binary**
   - The `dx` binary only prints resolved paths.
   - The shell hook performs `builtin cd` itself.

4. **Fallback on resolve miss**
   - If `dx resolve` exits non-zero or emits no path, fall back to native shell error handling.

## Prototype Hook Files

- `scripts/hooks/dx.bash`
- `scripts/hooks/dx.zsh`

These are prototype integration scripts and intentionally minimal.
