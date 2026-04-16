---
type: planning
entity: implementation-review-collated
plan: "whole-repo-review-remediation"
phase: 3
date: "2026-04-17"
status: accepted
---

# Collated Implementation Review: Phase 3 (2026-04-17)

## Inputs

- Three independent reviewer reports were used during collation; raw reviewer artifacts were later pruned during process-documentation cleanup.
- [Phase 3](../phases/phase-3.md)
- [Phase 3 Implementation Plan](../implementation/phase-3-impl.md)
- [Shell Smoke Matrix](../verification/shell-smoke-matrix.md)

## Reviewer Verdict Summary

| Reviewer | Final Verdict | Blocking Issues? | Notes |
|----------|---------------|------------------|-------|
| reviewer-1 | Conditional Pass | No | Low-severity follow-ups only: Bash runtime helper uses `bash -lc`, and `tech-docs/shell-hook-guarding.md` still refers to legacy prototypes in present tense after deletion. |
| reviewer-2 | Pass | No | Confirms the generated-hook authority transition is functionally correct; only notes the pre-existing clippy lint outside the Phase 3 diff. |
| reviewer-3 | Accepted | No | Confirms Phase 3 criteria are met; notes smoke rows are recorded automated real-shell/runtime evidence rather than separate manual shell transcripts. |

Consensus: all three reviewers accept Phase 3 with no blocking implementation issues.

## Agreements

- `tests/shell_hook_guard.rs` now validates generated hook output from `dx init <shell> --command-not-found` instead of sourcing `scripts/hooks/*`.
- The Bash and Zsh authority-transition evidence is sufficient: exact runtime tests pass, `tests/init_cli.rs` covers Zsh handler emission, and `hooks::tests::all_shells_freeze_command_not_found_guard_contract_markers` passes via the corrected `cargo test --lib ... -- --exact` invocation.
- `scripts/hooks/dx.bash` and `scripts/hooks/dx.zsh` are deleted and `scripts/hooks/` is now empty.
- Full-suite verification passed (`cargo test` with `289 passed`).
- `cargo clippy --all-targets -- -D warnings` currently fails only because of a pre-existing `clippy::let_and_return` finding in `src/common/mod.rs:104-105`, outside the Phase 3 diff.

## Differences in Emphasis

- Reviewer-1 is stricter about hermetic test execution and documentation wording, preferring follow-up work on Bash's `-lc` runtime helper and `tech-docs/shell-hook-guarding.md` wording before calling the phase unconditional.
- Reviewer-3 emphasizes that Phase 3 smoke rows are best described as recorded automated real-shell/runtime verification, not broad manual interactive smoke.

These are non-blocking differences in emphasis, not substantive disagreements about Phase 3 correctness or acceptance.

## Unified Decision

**Decision: Phase 3 is accepted and complete. Phase 4 remains pending until the user chooses whether to continue.**

Rationale: the reviewer trio agrees that the generated-hook authority transition landed correctly, legacy prototypes were retired safely after coverage migration, and the corrected verification record is sufficient for close-out. The remaining notes are low-severity follow-ups and do not justify holding the phase open or silently expanding the bounded Phase 4 scope.
