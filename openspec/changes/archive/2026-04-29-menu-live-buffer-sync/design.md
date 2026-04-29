## Context

`dx menu` currently runs as a subprocess that emits one final JSON action (`replace` or `noop`) on exit. That model works for final selection but cannot update shell input buffers per keypress while the menu is open. The requested UX requires live shell-line edits from typed filter input while preserving non-mutating selection navigation.

Because buffer mutation mechanics are shell-specific (zsh ZLE, bash/readline, fish commandline, PowerShell PSReadLine), this is a cross-cutting protocol and hook integration change.

## Goals / Non-Goals

**Goals:**
- Reflect typed menu filter input in the shell buffer on each keypress.
- Ensure selection movement does not mutate shell buffer until accept.
- Keep cancellation semantics explicit and deterministic.
- Preserve robust fallback behavior for non-interactive and error paths.

**Non-Goals:**
- Reworking candidate scoring/ranking logic.
- Introducing multi-column layout in this change.
- Replacing shell-native completion systems outside dx command scopes.

## Decisions

### D1: Introduce incremental action protocol
Define an action stream (or polling-compatible sequence) where `dx menu` can emit incremental `replace` updates for typed query changes and separate non-mutating selection updates.

Rationale: required for true per-keypress shell buffer synchronization.

Alternative considered: keep one-shot final action and synthesize typing in hooks. Rejected because hooks do not observe menu keypresses today.

### D2: Keep selection navigation side-effect free
Arrow/Tab/j/k movement updates menu highlight only; no shell-line writes for selection-only events.

Rationale: preserves expected “browse without editing” behavior and avoids command-line churn.

### D3: Shell-specific adapter layer applies incremental replace actions
Each shell hook consumes incremental actions and applies text replacements using native APIs (`BUFFER/CURSOR`, `READLINE_LINE/POINT`, `commandline`, `PSConsoleReadLine`).

Rationale: shell-native edit primitives are different; adapter isolation minimizes coupling.

### D4: Finalization semantics
- Accept: final selected candidate replacement is applied and session ends.
- Cancel: if typed query changed, shell retains those typed edits; otherwise buffer remains unchanged.

Rationale: aligns with user-requested persistence and existing menu expectations.

### D5: Backward compatibility guard
Menu integration remains opt-in (`--menu`) and protocol upgrades are gated so older generated hooks fail safely to native completion when they cannot interpret menu actions.

Rationale: avoids breaking existing shell setups during rollout.

## Risks / Trade-offs

- [Protocol complexity across shells] -> Define small, explicit action schema with conformance tests per shell template.
- [Terminal responsiveness regressions] -> Keep `/dev/tty` rendering path unchanged and isolate action emission from render loop.
- [Inconsistent cancel behavior] -> Add explicit tests for typed vs unchanged cancel outcomes.
- [Compatibility drift between binary and generated hooks] -> Add version/feature guard and fallback to native completion on mismatch.

## Migration Plan

1. Define and implement incremental menu action schema in Rust menu runtime.
2. Update `dx menu` command flow to emit/apply incremental query replacements during interaction.
3. Update shell templates for zsh/bash/fish/pwsh to consume incremental actions safely.
4. Add integration tests for per-keypress buffer sync and selection-only non-mutation.
5. Update docs with behavior matrix (typing, selection movement, accept, cancel).
6. Rollback strategy: disable incremental mode and revert to one-shot final-action behavior.

## Open Questions

- Should incremental actions be streamed over stdout, stderr, or an auxiliary fd/IPC channel?
- Do we need explicit protocol version negotiation between binary and generated hooks?
- Should whitespace keystrokes participate in filter-and-sync behavior by default?
