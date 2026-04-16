---
type: planning
entity: implementation-plan-review-collated
plan: "whole-repo-review-remediation"
phase: 4
date: "2026-04-17"
status: accepted-for-execution
---

# Collated Implementation-Plan Review: Phase 4 (2026-04-17)

## Inputs

- Three independent reviewer reports were used during collation; raw reviewer artifacts were later pruned during process-documentation cleanup.
- [Phase 4](../phases/phase-4.md)
- [Phase 4 Implementation Plan](../implementation/phase-4-impl.md)
- [Shell Smoke Matrix](../verification/shell-smoke-matrix.md)

## Reviewer Verdict Summary

| Reviewer | Final Verdict | Blocking Issues? | Notes |
|----------|---------------|------------------|-------|
| reviewer-1 | Conditional Pass | No | Low-severity consistency/test-target notes; execution-time guardrails recommended. |
| reviewer-2 | Pass | No | Plan is bounded and executable; one low-severity note on conditional-test clarity. |
| reviewer-3 | Needs Revision | No blockers found in scope, but requested explicit closeout/test guardrails | Requested concrete revisions in Required Context, resolve guardrails, smoke-row references, and conditional adopt-path verification specificity. |

Consensus: this is a **minor split**, not a fundamental planning disagreement. Reviewer-3 concerns are directly actionable refinements and have been incorporated into the refreshed implementation plan.

## Dissenting Concerns and Incorporated Revisions

1. **Closeout context completeness**
   - Concern: `plan.md` and `todo.md` were missing from Required Context despite Phase 4 closeout deliverables.
   - Incorporated: Added both files to Required Context in `implementation/phase-4-impl.md` and reinforced synchronized closeout in Step 4 considerations.

2. **Resolve behavioral guardrails when touching `ResolveMode` plumbing**
   - Concern: Step 1 lacked named behavioral guardrails for default/list/json resolve behavior.
   - Incorporated: Added `tests/resolve_cli.rs` and `tests/resolve_precedence.rs` to Required Context; Step 1 and Test Integrity now require preserving `resolve_cli` default/list/json behavior and name exact guardrail tests.

3. **Shell smoke matrix row references were off-by-one**
   - Concern: Phase 4 implementation plan referenced `:16-20` instead of actual Phase 4 range.
   - Incorporated: Corrected all Phase 4 row references to `17-21` (including Required Context, Step 4, and Reality Check anchors).

4. **Verify command target bug**
   - Concern: `init_pwsh_with_menu_flag_includes_psreadline_handler` resides in `tests/menu_cli.rs`, not `tests/init_cli.rs`.
   - Incorporated: Baseline verify command now uses `cargo test --test menu_cli init_pwsh_with_menu_flag_includes_psreadline_handler -- --exact`.

5. **Conditional adopt-path PowerShell regression test requirement**
   - Concern: ProxyCommand adoption branch needed at least one named exact-target regression test.
   - Incorporated: Step 3 and Test Integrity now explicitly require a named exact-target PowerShell regression test if adoption occurs (planned target: `proxycommand_adopted_preserves_psreadline_fallback` in `tests/menu_cli.rs`).

## Scope/Deferral Validation

The refreshed plan preserves bounded scope and keeps prior defer-by-default decisions intact unless directly touched by Phase 4 edits:

- Pre-existing clippy lint at `src/common/mod.rs:104-105` remains deferred by default.
- Bash `bash -lc` hermeticity follow-up remains deferred by default.
- Stale `tech-docs` wording follow-up remains deferred by default.

## Unified Decision

**Decision: Accept the refreshed Phase 4 implementation plan for execution now.**

Rationale: reviewer split was low-severity and execution-actionable; all dissenting concerns have now been incorporated directly into `implementation/phase-4-impl.md` without expanding scope beyond the gated Phase 4 intent.
