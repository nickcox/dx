---
type: review
entity: implementation-plan-review
plan: "architectural-doc-code-alignment"
phase: 1
status: final
reviewer: "general"
created: "2026-04-08"
---

# Implementation Plan Review: Phase 1 - Refresh Architecture Docs and Contracts

> Reviewing [Phase 1 Implementation Plan](../implementation/phase-1-impl.md)
> Against [Phase 1 Scope](../phases/phase-1.md) and [Plan](../plan.md)

## Overall Assessment

**Verdict**: Needs Revision

The implementation plan is close: it stays within the docs/contracts-only phase boundary, uses real code anchors, and Step 1 now makes the C4/C5 handoff materially more explicit. However, the primary verify command is still not strong enough to serve as the main acceptance gate for that handoff, because it mostly proves token presence rather than section-structured, shell-by-shell contract completeness.

## Scope Alignment

### Findings

- The plan remains within Phase 1 scope: all implementation steps target docs/plan artifacts only, defer runtime convergence to Phase 2, and preserve the thin-wrapper architecture described in `plan.md`, `phase-1.md`, and `AGENTS.md`.
- The only scope pressure I found is in Step 3's blanket prohibition on legacy spellings in refreshed docs, which is stricter than the phase acceptance criterion allowing clearly distinguished historical context.

## Technical Feasibility

### Findings

- The approach is technically sound for a docs/contracts-only phase: the cited anchors in `src/cli/*`, `src/menu/*`, and `src/hooks/*` are real and support the intended contract refresh.
- The primary verify command is satisfiable in principle, but it is weak as a readiness gate because most checks are unscoped string probes that can pass without proving the named C4/C5 sections actually contain the required structured decisions.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries with code-grounded decisions | yes | yes | Stronger than prior versions, but the verify command does not actually prove the named C4/C5 outputs are section-structured and shell-complete. |
| 2 | Refresh shell-hook contract docs to current semantics | yes | yes | Acceptance gate only checks `dx stack undo`, not `dx stack redo`, so the original C3 doc drift could partially survive. |
| 3 | Rewrite PRD to current command surface and architecture baseline | yes | yes | "Do not include verbatim obsolete command spellings anywhere" is stricter than the phase's allowance for clearly labeled historical context. |
| 4 | Cross-check docs against source and phase scope boundaries | yes | yes | Good boundary protection; no blocking issue. |

## Required Context Assessment

### Missing Context

- None.

### Unnecessary Context

- None material. `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` is not essential for editing docs in Phase 1, but its inclusion is harmless and helps preserve later-phase verification context.

## Testing Plan Assessment

### Test Integrity Check

The plan correctly states that no existing Rust/unit/integration tests should be modified or weakened in this phase, and it clearly defers the four-shell automated/smoke verification bar to Phases 2-3. That is appropriate for a docs/contracts-only slice.

### Test Gaps

- **Major**: The primary verify command does not verify that `Approved C4 Target Behavior` contains an actual per-shell handoff structure; it only checks for global presence of shell names and scenario labels somewhere in the inventory file. A malformed or incomplete C4 section could still pass.
- **Minor**: The primary verify command does not verify that `Approved C5 Payload/Escaping Contract` owns the required payload/offset/escaping/split-I/O details; those keywords can appear anywhere in the file and still satisfy the gate.
- **Minor**: The primary verify command checks for `dx stack undo` in `docs/shell-hook-guarding.md` but not `dx stack redo`, even though the documented drift in C3 is specifically an undo/redo pair.

### Real-World Testing

For this phase, no real-world shell execution is required beyond the documented lightweight manual sanity check, and that is acceptable because the phase is explicitly docs/contracts-only. Full shell smoke verification remains appropriately deferred to `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` in later phases.

## Reference Consistency

### Findings

- The referenced source files and symbols exist and match the plan's claims: `src/cli/mod.rs`, `src/cli/complete.rs`, `src/cli/stacks.rs`, `src/cli/init.rs`, `src/menu/action.rs`, `src/menu/buffer.rs`, `src/menu/tui.rs`, and the four shell hook generators all support the described current-state contract.
- The current docs under `docs/` also validate the conflicts the phase intends to resolve: `docs/cd-extras-cli-prd.md` still contains obsolete `dx add`, generic `complete <type> <word>`, and stale PowerShell init guidance; `docs/shell-hook-guarding.md` still says `dx undo`/`dx redo`.

## Reality Check Validation

### Findings

- The reality check is substantially honest: it cites enough real anchors, captures the Zsh fallback divergence, and correctly keeps runtime convergence in Phase 2.
- One remaining honesty gap is enforcement, not diagnosis: the plan text says the C4/C5 baseline must be unambiguous, but the verify command does not yet validate that requirement at the same level of precision.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Major | Testing Plan | The primary verify command is satisfiable but too weak to prove that the `Approved C4 Target Behavior` and `Approved C5 Payload/Escaping Contract` sections are actually complete, structured, and implementation-ready. | Tighten the verify command so it scopes checks to the named sections and validates shell-by-shell/scenario-by-scenario coverage rather than global token presence. |
| 2 | Minor | Testing Plan | The verify command checks `dx stack undo` in `docs/shell-hook-guarding.md` but not `dx stack redo`, so the specific C3 doc correction is only half enforced. | Add a parallel `dx stack redo` assertion in the primary verify command. |
| 3 | Minor | Scope Alignment | Step 3 and the negative grep checks forbid obsolete spellings anywhere in refreshed docs, which is stricter than the plan/phase allowance for clearly distinguished historical context. | Narrow the prohibition to current-contract sections or explicitly permit a labeled historical-context block. |
| 4 | Note | Reality Check | Step 1 is materially improved and now makes the C4/C5 handoff explicit enough in prose to guide an implementer. | Keep the Step 1 text; strengthen enforcement around it rather than rewriting the step again. |

## Recommendations

1. Strengthen the primary verify command so it validates the structure and ownership of the C4/C5 handoff sections, not just keyword presence anywhere in the inventory file.
2. Add the missing `dx stack redo` assertion so C3 acceptance matches the actual documented drift.
3. Relax or better scope the obsolete-string prohibition so the PRD can still carry clearly labeled historical context if needed.
