---
type: planning
entity: implementation-review-collated
plan: "whole-repo-review-remediation"
phase: 4
date: "2026-04-19"
status: accepted
---

# Collated Implementation Review: Phase 4 (2026-04-19)

## Inputs

- Three independent reviewer reports were used during collation; raw reviewer artifacts were later pruned during process-documentation cleanup.
- [Phase 4](../phases/phase-4.md)
- [Phase 4 Implementation Plan](../implementation/phase-4-impl.md)
- [ProxyCommand Evaluation](../verification/proxycommand-eval-phase-4.md)
- [Shell Smoke Matrix](../verification/shell-smoke-matrix.md)

## Reviewer Verdict Summary

| Reviewer | Final Verdict | Blocking Issues? | Notes |
|----------|---------------|------------------|-------|
| reviewer-1 | ACCEPT | No | Requested closeout artifact status sync, Fish smoke evidence-basis clarification, and impl-plan note clarifying `ResolveMode` parameter removed while enum remains used. |
| reviewer-2 | ACCEPT | No | Confirms bounded hygiene slice landed, ProxyCommand rejection is well-evidenced, and verification is complete. |
| reviewer-3 | NEEDS REWORK | Closeout-only | Flags unsynchronized closeout artifacts and smoke wording emphasis mismatch; no blocking code-correctness defects identified. |

Majority accepted implementation correctness; dissent focused on closeout synchronization and evidence phrasing.

## Agreements

- All five accepted hygiene items are implemented:
  1. `Resolver.config` tightened to `pub(crate)` (`src/resolve/mod.rs`).
  2. Dead `ResolveMode` parameter removed from `Resolver::resolve` (`src/resolve/pipeline.rs`), while mode usage remains in `src/resolve/output.rs`.
  3. `env_lock()` contract documented (`src/test_support.rs`).
  4. Hook marker test unwraps replaced with `expect(...)` diagnostics (`src/hooks/mod.rs`).
  5. Menu-enabled balanced-delimiter tests exist for Bash/Zsh/Fish/Pwsh (`src/hooks/mod.rs`).
- `ProxyCommand` evaluation was completed and explicit rejection is sound (`../verification/proxycommand-eval-phase-4.md`).
- Baseline verify command rerun succeeded, ending with full-suite `cargo test` pass (`293 passed`).
- Shell smoke matrix rows 17-20 are complete and row 21 is `Not Applicable` due to explicit rejection.

## Differences in Emphasis

- Reviewer-3 treated closeout artifact drift and smoke wording overstatement as a gating issue for acceptance until synchronized.
- Reviewer-1 similarly requested clearer wording for Fish/Bash/Zsh evidence basis and an explicit implementation-plan note that `ResolveMode` enum retention is intentional (parameter removal only).
- Reviewer-2 considered the same items non-blocking and accepted immediately.

These are minor closeout/documentation alignment differences, not a fundamental disagreement about implementation correctness.

## Unified Decision

**Decision: Phase 4 is accepted and complete after closeout synchronization and shell-smoke evidence wording clarification.**

Rationale: reviewer consensus on code correctness and verification remained intact; dissent concerns were limited to artifact synchronization and wording precision. Those closeout updates are now reflected across `implementation/phase-4-impl.md`, `phases/phase-4.md`, `plan.md`, `todo.md`, and `verification/shell-smoke-matrix.md`. No blocking code issues remain.
