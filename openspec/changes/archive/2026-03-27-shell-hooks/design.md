## Context

dx has three core capabilities implemented: path resolution (`dx resolve`), session stacks (`dx push/pop/undo/redo`), and bookmarks (`dx mark/unmark/bookmarks`). Each works as a standalone CLI subcommand, but none integrate into the shell's normal `cd` flow. Users must manually invoke `dx resolve` and pipe results to `cd`, or call `dx push` after every directory change.

Prototype hooks exist at `scripts/hooks/dx.bash` and `scripts/hooks/dx.zsh`. These define a `cd` wrapper and `command_not_found` handler with a recursion guard (`DX_RESOLVE_GUARD`), but they lack session stack integration (no `dx push` calls), are static files rather than generated output, and don't cover Fish or PowerShell.

The existing stacks CLI already supports session identity via `--session <id>` or the `DX_SESSION` environment variable (see `src/cli/stacks.rs:86-98`). This means hooks only need to export `DX_SESSION` once at init time for all stack operations to work transparently.

Target shells: Bash, Zsh, Fish, PowerShell.

## Goals / Non-Goals

**Goals:**

- Single onboarding line per shell profile (`eval "$(dx init bash)"`, `dx init fish | source`, `Invoke-Expression (dx init pwsh)`)
- cd wrapper that transparently resolves via `dx resolve` and records to session stacks
- Optional auto-cd via `command_not_found` handler for path-like inputs (opt-in at init time)
- Recursion-safe hooks (no infinite loops when `dx` calls fail or recurse)
- Session identity established once at init, inherited by all dx subcommands
- Consistent behavior across all four shells, with shell-idiomatic implementation details

**Non-Goals:**

- Tab completion integration (deferred to `completions` change)
- `dx menu` TUI binding (deferred to `dx-menu` change)
- Frecency recording (deferred until frecency store or zoxide integration exists)
- PowerShell `cd-extras` module migration or compatibility layer
- Nushell support (out of scope for this change)
- Forcing `command_not_found` overrides by default

## Decisions

### D1: Hook generation — Embedded string constants in Rust

**Choice:** Each shell's hook script is stored as a Rust string constant (or `include_str!` from `.sh`/`.fish`/`.ps1` template files) and printed to stdout by `dx init <shell>`. No runtime template file loading or filesystem dependencies.

**Why:** The dx binary must be fully self-contained. Users should be able to install dx as a single binary and immediately run `eval "$(dx init bash)"`. External template files would require a known install path, complicating distribution.

**Module layout:** `src/hooks/mod.rs` (public `generate(shell: Shell) -> String`), with `src/hooks/bash.rs`, `src/hooks/zsh.rs`, `src/hooks/fish.rs`, `src/hooks/pwsh.rs` containing per-shell hook code. `src/cli/init.rs` handles the CLI subcommand.

**Alternatives considered:**
- Runtime template files (e.g., `$PREFIX/share/dx/hooks/`): adds install complexity, breaks single-binary distribution.
- Generating hooks dynamically from a shared AST: over-engineered for what is essentially four static strings with minor variations.

### D2: Session identity — Export `DX_SESSION` at init time

**Choice:** The generated hook code exports `DX_SESSION` set to the shell's PID at init time. All subsequent dx subcommands (push, pop, undo, redo) pick it up from the environment automatically.

**Per-shell PID source:**
- Bash/Zsh: `$$`
- Fish: `$fish_pid` (Fish has no `$$`)
- PowerShell: `$PID`

**Why:** The stacks CLI already checks `DX_SESSION` as a fallback when `--session` is not provided (`src/cli/stacks.rs:91`). Exporting it once means the cd wrapper doesn't need to pass `--session` on every `dx push` call, and explicit user invocations of `dx undo`/`dx redo` also work without flags.

**Alternatives considered:**
- Pass `--session $$` explicitly in every dx push call within the hook: works but is redundant given the env var fallback already exists, and means explicit user calls to `dx undo` require `--session`.

### D3: cd wrapper flow — Resolve, change, record

**Choice:** The cd wrapper follows this flow:

1. **No arguments** → `builtin cd` (go to `$HOME`), then `dx push "$HOME"` to record.
2. **`cd -`** → `builtin cd -` (go to `$OLDPWD`), then `dx push "$OLDPWD"` to record.
3. **`cd <arg>`** → Run `dx resolve "<arg>"`. If resolve succeeds, `builtin cd "$resolved"`. If resolve fails, fall through to `builtin cd "<arg>"` (let the shell handle it natively). After a successful `cd`, call `dx push "$PWD"` to record.

The push call is fire-and-forget: if `dx push` fails (e.g., session dir unwritable), the cd still succeeds. The hook must never cause a directory change to fail due to stack recording errors.

**cd flags:** Flags like `-L` and `-P` are passed through to `builtin cd`. The hook only resolves the path argument, not flags.

**Why fall through on resolve failure:** The user may be passing arguments that `dx resolve` doesn't understand but `cd` does (e.g., environment variables, shell-specific special paths). The hook should enhance, not restrict.

**Alternatives considered:**
- Always require `dx resolve` success before cd: too restrictive, breaks edge cases.
- Skip push on `cd -` and `cd` with no args: loses history for these common patterns.

### D4: command_not_found integration — Opt-in with path-like filter

**Choice:** `dx init <shell>` always emits the `cd` wrapper. It emits `command_not_found` integration only when explicitly requested (for example `dx init <shell> --command-not-found`).

When enabled, the handler only invokes `dx resolve` when the unrecognized command looks path-like. The heuristic checks for: contains `/`, starts with `.` or `~`, or matches a multi-dot pattern (`...`, `....`, etc.).

If the input does not match the heuristic, the handler immediately falls through to the shell's standard "command not found" behavior without invoking dx.

