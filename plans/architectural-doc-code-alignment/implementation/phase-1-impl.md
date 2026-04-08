---
type: planning
entity: implementation-plan
plan: "architectural-doc-code-alignment"
phase: 1
status: completed
created: "2026-04-08"
updated: "2026-04-08"
---

# Implementation Plan: Phase 1 - Refresh Architecture Docs and Contracts

> Implements [Phase 1](../phases/phase-1.md) of [architectural-doc-code-alignment](../plan.md)

## Approach

Phase 1 is a docs/contracts-only alignment pass: adjudicate each known conflict case by case, document the approved baseline in the conflict inventory, and refresh architecture docs to match current code contracts without changing runtime behavior. The implementation focus is on authoritative documentation (`docs/*.md`) and plan artifacts (`plans/.../contracts/*`), grounded by direct code anchors in `src/hooks/`, `src/cli/`, and `src/menu/`.

This phase intentionally defers runtime hook/menu behavior changes to Phase 2. Where code and docs disagree, this phase records explicit decisions and handoff constraints so Phase 2 can implement atomically across all four shells without introducing new required dependencies.

## Affected Modules

| Module | Change Type | Description |
|--------|-------------|-------------|
| `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` | modify | Resolve or explicitly defer C1-C5 with concrete adjudications and Phase 2 handoff notes. |
| `docs/shell-hook-guarding.md` | modify | Align stack/navigation/menu/init contract wording to current implementation and approved cross-shell target behavior. |
| `docs/cd-extras-cli-prd.md` | modify | Replace obsolete proposal-era command surface with current `dx` CLI architecture and shell integration contract. |
| `docs/configuration.md` (conditional) | modify (only if needed) | Update only if contradiction is found while reconciling menu/init/runtime flags with source. |

## Required Context

| File | Why |
|------|-----|
| `plans/architectural-doc-code-alignment/plan.md` | Global constraints, user decisions, and phase sequencing boundary. |
| `plans/architectural-doc-code-alignment/phases/phase-1.md` | Gated Phase 1 scope and acceptance criteria (docs/contracts only). |
| `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` | Authoritative adjudication artifact for this phase. |
| `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` | Required four-shell verification baseline for later phases. |
| `AGENTS.md` | Captures the PowerShell init gotcha requiring single-script-block evaluation with `Out-String`. |
| `docs/shell-hook-guarding.md` | Primary shell contract doc to refresh against current hooks. |
| `docs/cd-extras-cli-prd.md` | PRD requiring full refresh to current command surface. |
| `docs/configuration.md` | Validate env/init/menu references; update only if contradiction exists. |
| `src/cli/mod.rs` | Current top-level command surface (`Resolve`, `Init`, `Complete`, `Navigate`, `Bookmarks`, `Stack`, `Menu`). |
| `src/cli/complete.rs` | Mode-based completion contract and Rust-side selector resolution for navigate. |
| `src/cli/stacks.rs` | Current `dx stack` subcommands (`push|undo|redo`) and session/target semantics. |
| `src/cli/init.rs` | Verifies shell init entrypoint and supported shell set (`bash, zsh, fish, pwsh`) reflected in docs/PRD. |
| `src/cli/menu.rs` | Current menu action behavior, noop/replace semantics, and query-preservation behavior. |
| `src/menu/buffer.rs` | Anchors `replaceStart`/`replaceEnd` semantics as byte offsets in parsed command buffer regions. |
| `src/menu/action.rs` | Machine-readable menu payload schema (`action`, `replaceStart`, `replaceEnd`, `value`). |
| `src/menu/tui.rs` | Confirms cancel/select semantics, no-TTY/degraded behavior, and terminal-state restoration expectations in menu docs/contracts. |
| `src/hooks/mod.rs` | Confirms canonical shell list and hook generation entrypoint used by `dx init`. |
| `src/hooks/bash.rs` | Bash fallback and menu payload parsing behavior; stack wrapper call shape. |
| `src/hooks/zsh.rs` | Zsh-specific widget fallback behavior divergence and parsing path. |
| `src/hooks/fish.rs` | Fish fallback and regex-based menu payload extraction path. |
| `src/hooks/pwsh.rs` | PowerShell JSON parsing (`ConvertFrom-Json`) and PSReadLine-based menu replacement path. |

