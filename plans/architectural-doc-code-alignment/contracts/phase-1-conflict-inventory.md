# Phase 1 Conflict Inventory and Decision Log

> Authoritative Phase 1 baseline for [architectural-doc-code-alignment](../plan.md).
>
> Phase 1 should update each row from `Open` to `Resolved` or `Deferred`, record the adjudicated contract, and cite the doc/code changes that implement that decision.

## Initial Inventory

| ID | Area | Conflict | Key Evidence | Likely Fix Type | Status | Decision / Notes |
|----|------|----------|--------------|-----------------|--------|------------------|
| C1 | PRD command surface | PRD documented legacy contracts and did not reflect the current command surface implemented in `dx`. | `docs/cd-extras-cli-prd.md`; `src/cli/mod.rs`; `src/cli/complete.rs`; `src/cli/stacks.rs`; `src/cli/menu.rs` | Docs | Resolved | Refreshed PRD to current contract language centered on `dx resolve`, mode-based `dx complete paths`, `dx stack`, `dx navigate`, and `dx menu`; removed legacy spellings from refreshed docs; evidence: updated `docs/cd-extras-cli-prd.md` command and architecture sections. |
| C2 | PowerShell init guidance | Docs needed consistent single-script-block guidance for PowerShell init evaluation. | `docs/cd-extras-cli-prd.md`; `docs/shell-hook-guarding.md`; `AGENTS.md` gotcha | Docs | Resolved | Standardized guidance to `Invoke-Expression ((& dx init pwsh --menu | Out-String))` (and non-menu equivalent) in refreshed docs; evidence: updated init snippets in `docs/cd-extras-cli-prd.md` and `docs/shell-hook-guarding.md`. |
| C3 | Stack wrapper wording | Shell-hook doc wording drifted from runtime contract for stack transitions. | `docs/shell-hook-guarding.md`; `src/hooks/{bash,zsh,fish,pwsh}.rs`; `src/cli/stacks.rs` | Docs | Resolved | Updated wording to `dx stack undo` / `dx stack redo` with optional `--target` for selector-driven jumps and clarified that wrappers avoid extra `dx stack push` on stack transitions; evidence: updated Navigation Wrapper Contract in `docs/shell-hook-guarding.md`. |
| C4 | Cross-shell menu fallback | Shell fallback behavior is not yet fully aligned in runtime, but Phase 2 needs a fixed target contract across shells. | `src/hooks/zsh.rs`; `src/hooks/bash.rs`; `src/hooks/fish.rs`; `src/hooks/pwsh.rs`; `docs/shell-hook-guarding.md` | Code + Docs | Resolved | Approved target behavior recorded below for Bash, Zsh, Fish, and PowerShell, with runtime convergence deferred to Phase 2; evidence: `Approved C4 Target Behavior` in this file plus matching target-language updates in `docs/shell-hook-guarding.md`. |
| C5 | Menu boundary parsing | Shell parsing strategies differ today; Phase 2 needs an explicit, safe, dependency-free boundary contract. | `src/hooks/{bash,zsh,fish,pwsh}.rs`; `src/menu/action.rs`; `src/menu/buffer.rs`; `src/cli/menu.rs` | Code + Docs | Resolved | Approved payload/escaping contract recorded below with fields, offset-unit semantics, value escaping requirements, dependency-free parsing rules, and split I/O constraints; evidence: `Approved C5 Payload/Escaping Contract` in this file and boundary notes in `docs/shell-hook-guarding.md`. |

## Approved C4 Target Behavior

Bash: menu-disabled → native completion fallback; successful replace/select → apply replacement from `dx menu`; cancel-with-query-change → apply final replace action using typed filter; noop/error fallback → native completion fallback; no-TTY/degraded → native completion fallback; no-candidates → treat as noop path and fallback consistently.

Zsh: menu-disabled → native completion fallback; successful replace/select → apply replacement from `dx menu`; cancel-with-query-change → apply final replace action using typed filter; noop/error fallback → target is native completion-equivalent fallback behavior (current divergence is Phase 2 implementation work); no-TTY/degraded → native completion-equivalent fallback; no-candidates → treat as noop path and fallback consistently.

Fish: menu-disabled → native completion fallback; successful replace/select → apply replacement from `dx menu`; cancel-with-query-change → apply final replace action using typed filter; noop/error fallback → native completion fallback; no-TTY/degraded → native completion fallback; no-candidates → treat as noop path and fallback consistently.

PowerShell: menu-disabled → PSReadLine/native completion fallback; successful replace/select → apply replacement through PSReadLine replace APIs; cancel-with-query-change → apply final replace action using typed filter; noop/error fallback → PSReadLine/native completion fallback; no-TTY/degraded → PSReadLine/native completion-equivalent fallback when interactive path unavailable; no-candidates → treat as noop path and fallback consistently.

## Approved C5 Payload/Escaping Contract

Fields: JSON object on stdout with `action` discriminator and, for replace actions, `replaceStart`, `replaceEnd`, and `value`.

Offset Unit: `replaceStart` and `replaceEnd` are offset-unit byte indexes into the original command buffer region as produced by menu buffer parsing; `replaceEnd` is the cursor-side boundary for replacement.

Value Escaping: `value` escaping must preserve intended shell buffer text and must not require external helper tools; replacement text should be treated as opaque payload data emitted by `dx menu` and consumed by shell-specific wrappers.

Dependency-Free Parsing: Hooks must not introduce new required dependencies (no required `jq`, Python, or external parser helpers); shell-specific parsing logic may differ as long as it is deterministic and respects the same JSON action contract.

Split I/O: stdout remains machine-readable JSON payload/action output only; interactive UI/input paths run via tty/dev-tty/PSReadLine mechanisms as applicable so machine-readable transport is isolated from interactive rendering/input capture.

## Phase 1 Exit Criteria for This Artifact

- Every initial conflict row is marked `Resolved` or `Deferred`.
- The approved noop/error/replace contract is recorded for Bash, Zsh, Fish, and PowerShell.
- The approved shell-to-`dx menu` boundary format and escaping expectations are recorded clearly enough for Phase 2 to implement without reopening scope.
