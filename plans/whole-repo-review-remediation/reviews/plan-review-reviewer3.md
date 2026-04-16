---
type: review
entity: plan-review
plan: "whole-repo-review-remediation"
status: final
reviewer: "reviewer-3"
created: "2026-04-16"
---

# Plan Review: whole-repo-review-remediation

> Reviewing [whole-repo-review-remediation](../plan.md)

## Overall Assessment

**Verdict**: Needs Revision

The plan is well grounded in the collated repo review, the impact analysis, and the current `tech-docs/` contract, and it captures the user's major decisions: fix the high-severity issues first, retire legacy hook prototypes if they are truly obsolete, and treat PowerShell `ProxyCommand` as an evidence-gated option rather than a foregone rewrite. The remaining issues are mostly about execution control: Phase 4 is too open-ended, the accepted hygiene scope is not concrete enough to be objectively closed, and shell-facing verification is too back-loaded and underspecified for the amount of cross-shell risk involved.

## Requirement Coverage

| Requirement | Covered By | Gap? | Notes |
| ----------- | ---------- | ---- | ----- |
| Safe atomic replacement with no delete-and-retry loss path | Plan Functional 1; Phase 1 | No | Well scoped and correctly prioritized first. |
| Durability tests for bookmark/session failure paths | Plan Functional 2; Phase 1; Testing Strategy | No | Strongly grounded in the review and impact artifact. |
| `dx menu --cwd` authoritative for `paths` mode | Plan Functional 3; Phase 2 | No | Matches the current CLI contract docs. |
| Approved flagged `cd` forms parse correctly | Plan Functional 4; Phase 2 | No | Supported forms are named, but fallback behavior still needs explicit test scenarios. |
| Generated-hook guard coverage replaces legacy prototypes; delete obsolete prototypes | Plan Functional 5; Phase 3 | Minor | Core intent is covered, but prototype retirement criteria are more explicit for Bash than for Zsh. |
| Evaluate `ProxyCommand` and adopt only if justified | Plan Functional 6; Phase 4; Risks/Open Questions | Minor | Framed correctly, but plan-level adopt/reject rubric should be sharper before execution. |
| Land adjacent minor hygiene safely | Plan Functional 7; Phase 4 | **Yes** | Accepted hygiene work is not enumerated, so scope and DoD are not objectively bounded. |
| Preserve thin-wrapper architecture and add no new dependencies | Non-Functional 1-2; Scope/Out of Scope; Phase 4 | No | Good architectural guardrails. |
| Keep user-facing behavior stable outside targeted fixes | Non-Functional 3; Scope/Out of Scope | No | Well stated. |
| Automated tests plus cross-shell smoke evidence | Non-Functional 4; Testing Strategy; DoD | **Yes** | Verification exists in principle, but minimum shell scenarios and phase-level timing are too vague. |
| Keep docs/plans/verification artifacts aligned | Non-Functional 5; DoD; Phase 3-4 | No | Covered. |

## Scope Clarity

### Findings

- The main remediation scope is clear and appropriately tied back to the major review findings.
- The only material scope-control problem is the repeated reference to "directly-adjacent minor hygiene" / "accepted minor hygiene follow-ups" without an explicit inventory of which minor review findings are accepted into this plan versus deferred. As written, Phase 4 can grow opportunistically.

## Definition of Done Assessment

### Findings

- Most DoD items are concrete, but the hygiene item is not objectively verifiable because the plan never names the accepted hygiene tasks.
- The shell-verification DoD requires a smoke matrix, but it does not define the minimum scenarios that must be present for the changed surfaces (`--cwd`, flagged `cd`, generated-hook fallback/guarding, and PowerShell init/menu fallback). That weakens closure for a shell-boundary remediation plan.

## Phase Structure Assessment

| Phase | Title | Verdict | Issue |
| ----- | ----- | ------- | ----- |
| 1 | Harden Persistence Writes | Good | Correct first phase; isolated, high-impact, and testable. |
| 2 | Fix Menu CWD and Flagged `cd` Parsing | Good | Cohesive shell-core slice with the right contract focus. |
| 3 | Retire Legacy Hook Prototypes | Good with revision | Order is reasonable, but prototype-retirement verification should say more clearly how Bash vs Zsh deletion is validated. |
| 4 | Finalize Hygiene and PowerShell Decision | Needs revision | Bundles optional exploration, residual hygiene, final shell smoke, and artifact closeout into one phase; this is the least bounded phase and the most likely place for scope creep or late-discovered regressions. |

## Testing Strategy Assessment

### Test Coverage Gaps

- The plan does not yet require phase-local shell smoke checks for Phase 2 and Phase 3, even though both phases change shell-facing behavior/contracts.
- The smoke evidence requirement is missing a minimum scenario list. At a minimum, the plan should name scenarios for `dx menu --cwd`, approved flagged `cd` forms, generated hook guard/fallback behavior, and PowerShell init/menu fallback behavior.
- Phase 3 should define how generated-hook guard coverage maps to the legacy prototypes being deleted, especially since the repo currently has both `scripts/hooks/dx.bash` and `scripts/hooks/dx.zsh` while the cited active guard test is Bash-only.

### Real-World Testing

Real-world / cross-shell testing is **planned**, not waived, which is the right posture. However, it is currently too deferred and too generic: the plan waits until Phase 4 to formalize most smoke evidence, which is risky for shell-boundary changes introduced in Phases 2 and 3.

## Completeness Check

### Findings

- The PowerShell `ProxyCommand` question is directionally framed correctly: it is explicitly optional, evidence-driven, and not allowed to silently expand into a broader shell rewrite. But execution readiness would improve if the plan itself—not only the future implementation plan—spelled out the comparison baseline and hard non-goals for that evaluation.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Major | Scope / DoD | Phase 4 includes "accepted minor hygiene" work, but the accepted minor items are never enumerated. That makes the phase open-ended and the DoD non-objective. | Convert the hygiene bundle into an explicit list of accepted minor findings (or an explicit defer list) before execution. |
| 2 | Major | Testing / Verification | Shell-facing verification is under-specified and back-loaded into Phase 4, despite Phases 2-3 changing shell/menu behavior. | Add phase-level smoke expectations and a minimum shell scenario matrix for the changed contracts. |
| 3 | Minor | PowerShell decision framing | The `ProxyCommand` question is framed with the right default posture, but the adopt/reject rubric is still too implicit at the plan level. | State the explicit comparison baseline and non-goals in the plan/Phase 4 doc: preserve current wrapper semantics, PSReadLine/menu fallback behavior, and do not use `ProxyCommand` as a substitute for the Rust parser fix. |
| 4 | Minor | Phase 3 closure | Prototype retirement criteria are clearer for the current Bash guard path than for the full set of legacy prototype files being deleted. | Clarify whether Zsh prototype removal is covered by generated-hook tests, cross-shell smoke only, or a narrower consciously documented deletion scope. |

## Recommendations

1. Tighten Phase 4 before execution by explicitly listing the accepted minor hygiene items and removing any implicit catch-all cleanup scope.
2. Expand the testing strategy and DoD with required phase-local smoke checks plus a minimum shell scenario list for Phases 2-4.
3. Sharpen the PowerShell `ProxyCommand` gate in the plan itself by documenting the comparison baseline, non-goals, and required evidence for adoption.
4. Clarify exactly how Phase 3 proves safe retirement of all legacy hook prototypes, not just the currently cited Bash guard path.