## Implementation Steps

### Step 1: Resolve conflict inventory entries with code-grounded decisions

- **What**: Update `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` so each initial conflict row (C1-C5) ends in `Resolved` or `Deferred` and includes non-empty decision/evidence text (no blank decision cells). Keep two explicit labeled outputs that Phase 2 can inherit without guesswork: `Approved C4 Target Behavior` and `Approved C5 Payload/Escaping Contract`. `Approved C4 Target Behavior` must use explicit per-shell entries (`Bash:`, `Zsh:`, `Fish:`, `PowerShell:`) and cover menu-disabled, successful replace/select, cancel-with-query-change, noop/error fallback, no-TTY/degraded behavior, and no-candidates when distinct. `Approved C5 Payload/Escaping Contract` must use explicit labeled items (`Fields:`, `Offset Unit:`, `Value Escaping:`, `Dependency-Free Parsing:`, `Split I/O:`) and define payload fields, `replaceStart`/`replaceEnd` offset-unit semantics, `value` escaping expectations, dependency-free parsing constraints, and split I/O expectations (stdout machine-readable JSON payload/action data vs tty/dev-tty/PSReadLine interactive UI/input paths).
- **Where**: `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md`.
- **Why**: Phase 1 acceptance requires this artifact to be the authoritative contract baseline for Phase 2.
- **Considerations**: Preserve user decisions explicitly: case-by-case adjudication, refresh PRD to current, align all shells, allow per-shell parsing differences without new dependencies, and keep verification bar at tests + four-shell smokes.

### Step 2: Refresh shell-hook contract docs to current semantics

- **What**: Update `docs/shell-hook-guarding.md` so stack wrapper wording matches current implementation (`dx stack undo|redo`, optional `--target` flow) and menu fallback semantics are documented with explicit cross-shell notes and target alignment guidance.
- **Where**: `docs/shell-hook-guarding.md`; grounded by `src/hooks/{bash,zsh,fish,pwsh}.rs`, `src/cli/menu.rs`, and `src/menu/action.rs`.
- **Why**: Current text has stale/incorrect contract statements that block reliable Phase 2 implementation planning.
- **Considerations**: Keep thin-wrapper boundary language intact (shell changes directory; Rust resolves paths/state). Do not imply runtime behavior changes in Phase 1. If `docs/configuration.md` conflicts are found, update only contradiction lines and avoid broad rewrites.

### Step 3: Rewrite PRD to current command surface and architecture baseline

- **What**: Refactor `docs/cd-extras-cli-prd.md` from legacy migration framing to a current-state architecture/contract PRD aligned with existing CLI and hook behavior.
- **Where**: `docs/cd-extras-cli-prd.md`; grounded by `src/cli/mod.rs`, `src/cli/complete.rs`, `src/cli/stacks.rs`, `src/cli/menu.rs`, and `src/hooks/mod.rs`.
- **Why**: The current PRD still documents obsolete commands (`dx add`, top-level `dx undo/redo`, generic `dx complete <type> <word>`) and outdated PowerShell init guidance.
- **Considerations**: Keep refreshed PRD text enforceably current-contract: summarize any historical context only in paraphrased form and do not include verbatim obsolete command spellings (`dx add`, top-level `dx undo`, top-level `dx redo`, `dx complete <type> <word>`, `Invoke-Expression (& dx init pwsh)`) anywhere in refreshed docs. Current contract sections must prioritize mode-based `dx complete`, `dx stack` subcommands, `dx navigate`, `dx menu`, and PowerShell single-script-block init (`Out-String`).

### Step 4: Cross-check docs against source and phase scope boundaries

