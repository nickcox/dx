---
type: review
entity: plan
plan: "whole-repo-review-remediation"
reviewer: reviewer-2
date: 2026-04-16
verdict: Approved
---

# Plan Review: whole-repo-review-remediation (Reviewer 2)

## Verdict

**Approved.** The plan is highly comprehensive, well-grounded in the codebase realities, and directly addresses the findings from the whole-repo review without overextending scope. The phrasing of the PowerShell `ProxyCommand` evaluation as a gated, evidence-driven decision in Phase 4 is particularly strong.

## Assessment

### 1. Scope and Definition of Done
The scope accurately captures all Major and Minor findings from the 2026-04-16 code review. The DoD is concrete, testable, and strictly tied to the remediation goals. The inclusion of the PowerShell evaluation and legacy hook deletion properly captures the user's specific requests while keeping them bounded.

### 2. Phase Boundaries and Ordering
Phase boundaries are distinct and logical. However, there is a minor optimization to be made regarding the ordering of Phase 2 and Phase 3 (see Finding 1).

### 3. PowerShell ProxyCommand Framing
Framing this as a time-boxed evaluation in Phase 4 with explicit adopt/reject criteria is an excellent way to contain risk. It ensures the Rust core correctness fixes (Phases 1 and 2) land regardless of whether the PowerShell wrapper architecture changes.

### 4. Verification Expectations
The testing strategy is rigorous. Mandating failure-injection tests for persistence and a cross-shell smoke matrix for CLI/hook changes ensures that the remediations will be verifiable and durable.

## Findings

### 1. Phase Ordering: Consider swapping Phase 2 and Phase 3 (Minor)
**Description:** Phase 3 migrates the test safety net (`tests/shell_hook_guard.rs`) from legacy prototype scripts to the actual generated hooks. Phase 2 introduces parsing changes at the shell/menu boundary.
**Impact:** If Phase 2 modifies how the menu behaves and interacts with the shell hooks, it would be safer to have the guard tests validating the *real* generated hooks (Phase 3's goal) *before* making the Phase 2 changes.
**Recommendation:** Consider moving the test migration/legacy deletion (current Phase 3) to run before the parsing fixes (current Phase 2), so the safety net is fully aligned with production artifacts before modifying behavior.

### 2. Flagged `cd` Degradation Clarity (Minor)
**Description:** Phase 2 mentions supporting approved flagged `cd` forms and falling back for unsupported forms. Given the variance in `cd` flags across shells (e.g., `-e`, `-@` on macOS/zsh), "predictable fallback" is critical.
**Impact:** If the parser misidentifies a flag, it could lead to broken paths being passed to the shell.
**Recommendation:** Ensure the Phase 2 implementation plan strictly defines that any unrecognized flag sequence immediately aborts the `dx menu` intervention and falls back to the native shell `cd` completion/execution.

## Summary

This is a very solid remediation plan. The strict boundaries around what will *not* be done (e.g., no broad redesigns) keep it actionable. Once the minor findings are considered, it is ready for execution.
