---
type: plan-review
reviewer: reviewer-1
plan: whole-repo-review-remediation
date: 2026-04-16
verdict: Accepted with Minor Revisions
---

# Plan Review: whole-repo-review-remediation (Reviewer 1)

## Verdict

**Accepted with Minor Revisions.** The plan is well-grounded in the source review and impact artifacts, phase sequencing is sound, and the scope/DoD correctly maps to the review findings and stated user preferences. Several gaps are worth addressing before execution begins: Phase 3's prerequisite is under-specified relative to its risk, the "minor hygiene" bundle in Phase 4 is vague enough to create scope-creep ambiguity, and cross-shell smoke evidence expectations are mentioned in the Testing Strategy but not given a concrete minimum matrix in any phase doc.

---

## Findings

### F1 — MAJOR: Phase 3 prerequisite does not gate on *coverage equivalence*, only on Phase 2 completion

**Location:** `phases/phase-3.md` → Prerequisites, Acceptance Criteria

**Detail:** The prerequisite "Phase 2 is completed and verified" does not ensure that replacement generated-hook coverage is in place *before* any deletion is attempted. The Acceptance Criteria entry "No active tests source `scripts/hooks/*`" describes the end state but does not define what constitutes sufficient replacement coverage. The risk noted in the impact doc — "removing `scripts/hooks/*` requires first migrating guard assertions to generated-hook paths … to avoid coverage regression" — is acknowledged in the plan's Risks table but not translated into a checkable phase gate.

As confirmed in the code, `tests/shell_hook_guard.rs` currently has hardcoded absolute paths to `scripts/hooks/dx.bash`; deleting these files without first rewriting the tests would break the entire shell-guard test suite.

**Recommendation:** Add an explicit prerequisite to Phase 3: "Replacement generated-hook tests cover every behavior currently asserted against `scripts/hooks/dx.bash` and `scripts/hooks/dx.zsh`, and `cargo test` passes on those replacement tests." This should be a Phase 3 implementation-plan gate, not assumed from Phase 2 completion.

---

### F2 — MAJOR: No minimum cross-shell smoke matrix is specified anywhere in the phase docs

**Location:** `plan.md` → Testing Strategy; `phases/phase-2.md`, `phases/phase-4.md`

**Detail:** The Testing Strategy requires "cross-shell smoke verification using a matrix artifact" and Phase 4 Acceptance Criteria list Bash/Zsh/Fish/PowerShell results. However, no phase document specifies the minimum scenario set that constitutes sufficient smoke evidence, nor where the matrix artifact will live. The prior `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` is referenced but not adopted as a template. Phase 2 changes the menu/parser behavior that affects all four shells, yet Phase 2's own Acceptance Criteria contain zero shell-smoke expectations.

**Recommendation:** Either reference an explicit minimum smoke scenario set in the Phase 4 Acceptance Criteria, or add a brief "Verification Evidence" subsection in Phase 2 requiring at minimum: Bash/Zsh `dx menu --cwd <X>` paths mode runs against known directory and confirms candidates, and flagged `cd -P` / `cd --` buffer replacement targets the path token correctly (where manually verifiable). Keep it proportional but concrete.

---

### F3 — MINOR: "Minor hygiene" scope in Phase 4 is under-bounded

**Location:** `phases/phase-4.md` → Scope → Includes; `plan.md` → Requirements (Functional, last bullet)

**Detail:** "Applying minor resolver/test hygiene fixes that de-risk or clarify the touched surfaces" and "Land directly-adjacent minor hygiene updates from the review where they simplify or de-risk the main remediation work" are open-ended. The review identifies five specific items (F5: `Resolver.config` visibility, F6: dead `_mode` parameter, F7a: env-lock doc, F7b: `expect()` in marker tests, F7c: delimiter-balance for menu=true scripts). Without an explicit list in Phase 4 of which items are in-scope and which are explicitly deferred, there is no way to close the phase definitively.

**Recommendation:** Add a table or checklist to `phases/phase-4.md` enumerating the specific hygiene items that are in scope for this phase, and mark which are deferred-with-rationale. This matches the DoD requirement for "explicitly deferred with rationale."

---

### F4 — MINOR: Phase 2 does not explicitly define the supported `cd` grammar before execution

**Location:** `phases/phase-2.md` → Prerequisites; `plan.md` → Risks & Open Questions

**Detail:** The phase prerequisite says "The supported flagged-`cd` grammar and fallback behavior are explicitly approved in the Phase 2 implementation plan." This correctly gates on the implementation plan, which does not yet exist. However, the phase doc itself has no placeholder for *what the grammar decision must resolve* — i.e., the three open options (`-P`, `-L`, `--`, and whether grouped/unknown flags fall back). This makes it possible to author an implementation plan that silently skips the grammar decision.