- **What**: Perform a completion pass with explicit done checks: (1) conflict inventory has zero `Open` rows and each row includes decision/evidence text, (2) C4/C5 contract outputs are present and unambiguous, (3) refreshed docs match current symbols/contracts in required context files, and (4) any code-vs-doc mismatch needing runtime change is logged under `Reality Check` as Phase 2 work.
- **Where**: Updated docs/contracts plus anchored source files in `src/hooks/`, `src/cli/`, and `src/menu/`.
- **Why**: Prevent scope bleed into runtime changes while still producing a complete, implementation-ready contract baseline.
- **Considerations**: Done signal for Phase 1 is documentation-contract completeness, not runtime parity: if Zsh/Bash/Fish/PowerShell behavior is still divergent in code, keep it explicitly documented as a Phase 2 implementation item rather than rewriting scope.

## Testing Plan

Primary verify command (Phase 1 docs/contracts consistency check):

```bash
bash -lc '
set -euo pipefail
inv="plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md"
prd="docs/cd-extras-cli-prd.md"
guard="docs/shell-hook-guarding.md"

! rg -n -F "| Open |" "$inv" >/dev/null
rg -n "^\| C1 \|.*\| (Resolved|Deferred) \| .*[^|[:space:]].*\|$" "$inv" >/dev/null
rg -n "^\| C2 \|.*\| (Resolved|Deferred) \| .*[^|[:space:]].*\|$" "$inv" >/dev/null
rg -n "^\| C3 \|.*\| (Resolved|Deferred) \| .*[^|[:space:]].*\|$" "$inv" >/dev/null
rg -n "^\| C4 \|.*\| (Resolved|Deferred) \| .*[^|[:space:]].*\|$" "$inv" >/dev/null
rg -n "^\| C5 \|.*\| (Resolved|Deferred) \| .*[^|[:space:]].*\|$" "$inv" >/dev/null

rg -n -F "Approved C4 Target Behavior" "$inv" >/dev/null
rg -n -F "Bash:" "$inv" >/dev/null
rg -n -F "Zsh:" "$inv" >/dev/null
rg -n -F "Fish:" "$inv" >/dev/null
rg -n -F "PowerShell:" "$inv" >/dev/null
rg -n -F "Bash" "$inv" >/dev/null
rg -n -F "Zsh" "$inv" >/dev/null
rg -n -F "Fish" "$inv" >/dev/null
rg -n -F "PowerShell" "$inv" >/dev/null
rg -n -F "menu-disabled" "$inv" >/dev/null
rg -n -F "successful replace/select" "$inv" >/dev/null
rg -n -F "cancel-with-query-change" "$inv" >/dev/null
rg -n -F "noop/error fallback" "$inv" >/dev/null
rg -n -F "no-TTY/degraded" "$inv" >/dev/null
rg -n -F "no-candidates" "$inv" >/dev/null

rg -n -F "Approved C5 Payload/Escaping Contract" "$inv" >/dev/null
rg -n -F "Fields:" "$inv" >/dev/null
rg -n -F "Offset Unit:" "$inv" >/dev/null
rg -n -F "Value Escaping:" "$inv" >/dev/null
rg -n -F "Dependency-Free Parsing:" "$inv" >/dev/null
rg -n -F "Split I/O:" "$inv" >/dev/null
rg -n -F "replaceStart" "$inv" >/dev/null
rg -n -F "replaceEnd" "$inv" >/dev/null
rg -n -F "offset-unit" "$inv" >/dev/null
rg -n -F "value escaping" "$inv" >/dev/null
rg -n -F "dependency-free" "$inv" >/dev/null
rg -n -F "stdout" "$inv" >/dev/null
rg -n -F "machine-readable" "$inv" >/dev/null
rg -n -F "JSON" "$inv" >/dev/null
rg -n -F "dev-tty" "$inv" >/dev/null
rg -n -F "PSReadLine" "$inv" >/dev/null

rg -n -F "dx resolve" "$prd" >/dev/null
rg -n -F "dx stack" "$prd" >/dev/null
rg -n -F "dx navigate" "$prd" >/dev/null
rg -n -F "dx complete paths" "$prd" >/dev/null
rg -n -F "dx menu" "$prd" >/dev/null
rg -n -F "Out-String" "$prd" >/dev/null

rg -n -F "dx stack undo" "$guard" >/dev/null
rg -n -F "dx stack redo" "$guard" >/dev/null
rg -n -F "dx navigate" "$guard" >/dev/null
rg -n -F "dx menu" "$guard" >/dev/null
rg -n -F "Out-String" "$guard" >/dev/null

! rg -n -F "dx add" "$prd" "$guard" >/dev/null
! rg -n -F "dx undo" "$prd" "$guard" >/dev/null
! rg -n -F "dx redo" "$prd" "$guard" >/dev/null
! rg -n -F "Invoke-Expression (& dx init pwsh)" "$prd" "$guard" >/dev/null
! rg -n -F "complete <type> <word>" "$prd" "$guard" >/dev/null
'
```

