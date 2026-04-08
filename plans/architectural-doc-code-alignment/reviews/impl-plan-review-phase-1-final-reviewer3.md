---
type: review
entity: implementation-plan-review
plan: "architectural-doc-code-alignment"
phase: 1
status: final
reviewer: "reviewer-3"
created: "2026-04-08"
---

# Implementation Plan Review: Phase 1 - Refresh Architecture Docs and Contracts

> Reviewing [Phase 1 Implementation Plan](../implementation/phase-1-impl.md)
> Against [Phase 1 Scope](../phases/phase-1.md) and [Plan](../plan.md)

## Overall Assessment

**Verdict**: Needs Revision

The revised plan is much closer to execution-ready: it stays within the docs/contracts-only phase boundary, uses real code anchors, and Step 1 now explicitly calls for named C4/C5 outputs. The remaining issue is the primary verify command: it is satisfiable, but it is still too shallow and slightly misaligned with the step text/acceptance bar, so an implementer could satisfy it without actually producing a strong enough Phase 1 contract baseline.

## Scope Alignment

### Findings

- The plan stays within Phase 1 scope. Affected modules are documentation/plan artifacts only, and Step 4 explicitly defers runtime mismatches to Phase 2 instead of pulling code changes into this phase.

## Technical Feasibility

### Findings

- The approach is technically feasible and grounded in current source reality. The cited `src/cli/*`, `src/hooks/*`, `src/menu/action.rs`, and `src/menu/tui.rs` files are the right anchors for adjudicating the current contract without changing behavior.
- **Minor**: Step 1 is improved, but C4 still relies on the phrase `noop/error/replace target behavior matrix` rather than explicitly naming all behavior branches already present in the plan-level smoke matrix (`menu disabled`, `cancel with typed query`, `no candidates`, `no TTY/degraded`, command failure). That leaves some room for under-specifying the target behavior baseline.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries with code-grounded decisions | Yes | Yes | Good improvement: named C4/C5 outputs are explicit. Minor gap: C4 should enumerate required subcases, not just noop/error/replace at a high level. |
| 2 | Refresh shell-hook contract docs to current semantics | Yes | Yes | Actionable and correctly scoped to docs. |
| 3 | Rewrite PRD to current command surface and architecture baseline | Yes | Yes | Actionable, with a useful distinction between current contract and optional historical background. |
| 4 | Cross-check docs against source and phase scope boundaries | Yes | Mostly | The completion pass is sensible, but the primary verify command below does not fully enforce what this step says must be true. |

## Required Context Assessment

### Missing Context

- `AGENTS.md` / project guidance for the PowerShell `Out-String` gotcha is not listed, even though the current conflict inventory cites that guidance for C2 and the implementation plan depends on it when refreshing PowerShell init wording.

### Unnecessary Context

- `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` is useful as future-phase context but is not strictly required to execute the concrete doc edits in Phase 1.

## Testing Plan Assessment

### Test Integrity Check

- The plan clearly states that no existing Rust/unit/integration tests are to be modified, removed, skipped, or weakened in Phase 1. That satisfies the required test-integrity guard for a docs/contracts-only phase.

### Test Gaps

- **Major**: The primary verify command is satisfiable, but not strong enough for Phase 1 acceptance. It proves only string/header presence/absence (`Approved C4...`, `Approved C5...`, a few command markers, and stale-string removal), so it can pass even if the conflict inventory does not actually define the shell-by-shell fallback contract, does not specify escaping/offset semantics well enough for Phase 2, or leaves decision text vague.
- **Minor**: The primary verify command does not check for `dx resolve`, even though Step 3 and the lightweight sanity checklist treat `dx resolve` as part of the refreshed current command surface.
- **Minor**: The verify command forbids obsolete strings anywhere in the refreshed docs, but Step 3 explicitly allows retaining clearly labeled historical background. As written, verification is stricter than the step text and could force the author to remove acceptable historical context just to satisfy the command.

### Real-World Testing

For this phase, limiting real-world testing to a lightweight manual docs/code anchor checklist is acceptable because the phase is explicitly docs/contracts-only and runtime verification is deferred to later phases. The limitation is appropriately acknowledged, and the plan preserves the higher four-shell smoke bar for Phases 2-3.

## Reference Consistency

### Findings

- The cited source references are real and materially relevant: `src/cli/mod.rs` shows the current top-level command surface; `src/cli/stacks.rs` confirms `dx stack undo|redo`; `src/cli/menu.rs` and `src/menu/action.rs` confirm the JSON action contract; and `src/hooks/{bash,zsh,fish,pwsh}.rs` confirm the current shell divergence called out in the reality check.
- The implementation plan correctly treats current Zsh menu behavior as divergent from Bash/Fish/PowerShell rather than pretending Phase 1 already aligns runtime behavior.

## Reality Check Validation

### Findings

- The reality check is honest and well anchored. It correctly captures that Zsh currently does not fall through to native completion on noop/error, while Bash/Fish/PowerShell do.
- It also correctly identifies the current payload asymmetry: PowerShell parses structured JSON with `ConvertFrom-Json`, while Bash/Zsh/Fish use string/regex extraction.
- **Minor**: Because the current plan makes C5 a handoff contract for Phase 2, the reality check should more explicitly call out that `replaceStart`/`replaceEnd` are part of the emitted JSON contract and therefore need a documented offset-unit interpretation in the Phase 1 contract output.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Major | Testing Plan | The primary verify command is satisfiable but too weak: it can pass without proving that C4/C5 are substantively specified strongly enough for Phase 2 handoff and Phase 1 acceptance. | Strengthen the primary verify command or the gated checklist so it asserts substantive C4/C5 contract content, not just header presence. |
| 2 | Minor | Step Quality Assessment | Step 1 now names the C4/C5 outputs, but C4 still does not explicitly enumerate the required behavior branches already implied by the smoke matrix. | Expand Step 1 so `Approved C4 Target Behavior` must cover menu-disabled, replace/select, cancel-with-query-change, noop/error, and no-TTY/degraded behavior for each shell. |
| 3 | Minor | Testing Plan | Verification currently bans obsolete strings anywhere in refreshed docs, which conflicts with Step 3's allowance for clearly labeled historical background. | Either narrow the prohibition to current-contract sections or adjust Step 3 to forbid verbatim obsolete spellings everywhere in the refreshed docs. |
| 4 | Minor | Required Context / Reality Check | The plan depends on the PowerShell init gotcha and emitted menu payload contract details, but required context and reality-check wording do not call those two points out as explicitly as they could. | Add `AGENTS.md` as context for C2 and require the Phase 1 C5 output to state offset-unit and escaping expectations explicitly. |

## Recommendations

1. Strengthen the primary verify command/checklist so it validates substantive C4/C5 contract content rather than only header/marker presence.
2. Tighten Step 1 by explicitly naming the C4 behavior branches that must be documented for each shell.
3. Resolve the inconsistency between Step 3's historical-context allowance and the verify command's blanket stale-string prohibition.
4. Add the PowerShell init gotcha guidance and explicit payload offset/escaping expectations to the required context/contract wording.
