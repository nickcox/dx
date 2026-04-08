---
type: review
entity: plan-review
plan: "architectural-doc-code-alignment"
status: final
reviewer: "general"
created: "2026-04-08"
---

# Plan Review: architectural-doc-code-alignment

> Reviewing [architectural-doc-code-alignment](../plan.md)

## Overall Assessment

**Verdict**: Needs Revision

The plan is pointed at real, code-backed drift and is architecturally aligned with the current thin-wrapper/Rust-owned navigation model. However, it is not yet execution-ready because the contract-adjudication output, the shell-boundary success criteria, and the cross-shell smoke matrix are still too implicit for a high-risk doc/code alignment effort.

## Requirement Coverage

| Requirement | Covered By | Gap? | Notes |
| ----------- | ---------- | ---- | ----- |
| Adjudicate architecture conflicts case by case | Phase 1 | Yes | Decision process is mentioned, but the authoritative output/location for those decisions is not explicit. |
| Refresh `docs/cd-extras-cli-prd.md` | Phase 1 | No | Directly owned by Phase 1. |
| Refresh shell-hook documentation | Phase 1, Phase 3 | No | Covered, with final verification-driven cleanup in Phase 3. |
| Align menu noop/error handling across shells | Phase 2, Phase 3 | Yes | Covered, but pass/fail behavior is still not specified as a concrete shell-by-shell matrix. |
| Harden shell-to-`dx menu` boundary without new deps | Phase 2 | Yes | Intent is covered, but acceptance criteria do not define the target payload/parsing contract concretely. |
| Preserve thin-wrapper architecture and current command contracts | Plan scope, Phase 1, Phase 2 | No | Explicitly protected and matches current code architecture. |
| Add/update tests and smoke procedures for all four shells | Phase 2, Phase 3 | Yes | Present, but under-specified for highest-risk quoting/cancel/no-TTY flows. |
| No new required dependencies | Plan scope, Phase 2 | No | Explicitly stated. |
| Prefer minimal deltas outside targeted corrections | Plan scope | No | Explicitly stated. |
| Keep current contract vs historical background clear | Phase 1 | Yes | Good intent, but the mechanism for labeling retained legacy context is not fully specified. |
| Include automated Rust tests plus manual/smoke validation | Testing Strategy, Phase 2, Phase 3 | Yes | Present, but evidence expectations and required scenarios need tightening. |
| Maintain graceful behavior when menu disabled/no TTY/no interactive replacement | Plan requirements, Phase 2, Phase 3 | Yes | Covered at a high level; concrete verification cases are missing. |

## Scope Clarity

### Findings

- The overall scope is well targeted, but the plan does not explicitly say where Phase 1's case-by-case adjudication decisions will be recorded so later phases can treat them as authoritative.

## Definition of Done Assessment

### Findings

- Several DoD items remain subjective, especially “robust under the supported escaping/quoting contract” and “matches the approved menu noop/error fallback semantics,” because the plan does not yet define the exact contract or evidence required to prove it.

## Phase Structure Assessment

| Phase | Title | Verdict | Issue |
| ----- | ----- | ------- | ----- |
| 1 | Refresh Architecture Docs and Contracts | Mostly sound | Needs an explicit decision-log/output artifact for adjudicated contracts. |
| 2 | Harden Cross-Shell Menu and Hook Boundaries | Too broad | Bundles contract finalization, four shell implementations, tests, and doc updates into one large phase. |
| 3 | Verify Across Shells and Finalize Documentation | Sound but underspecified | Verification intent is right, but required smoke cases/evidence are not enumerated. |

## Testing Strategy Assessment

### Test Coverage Gaps

- The plan requires testing across Bash, Zsh, Fish, and PowerShell, but it does not define a minimum shell-by-shell scenario matrix for the riskiest paths: replace payload parsing, quoted paths, cancel-with-typed-filter, noop/error fallback, no-TTY behavior, and PowerShell PSReadLine-dependent flows.
- Automated testing is called for, but there is no explicit requirement to add regression cases for the current string-slicing boundary used in Bash/Zsh/Fish versus JSON parsing in PowerShell, even though that boundary is the plan's highest-risk implementation area.

### Real-World Testing

Real-world testing is **present and required**, not waived: the plan explicitly calls for manual or smoke verification in Bash, Zsh, Fish, and PowerShell. That said, the real-world testing bar is not yet execution-ready because the plan does not define the exact smoke scenarios, environment assumptions, or evidence to capture for each shell.

## Reference Consistency

### Findings

- The plan correctly targets real drift in the repo (for example, `docs/cd-extras-cli-prd.md` still describes obsolete `dx add`/`undo`/`redo` and generic completion contracts, while the codebase uses `dx stack ...`, `dx navigate`, mode-based `dx complete`, and `dx menu`).
- The plan is also grounded in a real shell-hook inconsistency: `docs/shell-hook-guarding.md` currently says stack wrappers use `dx undo`/`dx redo`, while the code uses `dx stack undo`/`dx stack redo`. This supports the plan, but also raises the need for a more explicit adjudication artifact in Phase 1.

## Completeness Check

### Findings

- A new implementer could understand the intent of Phase 1, but would still need follow-up decisions before safely executing Phase 2 because the approved boundary contract and the proof obligations for shell verification are not yet explicit.
- Risks are identified well, but the mitigation for the payload-format decision is deferred to the implementation-plan step rather than narrowed enough here to make the plan fully actionable.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Major | Scope / Actionability | Phase 1 does not specify where adjudicated contract decisions are recorded and how they become the authoritative baseline for later phases. | Add an explicit Phase 1 output (for example, a decision-log section or named contract artifact) that later phases must reference. |
| 2 | Major | Definition of Done / Architecture | The plan leaves the menu boundary success criteria too subjective by not defining the target payload/parsing contract or the exact conditions that count as “safe” quoting/escaping behavior. | State the intended boundary contract at plan level, or require a concrete decision gate and acceptance checklist before Phase 2 implementation begins. |
| 3 | Major | Testing Strategy | Cross-shell real-world testing is required but under-specified for the highest-risk behaviors, especially cancel/noop/error/no-TTY flows and quoting-sensitive replacements. | Add a shell-by-shell smoke matrix with required scenarios, expected outcomes, and evidence to record. |
| 4 | Minor | Phase Structure | Phase 2 is likely oversized for a single clean execution slice because it combines contract hardening, per-shell hook changes, tests, and incidental docs updates. | Split Phase 2 into smaller execution slices or explicitly bound its file/surface area. |

## Recommendations

1. Define the authoritative output of Phase 1 adjudication and require Phase 2/3 to cite it.
2. Tighten the DoD around the `dx menu` boundary by naming the contract and the acceptance checks for escaping/fallback behavior.
3. Add a required cross-shell smoke matrix covering replace, cancel-with-query, noop, error, no-TTY, and PowerShell init/PSReadLine cases.
4. Reduce Phase 2 scope or pre-split it so implementation and review stay tractable.