Expected outcome: command exits 0 only when (a) no conflict-inventory rows remain `Open`, (b) each C1-C5 row is `Resolved` or `Deferred` with non-empty decision text, (c) required C4 branches and C5 payload/offset/escaping + split-I/O markers are present, (d) refreshed PRD specifically contains current-contract markers (`dx resolve`, `dx stack`, `dx navigate`, `dx complete paths`, `dx menu`, `Out-String`), (e) shell-hook guarding doc contains multiple current-contract navigation/menu/init markers, and (f) obsolete contract strings are absent from refreshed docs targets.

| Test Type | What to Test | Expected Outcome |
|-----------|-------------|-----------------|
| Docs contract consistency | Target docs/contracts no longer contain obsolete command/init contract language and reflect approved baseline decisions. | No obsolete-pattern matches; conflict inventory entries resolved/deferred with rationale. |
| Lightweight sanity check (non-invasive) | Manual checklist with explicit anchors: (1) inspect `docs/shell-hook-guarding.md` and point to lines that mention `dx stack undo`/`dx stack redo`, `dx navigate`, and menu fallback wording; (2) inspect `docs/cd-extras-cli-prd.md` current-contract section and point to lines that mention `dx resolve`, `dx complete paths`, `dx stack`, `dx navigate`, `dx menu`, and PowerShell init with `Out-String`; (3) inspect `plans/.../phase-1-conflict-inventory.md` and point to `Approved C4 Target Behavior` branches for menu-disabled, successful replace/select, cancel-with-query-change, noop/error fallback, no-TTY/degraded (and no-candidates if distinct) plus `Approved C5 Payload/Escaping Contract` lines for fields, `replaceStart`/`replaceEnd` offset-unit interpretation, and escaping expectations. | Reviewer can cite concrete lines for all three checklist items, and any remaining runtime divergence is still documented as Phase 2 work (not marked implemented in Phase 1 docs). |

### Test Integrity Constraints

- No existing Rust/unit/integration tests are modified, removed, skipped, or weakened during Phase 1 (docs/contracts-only scope).
- Phase 2 and Phase 3 must still satisfy the plan-level verification bar: updated automated tests plus shell smokes across Bash, Zsh, Fish, and PowerShell.
- `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` remains the required source for four-shell smoke evidence; Phase 1 must not mark unexecuted scenarios as passed.

## Rollback Strategy

If Phase 1 doc updates introduce confusion or overreach scope, revert only the edited Phase 1 documentation artifacts (`docs/shell-hook-guarding.md`, `docs/cd-extras-cli-prd.md`, optional `docs/configuration.md`, and `plans/.../phase-1-conflict-inventory.md`) to last known good state, then re-apply changes conflict-by-conflict using explicit code anchors.

## Open Decisions