**Why:** This keeps the default integration conservative and low-risk while still enabling the "type abbreviated path directly" UX for users who want it.

On success, the handler calls shell-native `cd` (`builtin cd` in POSIX shells, `Set-Location` in PowerShell) and then `dx push "$PWD"` (same as cd wrapper semantics).

**Alternatives considered:**
- Default-on command_not_found with opt-out: more convenient, but too invasive by default and can conflict with existing shell frameworks.
- No command_not_found support: safest, but loses a key auto-cd workflow from the original cd-extras experience.

### D5: Recursion guard — `DX_RESOLVE_GUARD` env var

**Choice:** Keep the recursion guard, but only in the `command_not_found` path (which is already opt-in). The handler sets `DX_RESOLVE_GUARD=1` before calling `dx resolve` and unsets it immediately after. If the guard is already set when the handler is entered, it short-circuits to default behavior.

Additionally, hooks check that `dx` is available (`command -v dx` / `Get-Command dx`) before invoking resolver calls.

This prevents infinite loops when:
- The command_not_found handler is invoked for `dx` itself (if dx is not on PATH)
- A nested `command_not_found` path occurs while resolving

**Implementation per shell:**
- Bash/Zsh: `DX_RESOLVE_GUARD=1 dx resolve ...` (inline env assignment, doesn't export to subshells)
- Fish: `set -lx DX_RESOLVE_GUARD 1; dx resolve ...; set -e DX_RESOLVE_GUARD`
- PowerShell: `$env:DX_RESOLVE_GUARD = '1'; dx resolve ...; Remove-Item Env:DX_RESOLVE_GUARD`

### D6: PowerShell integration — Set-Location wrapper and CommandNotFoundAction

**Choice:** PowerShell hooks use `Set-Location` instead of `builtin cd` (PowerShell has no `builtin` keyword — `Set-Location` is the native cmdlet). The cd wrapper is implemented as a `function cd` that shadows the default `cd` alias.

For command_not_found (when enabled): use `$ExecutionContext.InvokeCommand.CommandNotFoundAction` when that member exists. This is feature-detected at runtime rather than hardcoding a minimum PowerShell version.

**Session identity:** `$PID` is available in all PowerShell versions and is equivalent to `[System.Diagnostics.Process]::GetCurrentProcess().Id`.

**Onboarding:** `Invoke-Expression (dx init pwsh)` in `$PROFILE`.

**Alternatives considered:**
- Hardcoded version checks (for example "7.4+"): brittle and easy to get wrong.
- Use `Set-PSBreakpoint` trick for command not found: fragile and non-standard.

### D7: Fish auto-cd interaction — Cooperate, don't disable

**Choice:** Fish has built-in auto-cd (typing a directory name as a command changes to it). The dx `fish_command_not_found` handler cooperates with this: it only attempts `dx resolve` for inputs that Fish's native auto-cd would not handle (abbreviated paths, multi-dot patterns, bookmark names). If the input is a literal existing directory, Fish's auto-cd fires first and the command_not_found handler is never reached.

**Why:** Disabling Fish's auto-cd would break user expectations. Since Fish auto-cd only handles literal directory names and dx resolve handles abbreviations and special patterns, there's minimal overlap.

**Alternatives considered:**
- Disable Fish auto-cd and route everything through dx: too invasive, breaks expected Fish behavior.
- Skip command_not_found entirely for Fish: loses abbreviated path auto-cd support.

### D8: Module and CLI layout

**Choice:**

```
src/
  hooks/
    mod.rs       # Shell enum, generate() dispatcher
    bash.rs      # Bash hook script constant + generation
    zsh.rs       # Zsh hook script constant + generation
    fish.rs      # Fish hook script constant + generation
    pwsh.rs      # PowerShell hook script constant + generation
  cli/
    init.rs      # `dx init <shell>` subcommand handler
    mod.rs       # Add Init variant to Commands enum
```

The `Shell` enum (`Bash`, `Zsh`, `Fish`, `Pwsh`) is defined in `src/hooks/mod.rs` and reused by the CLI. Each shell module exports a `fn generate() -> String` that returns the complete hook script.

## Risks / Trade-offs

**[Risk] Shell version fragmentation** → PowerShell 5.1 vs 7.x, Bash 3.2 (macOS default) vs 5.x, Fish 3.x vs 4.x. Mitigation: test against oldest supported versions. Use only POSIX-compatible constructs in Bash hooks. Document minimum shell versions.

**[Risk] Conflicts with existing shell frameworks** → oh-my-zsh, Prezto, Fisher, and Starship may also override `cd` or `command_not_found`. Mitigation: dx hooks use `builtin cd` (not the framework's cd), and the command_not_found handler can chain to a previously defined handler if one exists. Document known interactions.

**[Risk] dx binary not on PATH when hook runs** → If `dx` is installed but not on PATH, `eval "$(dx init bash)"` will fail silently or error. Mitigation: `dx init` could emit a comment with the full binary path, but the simpler approach is to document that dx must be on PATH before the eval line in the profile.

**[Trade-off] Static strings vs dynamic generation** → Embedding hook code as string constants means hooks can't be customized without recompiling dx. Accepted: hook customization is a non-goal; users who want custom behavior can write their own hooks using dx subcommands directly.

**[Trade-off] Fire-and-forget push** → Silent failure on `dx push` errors means users might not notice broken session stack recording. Accepted: a failed stack push should never block navigation. Users can diagnose with `dx undo --session $$` if stacks seem empty.

## Open Questions

- **Should hooks chain to a previously defined command_not_found handler?** If oh-my-zsh or another framework already defines one, should dx's handler call the original on dx resolve failure? This adds complexity but improves interop.
