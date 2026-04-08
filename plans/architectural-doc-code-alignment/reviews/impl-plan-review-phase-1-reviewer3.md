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

The implementation plan is well grounded in real doc/code drift and generally stays inside the intended docs/contracts-only Phase 1 boundary. However, it is not fully execution-ready yet because the plan leaves the required C4/C5 boundary-contract output too implicit and its primary verify command is too weak to prove Phase 1 acceptance.

## Scope Alignment

### Findings

- The plan stays within the gated phase scope: all listed file changes are docs/contracts artifacts, runtime changes are explicitly deferred to Phase 2, and the thin-wrapper/Rust-owned contract is preserved.
- **Minor**: Step 1 does not explicitly require the conflict inventory to record the concrete approved noop/error/replace behavior and payload/escaping contract in the level of detail demanded by `phase-1.md` acceptance criteria; “resolved/deferred with decision text” is directionally right but still leaves implementer discretion on the exact artifact shape.

## Technical Feasibility

### Findings

- The proposed approach is technically sound for a docs/contracts-only pass: the cited source files match the current command surface (`dx complete`, `dx navigate`, `dx stack`, `dx menu`) and real cross-shell divergences.
- No runtime behavior changes are proposed here, which is appropriate given Phase 1 excludes hook/menu implementation changes.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries with code-grounded decisions | Yes | No | Needs an explicit requirement to record the approved C4/C5 shell fallback matrix and menu payload/escaping contract, not just status + notes. |
| 2 | Refresh shell-hook contract docs to current semantics | Yes | Yes | — |
| 3 | Rewrite PRD to current command surface and architecture baseline | Yes | Yes | — |
| 4 | Cross-check docs against source and phase scope boundaries | Yes | No | “Final pass” is too generic; it should name the specific completion checks that prove Phase 1 acceptance. |

## Required Context Assessment

### Missing Context

- `src/cli/init.rs` — Phase 1 explicitly refreshes init guidance, especially PowerShell usage, and this file is the concrete `dx init` entrypoint.
- `src/menu/tui.rs` — if Phase 1 is documenting no-TTY/degraded menu behavior and internal environment constraints, this file is the direct anchor for TTY/backend behavior and cleanup assumptions.

### Unnecessary Context

- None.

## Testing Plan Assessment

### Test Integrity Check

The plan does state the key integrity constraint for this phase: no existing Rust/unit/integration tests should be modified, removed, skipped, or weakened during a docs/contracts-only pass. That is good and consistent with scope. However, the verify strategy is still insufficient because it does not fully prove the contract baseline that later phases must rely on.

### Test Gaps

- **Major**: The primary verify command only checks for removal of a handful of obsolete strings. It does not verify that every conflict inventory row is actually `Resolved`/`Deferred`, that the approved C4/C5 contract is recorded, or that refreshed docs now state the intended current/target behavior correctly.
- **Minor**: The plan includes no explicit quick sanity or real-world check, even though the phase scope allows quick sanity checks while editing docs. A lightweight generated-hook inspection step would better protect against documenting behavior that the code does not actually expose.

### Real-World Testing

Real-world testing is **not meaningfully planned** in this implementation plan. For a docs/contracts-only phase that is an acceptable limitation, but it should be stated explicitly as a limitation and supplemented with at least one lightweight sanity check (for example, inspecting generated init output / hook text across supported shells) so Phase 1 does not rely on text-only grep validation.

## Reference Consistency

### Findings

- The cited file references are mostly valid and well chosen: `docs/cd-extras-cli-prd.md` and `docs/shell-hook-guarding.md` are genuinely stale relative to `src/cli/*`, `src/hooks/*`, and `src/menu/action.rs`.
- **Minor**: The implementation plan discusses PowerShell init guidance and degraded menu behavior, but its required context and reality-check anchors omit the most direct files for those topics (`src/cli/init.rs` and `src/menu/tui.rs`).

## Reality Check Validation

### Findings

- The Reality Check is substantially honest: the listed mismatches are real, and the Zsh fallback divergence / parser asymmetry are accurately called out as still-unimplemented runtime convergence work.
- **Minor**: The anchor set is good but not fully complete for the plan's stated topics; adding direct anchors for init dispatch and TTY/degraded behavior would make the Reality Check stronger and reduce guesswork during doc refresh.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Major | Testing Plan | The primary verify command only proves that a few obsolete phrases were removed; it does not prove the conflict inventory is complete or that the approved contract baseline was actually captured. | Replace or extend the verify step with checks that assert C1-C5 status completion and presence of the approved C4/C5 contract language, not just absence of stale strings. |
| 2 | Major | Step Quality / Scope Compliance | The plan does not explicitly require the conflict inventory to capture the concrete approved shell fallback matrix and menu payload/escaping contract required by Phase 1 acceptance criteria. | Make Step 1 explicit about the required C4/C5 output structure and the exact contract details that Phase 2 must inherit. |
| 3 | Minor | Required Context | `src/cli/init.rs` and `src/menu/tui.rs` are missing from the required context even though the plan covers init guidance and degraded/no-TTY menu behavior. | Add both files to Required Context and reference them in the Reality Check. |
| 4 | Minor | Real-World Testing | The plan does not include even a lightweight sanity check, leaving validation almost entirely to grep-based text checks. | Add one non-invasive sanity step such as inspecting generated hook/init output for the documented contracts. |

## Recommendations

1. Strengthen the verify plan so it proves Phase 1 acceptance, not just removal of stale phrases.
2. Make the C4/C5 contract output explicit in Step 1 so Phase 2 can implement without guessing.
3. Add `src/cli/init.rs` and `src/menu/tui.rs` to the required context / reality-check anchors.
4. Add a lightweight real-world sanity check and document Phase 1's limited testing posture explicitly.