**Recommendation:** Add a "Grammar Decision Required" note to Phase 2 prerequisites explicitly listing the options from the impact doc (`-P`, `-L`, `--`, grouped/unknown flags) and requiring the implementation plan to specify accepted and rejected forms before any buffer.rs code is changed.

---

### F5 — MINOR: Phase dependency rationale for Phase 3 blocking Phase 4 is superficially motivated

**Location:** `phases/phase-3.md` → Dependencies on Other Phases; `phases/phase-4.md` → Dependencies

**Detail:** Phase 4 is listed as blocked by Phase 3, with the rationale "Hook authority and legacy cleanup must already be complete." This is defensible but the plan does not explain *why* Phase 4's hygiene and PowerShell evaluation require the hook cleanup to precede them. If the ProxyCommand evaluation is independent of whether legacy scripts still exist, this ordering may introduce unnecessary delay. Conversely, if the cross-shell smoke matrix in Phase 4 requires the cleaned-up hook authority model, that should be stated.

**Recommendation:** Clarify in the Phase 4 dependencies table whether the Phase 3 block is strict (smoke evidence references generated-hook state; legacy hooks being present would contaminate the evidence) or advisory. If it is strict, say so explicitly.

---

### F6 — INFORMATIONAL: PowerShell ProxyCommand framing is appropriate and appropriately scoped

**Location:** `plan.md` → Risks & Open Questions; `phases/phase-4.md`; impact doc §5

**Assessment:** The plan correctly frames the ProxyCommand question as an evidence-driven evaluation with explicit adopt/reject criteria, deferred to Phase 4, with a default posture of rejection unless concrete benefit is demonstrated. The criteria from the impact doc (correctness gains, no regression, maintainability, no new dependencies) are referenced in the plan. The framing does not overreach into architectural redesign. This is well-handled.

---

### F7 — INFORMATIONAL: DoD coverage for all major review findings is confirmed complete

**Assessment:** All four Major findings from the collated review (F1 atomic-write, F2 menu cwd, F3 hook guard drift, F4 flagged cd parsing) map 1:1 to DoD bullets and phase objectives. The DoD also correctly covers the PowerShell decision with adopt/reject evidence. Minor findings F5/F6/F7 are captured in Phase 4's scope (though under-bounded, as noted in F3 above). No review finding is silently dropped.

---

### F8 — INFORMATIONAL: Codebase grounding confirmed for cited locations

**Assessment:** Direct inspection confirmed:
- `src/common/mod.rs:46-63`: The unsafe delete-and-retry path exists exactly as described — line 53 calls `fs::remove_file(target)` before the second `rename`.
- `tests/shell_hook_guard.rs:20-23`: Hard-coded absolute path `"/Users/nick/code/personal/dx/scripts/hooks/dx.bash"` in the test format string, confirming legacy prototype dependency.
- `tech-docs/shell-hook-guarding.md:48-55`: Explicitly declares `src/hooks/bash.rs` et al. as source of truth and states "Legacy prototype scripts under `scripts/hooks/` are not authoritative."
- `tech-docs/cd-extras-cli-prd.md:64`: Documents `--cwd` as a real CLI parameter in the menu boundary contract.

All four major finding locations check out. Plan grounding is solid.

---

## Summary

| # | Severity | Area | Short Description |
|---|----------|------|-------------------|
| F1 | **Major** | Phase 3 prerequisites | Coverage-equivalence gate missing before deletion is allowed |
| F2 | **Major** | Verification | No minimum cross-shell smoke scenario set defined in any phase |
| F3 | Minor | Phase 4 scope | Hygiene bundle is open-ended; no explicit item list or deferral list |
| F4 | Minor | Phase 2 prerequisites | Grammar options not enumerated in phase doc — could be silently skipped in impl plan |
| F5 | Minor | Phase ordering | Phase 4 block on Phase 3 is not justified as strict or advisory |
| F6 | Info | PowerShell question | Framing is appropriate and appropriately scoped |
| F7 | Info | DoD completeness | All major review findings are covered |
| F8 | Info | Codebase grounding | Cited code locations verified correct |

## Overall Assessment

The plan correctly captures the scope, sequencing, and DoD for all major review findings. Phase boundaries are logical and the motivation for the ordering (persistence first, then menu correctness, then cleanup, then finalization) is sound. The PowerShell evaluation is appropriately gated and framed.

The two **Major** gaps are:
1. Phase 3 can be attempted without ensuring replacement test coverage exists — this can cause a coverage regression if deletion happens too early.
2. No concrete cross-shell smoke scenario list exists anywhere; the requirement is real but unactionable as written.

Both are fixable with targeted additions to the phase prerequisites or a brief smoke-minimum table before execution begins. Execution should not start until at minimum the Phase 3 prerequisite gap (F1) is resolved.
