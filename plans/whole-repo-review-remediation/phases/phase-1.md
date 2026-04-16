---
type: planning
entity: phase
plan: "whole-repo-review-remediation"
phase: 1
status: completed
created: "2026-04-16"
updated: "2026-04-16"
---

# Phase 1: Harden Persistence Writes

> Part of [whole-repo-review-remediation](../plan.md)

## Objective

Eliminate the data-loss path in persistence writes so bookmark and session state keep the last known-good file when replacement fails.

## Scope

### Includes

- Reworking `src/common::write_atomic_replace` to remove the delete-and-retry fallback.
- Updating bookmark/session storage callers only as needed to support the safer replacement contract.
- Adding targeted failure-mode tests that prove the old target survives replacement failures.

### Excludes (deferred to later phases)

- Menu cwd or flagged `cd` parsing changes.
- Hook authority cleanup and legacy prototype deletion.
- PowerShell `ProxyCommand` evaluation.

## Prerequisites

- [x] The plan and Phase 1 implementation plan are reviewed and accepted for execution.

## Deliverables

- [x] A safe replacement implementation for persistence writes.
- [x] Updated bookmark/session storage tests covering replacement failures.
- [x] Verification evidence showing the new durability behavior passes targeted and full test suites.

## Acceptance Criteria

- [x] No code path deletes an existing bookmark/session target before the replacement operation itself succeeds.
- [x] Targeted tests fail on the old unsafe behavior and pass on the new implementation.
- [x] `cargo test` passes after the phase completes.

## Dependencies on Other Phases

| Phase | Relationship | Notes |
|-------|-------------|-------|
| 2 | blocks | Persistence hardening lands first because it is the highest-severity correctness fix. |
| 3 | blocks | Hook cleanup should not start until the persistence baseline is safe and verified. |
| 4 | blocks | Finalization and hygiene phase depends on earlier correctness work being complete. |

## Notes

- The review identified this as the highest-impact issue because it affects persisted user state.
- Same-directory temp-file behavior should remain explicit to avoid cross-device replacement surprises.
- Completion evidence:
  - Implementation landed per [Phase 1 implementation plan](../implementation/phase-1-impl.md).
  - Collated implementation-review decision: [accepted with non-blocking notes](../reviews/impl-review-phase-1-collated-2026-04-16.md).
  - Reviewer evidence includes targeted durability tests plus full suite pass (`cargo test`) in reviewer-3 verification notes.
  - Deferred notes (non-blocking): temp-file cleanup assertion gap, consume-once seam invariant documentation, fsync-before-rename limitation documentation, and lack of real filesystem-level replace-failure test.
