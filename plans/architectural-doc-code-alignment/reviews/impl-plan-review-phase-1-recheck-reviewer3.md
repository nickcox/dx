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

The revised implementation plan resolves the main gaps from the prior pass: it now includes the missing code context (`src/cli/init.rs`, `src/menu/tui.rs`, `src/hooks/mod.rs`), makes the C4/C5 conflict-inventory outputs explicit, and adds an automated check that no inventory rows remain `Open`. It remains scoped correctly to docs/contracts only, but the primary verify command is still not strong enough to prove Phase 1 acceptance because it mainly checks sentinel phrases and stale-string removal rather than confirming that the refreshed docs positively encode the approved current contracts.

## Scope Alignment

### Findings

- The plan stays within the gated Phase 1 scope: all affected files are docs/plan artifacts, runtime hook/menu behavior changes are explicitly deferred to Phase 2, and the thin-wrapper / Rust-owned boundary is preserved.
- Previous scope-quality gaps are resolved: Step 1 now explicitly requires the approved cross-shell noop/error/replace matrix and the approved shell-to-`dx menu` payload/escaping contract.

## Technical Feasibility

### Findings

- The approach is technically sound for a docs/contracts-only phase and is grounded in real source anchors across `src/cli/`, `src/hooks/`, and `src/menu/`.
- The plan correctly distinguishes current runtime divergence (especially Zsh fallback behavior) from the Phase 1 documentation baseline that Phase 2 must later implement.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries with code-grounded decisions | Yes | Yes | — |
| 2 | Refresh shell-hook contract docs to current semantics | Yes | Yes | — |
| 3 | Rewrite PRD to current command surface and architecture baseline | Yes | Yes | — |
| 4 | Cross-check docs against source and phase scope boundaries | Yes | Mostly | Completion criteria are clearer now, but the testing section still does not give a strong enough positive proof that the updated docs state the approved current contracts. |

## Required Context Assessment

### Missing Context

- None.

### Unnecessary Context

- None.

## Testing Plan Assessment

### Test Integrity Check

The revised plan now states the Phase 1 integrity boundary clearly: no existing Rust/unit/integration tests are changed or weakened, and four-shell smoke evidence remains deferred to later phases. That resolves the earlier ambiguity about test integrity and phase boundaries.

### Test Gaps

- **Major**: The primary verify command is improved, but it is still not sufficient as the main Phase 1 acceptance gate. It proves that `Open` rows are gone, that two marker phrases exist, and that several stale strings were removed; it does **not** prove that the docs now positively describe the approved current contracts (`dx stack undo|redo`, mode-based `dx complete`, `dx navigate`, PowerShell `Out-String` guidance, and clearly labeled current-vs-target shell fallback behavior).
- **Minor**: The “Lightweight sanity check” is directionally good but still not operationalized as a concrete command or checklist, so the non-runtime validation remains partly subjective.

### Real-World Testing

No real-world shell testing is planned for Phase 1, and that is acceptable because this phase is intentionally docs/contracts-only and phase scope explicitly defers runtime verification. The plan does correctly preserve the stronger four-shell smoke requirement for later phases, but because there is no real-world testing here, the documentation-to-source verification step should be more concrete than a grep-only gate.

## Reference Consistency

### Findings

- Reference quality is materially improved from the prior pass: the plan now includes direct context for init behavior (`src/cli/init.rs`), menu degradation/cleanup behavior (`src/menu/tui.rs`), and hook-surface dispatch (`src/hooks/mod.rs`).
- The cited source anchors align with the current codebase and accurately support the plan’s claims about stale docs and cross-shell divergence.

## Reality Check Validation

### Findings

- The Reality Check is now substantially stronger and more honest than the prior version: it covers the actual command surface, menu action schema, hook generation surface, and real Zsh divergence.
- The noted mismatches are genuine and appropriately framed as either Phase 1 documentation updates or Phase 2 runtime work.
- **Note**: The remaining weakness is not in the anchor set but in the acceptance verification derived from it; the plan observes reality correctly, but the primary verify command still under-tests whether the rewritten docs fully capture that reality.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Major | Testing Plan | The primary verify command is still too weak to serve as the main Phase 1 acceptance gate because it checks markers/absence of stale strings more than positive contract correctness. | Extend the verify gate with positive assertions or a concrete checklist that confirms the refreshed docs explicitly encode the current command surface, PowerShell init guidance, stack wording, and approved current-vs-target fallback contract. |
| 2 | Minor | Testing Plan | The lightweight sanity check remains non-executable and partly subjective. | Add a concrete non-runtime validation step (for example, a short checklist or scripted `rg` assertions for required current-contract strings) so Phase 1 validation is repeatable. |
| 3 | Note | Recheck Outcome | The previously reported gaps around missing context, explicit C4/C5 outputs, and `Open`-row verification have been addressed. | Keep those revisions; they make the plan materially closer to execution-ready. |

## Recommendations

1. Strengthen the primary verify gate so it positively proves the refreshed docs/contracts now state the approved current baseline, not just that stale wording was removed.
2. Turn the lightweight sanity check into a concrete repeatable step.
3. After those testing-plan refinements, proceed with execution; the rest of the plan is appropriately scoped and ready for a docs/contracts-only Phase 1 pass.