| Decision | Options | Chosen | Rationale |
|----------|---------|--------|-----------|
| Documenting current vs target cross-shell noop/error fallback semantics in Phase 1 | (A) Document current divergence as-is, (B) Document intended aligned target and note current mismatch | B | Preserves user decision to align all shells while keeping Phase 1 scope docs/contracts-only and deferring runtime convergence to Phase 2. |

## Reality Check

### Code Anchors Used

| File | Symbol/Area | Why it matters |
|------|-------------|----------------|
| `src/cli/mod.rs:20-54` | `Commands` enum | Confirms current top-level CLI command surface replacing obsolete PRD commands. |
| `src/cli/complete.rs:13-55` | `CompleteCommand` modes | Verifies mode-based completion contract (`paths|ancestors|frecents|recents|stack`). |
| `src/cli/complete.rs:146-166` | `run_navigate` | Confirms selector resolution is in Rust for `up|back|forward`. |
| `src/cli/stacks.rs:12-17` | `StackCommand` | Confirms stack commands are `push|undo|redo` under `dx stack`. |
| `src/cli/init.rs:3-14` | `run_init` | Confirms init contract routes through shell parsing/generation and supports only documented shells. |
| `src/menu/buffer.rs:10-14` | `ParsedBuffer.replace_start/replace_end` | Anchors that replacement boundaries are byte offsets, informing C5 offset-unit requirements. |
| `src/menu/buffer.rs:102-173` | `parse_buffer` | Confirms how replacement byte ranges are computed from buffer/cursor input for `dx menu` actions. |
| `src/menu/action.rs:6-19` | `MenuAction` schema | Establishes machine-readable payload keys (`action`, `replaceStart`, `replaceEnd`, `value`). |
| `src/cli/menu.rs:245-281` | `MenuResult::Cancelled` and noop path | Confirms cancel-with-changed-query emits `replace`; unchanged/no-TTY emits `noop`. |
| `src/menu/tui.rs:82-116` | `CleanupGuard::drop` | Confirms TUI exit/cleanup guarantees (cursor/raw-mode restoration) for degraded/abort paths. |
| `src/menu/tui.rs:139-220` | `select` | Confirms no-candidate/cancel/select flow boundaries reflected in docs contract language. |
| `src/hooks/mod.rs:25-37` | `supported_list` / `generate` | Confirms canonical shell surface and generator dispatch used by `dx init`. |
| `src/hooks/bash.rs:304-330` | `__dx_try_menu` / `_dx_menu_wrapper` | Bash falls back to native completion when menu returns non-replace/error. |
| `src/hooks/zsh.rs:307-311` | `__dx_menu_widget` noop/error path | Zsh currently resets prompt and returns without native completion fallback (divergence). |
| `src/hooks/fish.rs:238-247` | `__dx_menu_complete` fallback path | Fish falls back to `commandline -f complete` on error/non-replace. |
| `src/hooks/pwsh.rs:336-347` | PSReadLine JSON handling | PowerShell uses structured JSON parsing (`ConvertFrom-Json`) and Replace API. |

### Mismatches / Notes

- `docs/shell-hook-guarding.md` currently states stack wrappers use `dx undo`/`dx redo`, but all hook generators call `dx stack undo|redo` (and optional `--target` for selector-driven jumps).
- `docs/cd-extras-cli-prd.md` is still proposal-era and contradicts current command surface and PowerShell init guidance.
- Current runtime behavior is not yet shell-aligned for menu noop/error fallback: Zsh differs from Bash/Fish/PowerShell and requires Phase 2 implementation work.
- Parsing strategies are intentionally asymmetric today (string/regex extraction in Bash/Zsh/Fish vs JSON object parsing in PowerShell); Phase 1 must codify a safe boundary contract and dependency constraints without forcing immediate parser unification.
- `src/menu/tui.rs` confirms interactive cleanup/no-TTY behavior that docs should describe as contract expectations, but this phase must not alter TUI/runtime code.
- Conflict inventory may intentionally retain legacy command strings as historical conflict evidence; stale-string removal checks are therefore limited to refreshed docs, while inventory validation focuses on `Open` status and required decision outputs.
