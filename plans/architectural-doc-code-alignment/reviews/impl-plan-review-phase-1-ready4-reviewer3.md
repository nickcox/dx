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

This revision is close: it stays within the docs/contracts-only phase boundary, uses real source anchors, and Step 1 now makes the C4/C5 Phase 2 handoff requirements materially explicit. The remaining blocker is the single primary verify command, which is satisfiable but still too syntactic to prove that the Phase 1 contract baseline is strong enough for acceptance.

## Scope Alignment

### Findings

- The implementation plan stays within gated Phase 1 scope. It limits edits to docs/plan artifacts and explicitly treats any runtime mismatch discovered during review as Phase 2 work rather than hidden scope creep.

## Technical Feasibility

### Findings

- The approach is technically feasible and aligned with the current codebase. The cited anchors in `src/cli/*`, `src/hooks/*`, `src/menu/action.rs`, `src/menu/buffer.rs`, and `src/menu/tui.rs` are the right sources for adjudicating current contracts without changing runtime behavior.
- Step 1 is substantially improved: it now requires explicit per-shell C4 entries and labeled C5 contract items covering fields, offset semantics, escaping, dependency-free parsing, and split I/O.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries with code-grounded decisions | Yes | Yes | Strong and explicit for C4/C5 handoff; no material ambiguity remains in the step text itself. |
| 2 | Refresh shell-hook contract docs to current semantics | Yes | Yes | Concrete and correctly bounded to docs/contracts. |
| 3 | Rewrite PRD to current command surface and architecture baseline | Yes | Yes | Concrete and aligned to current CLI/hook reality. |
| 4 | Cross-check docs against source and phase scope boundaries | Yes | Mostly | The completion criteria are sound, but the primary verify command does not fully enforce them. |

## Required Context Assessment

### Missing Context

- None. The required context now includes the plan, phase doc, conflict inventory, smoke matrix, AGENTS guidance, relevant docs, and the specific `src/cli`, `src/hooks`, and `src/menu` anchors needed to execute the work.

### Unnecessary Context

- None material. `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` is future-phase verification context, but it is still useful because Step 1 and the docs refresh must set up those scenarios precisely.

## Testing Plan Assessment

### Test Integrity Check

- The plan explicitly preserves test integrity for this docs/contracts-only phase: no existing Rust/unit/integration tests are to be modified, removed, skipped, or weakened, and later phases still inherit the full automated-test plus four-shell smoke bar.

### Test Gaps

- **Major**: The primary verify command is satisfiable, but it still does not prove the substantive strength of the Phase 1 handoff contract. It checks for global marker presence (`Bash:`, `Zsh:`, `Fields:`, `Offset Unit:`, etc.) and stale-string absence, but it does not verify that each shell entry actually covers the required C4 scenarios, nor that the C5 section states offset/escaping semantics with enough specificity for Phase 2 implementation.
- **Minor**: The verify command proves presence of a few current-contract markers in `docs/cd-extras-cli-prd.md` and `docs/shell-hook-guarding.md`, but it does not check for some architecture commitments the plan says should remain intact, especially the thin-wrapper boundary and Rust-owned selector resolution.

### Real-World Testing

For Phase 1, the lightweight manual docs/code-anchor sanity check is acceptable because the phase is intentionally docs/contracts-only and does not claim runtime convergence. The plan appropriately preserves real four-shell smoke verification for later phases instead of overstating what Phase 1 can prove.

## Reference Consistency

### Findings

- The implementation plan's file references are valid against the current repo. The cited code paths exist, and the referenced symbols/areas match current behavior: `dx stack undo|redo` exists under `src/cli/stacks.rs`; `dx menu` emits JSON via `src/menu/action.rs`; `replaceStart`/`replaceEnd` are byte offsets in `src/menu/buffer.rs`; and current shell divergence is visible in `src/hooks/{bash,zsh,fish,pwsh}.rs`.

## Reality Check Validation

### Findings

- The reality check is honest and well grounded in current code. It correctly captures the stale doc areas, the present Zsh fallback divergence, the parsing asymmetry between PowerShell and the POSIX shells, and the fact that Phase 1 must document rather than implement convergence.
- The chosen anchors support the key Phase 1 contract questions, including the important byte-offset detail for `replaceStart`/`replaceEnd` and the split-I/O/TTY behavior around `dx menu`.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Major | Testing Plan | The single primary verify command is satisfiable but too weak to prove that C4/C5 are specified strongly enough for Phase 1 acceptance and Phase 2 handoff. | Strengthen the command so it validates substantive C4/C5 content, not just marker presence. |
| 2 | Minor | Testing Plan | Verification checks a handful of command markers but does not assert some preserved architecture contracts the plan says must stay intact, such as thin shell wrappers and Rust-owned selector resolution. | Add checks or checklist items for those architecture invariants in the refreshed docs. |
| 3 | Note | Step Quality Assessment | Step 1 now makes the C4/C5 handoff requirements explicit enough for Phase 2 and is no longer the weak point in the plan. | Keep Step 1 as written; focus revisions on the verification gate. |

## Recommendations

1. Strengthen the primary verify command so it validates substantive C4/C5 contract content, especially shell-by-shell C4 scenario coverage and explicit C5 offset/escaping semantics.
2. Extend verification or the manual checklist to confirm preserved architecture contracts such as thin wrappers and Rust-owned selector resolution.
3. Otherwise keep the implementation plan structure and docs/contracts-only boundary intact; the plan is close to execution-ready once the verify gate is tightened.
