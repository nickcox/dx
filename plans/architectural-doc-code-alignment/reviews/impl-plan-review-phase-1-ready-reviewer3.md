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

The plan is close and remains appropriately scoped to docs/contracts-only work, and the primary verify command is now satisfiable in principle because it checks only text artifacts that Phase 1 is allowed to change. However, the acceptance gate is still not strong enough for Phase 1 sign-off: it can pass without proving the C4/C5 handoff contract is complete enough for Phase 2, especially around the `dx menu` split I/O boundary and the byte-based offset contract implemented in `src/menu/buffer.rs`/`src/cli/menu.rs`.

## Scope Alignment

### Findings

- The plan stays within Phase 1 scope and correctly defers runtime hook/menu changes to Phase 2; I found no scope-creep issue.
- Step 3 remains docs-only and aligned to the phase deliverables, though its blanket ban on obsolete spellings in refreshed docs is stricter than the phase acceptance criteria, which allow clearly labeled historical context.

## Technical Feasibility

### Findings

- The proposed approach is technically feasible and matches the current architecture: current command surface in `src/cli/mod.rs`, selector resolution in `src/cli/complete.rs`, stack commands in `src/cli/stacks.rs`, and current hook divergence in `src/hooks/{bash,zsh,fish,pwsh}.rs` all support a docs/contracts-only adjudication pass.
- **Major**: Step 1 does not explicitly require the C5 handoff to capture the split I/O contract that the current implementation depends on: `dx menu` keeps stdout machine-readable JSON while the TUI interacts via tty/dev-tty paths (`src/menu/tui.rs`, `src/hooks/*.rs`, AGENTS “Menu integration is opt-in with noop fallback”). Without that, Phase 2 could produce a payload contract that is textually complete but operationally incomplete.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries with code-grounded decisions | yes | no | Missing explicit requirement to document the stdout-vs-tty handoff and byte-offset source of truth for `replaceStart`/`replaceEnd`. |
| 2 | Refresh shell-hook contract docs to current semantics | yes | yes | Concrete and properly scoped to docs only. |
| 3 | Rewrite PRD to current command surface and architecture baseline | yes | yes | Actionable, though stricter-than-necessary prohibition on obsolete spellings may unnecessarily rule out clearly labeled historical context. |
| 4 | Cross-check docs against source and phase scope boundaries | yes | no | Done checks are useful, but the primary gate still allows shallow marker compliance instead of proving acceptance-level contract completeness. |

## Required Context Assessment

### Missing Context

- `src/menu/buffer.rs` — this is the code that defines `replace_start`/`replace_end` as buffer byte offsets and explains the replacement region semantics. Because Step 1 explicitly requires documenting offset-unit interpretation, omitting this file leaves a core C5 detail guessable.

### Unnecessary Context

- None.

## Testing Plan Assessment

### Test Integrity Check

- The plan clearly states that no existing tests should be modified or weakened in Phase 1, which is appropriate for a docs/contracts-only phase.
- **Major**: the primary verify command is satisfiable, but it is still too marker-driven to serve as the main acceptance gate. It checks for headings/keywords such as `Approved C4 Target Behavior`, `replaceStart`, `offset-unit`, and a few current-contract strings in docs, but it does not prove that each conflict row has meaningful decision/evidence text, that the per-shell C4 branches are all explicitly enumerated, or that the shell-hook doc actually documents the agreed target fallback behavior beyond one `dx stack undo` marker.

### Test Gaps

- **Major**: the verify command does not assert the shell-boundary I/O contract (`stdout` reserved for JSON action output; interactive UI on tty/dev-tty paths) even though this is part of the real code contract in `src/menu/tui.rs`, `src/hooks/bash.rs`, `src/hooks/zsh.rs`, `src/hooks/fish.rs`, and `src/hooks/pwsh.rs`.
- **Minor**: the verify command checks `dx stack undo` in `docs/shell-hook-guarding.md` but not the corresponding `dx stack redo`, `dx navigate`, or explicit target-alignment wording for noop/error fallback, so the primary gate can still miss a partially refreshed shell-hook contract doc.

### Real-World Testing

For a docs/contracts-only phase, the lightweight manual line-citation sanity check is an acceptable real-world complement to the text-based verify command. Deferring four-shell smoke execution to later phases is consistent with the phase scope, as long as this phase only documents the target contract and does not claim runtime convergence.

## Reference Consistency

### Findings

- The referenced CLI/hook files and symbols in the implementation plan are real and current.
- **Minor**: because `src/menu/buffer.rs` is omitted, the plan’s references for C5 are incomplete relative to the exact offset semantics it asks the implementer to document.

## Reality Check Validation

### Findings

- The reality check is mostly honest: it correctly calls out stale docs, current Zsh divergence, and the asymmetric parsing approaches across shells.
- **Major**: the code-anchor set is not quite sufficient for the claimed C5 contract baseline because it misses `src/menu/buffer.rs`, where the cursor/replacement offset semantics are actually defined. That omission weakens both Step 1 and the verify strategy.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Major | Step 1 / C5 handoff | The plan still does not explicitly require the `dx menu` split I/O contract (stdout JSON vs tty/dev-tty UI interaction) to be recorded in the approved C5 contract. | Amend Step 1 and the C5 acceptance checks to require explicit documentation of stdout-only machine output, tty/dev-tty interaction expectations, and dependency-free parsing constraints per shell. |
| 2 | Major | Testing Plan | The primary verify command is satisfiable but not strong enough for Phase 1 acceptance; it can pass on keyword presence without proving decision quality or complete C4/C5 handoff coverage. | Strengthen the primary gate so it verifies all conflict rows are resolved/deferred with non-empty rationale/evidence, and add checks for the shell-hook contract doc plus the key C4/C5 contract clauses. |
| 3 | Major | Required Context / Reality Check | `src/menu/buffer.rs` is missing even though it defines the byte-based replacement-range semantics Step 1 asks the implementer to specify. | Add `src/menu/buffer.rs` to Required Context and Reality Check anchors, and explicitly tie `replaceStart`/`replaceEnd` to byte offsets in the handoff contract. |
| 4 | Minor | Testing Plan | The shell-hook doc portion of the verify command is narrow and may miss partial refreshes (`dx stack redo`, `dx navigate`, target fallback wording). | Expand the shell-hook doc assertions or fold those items into the primary command instead of leaving them only to the manual checklist. |
| 5 | Note | Scope Alignment | The plan otherwise remains execution-ready and appropriately docs/contracts-only. | Keep the runtime-deferral wording unchanged while tightening the C4/C5 contract and verify gate. |

## Recommendations

1. Add an explicit C5 requirement that records the `dx menu` split I/O contract: stdout is reserved for machine-readable JSON actions, while interactivity occurs through tty/dev-tty/PSReadLine paths depending on shell.
2. Add `src/menu/buffer.rs` to Required Context and Reality Check, and require the conflict inventory to name `replaceStart`/`replaceEnd` as byte offsets over the original buffer.
3. Strengthen the primary verify command so it checks more than marker presence: include shell-hook doc contract markers and some structural assertion that each conflict row has been resolved/deferred with rationale/evidence.
4. Optionally relax Step 3’s blanket ban on obsolete spellings if clearly labeled historical context is intentionally retained.
