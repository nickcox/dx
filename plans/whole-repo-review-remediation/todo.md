---
type: planning
entity: todo
plan: "whole-repo-review-remediation"
updated: "2026-04-19"
---

# Todo: whole-repo-review-remediation

> Tracking [whole-repo-review-remediation](plan.md)

## Active Phase: Plan Completed

### Phase Context

- **Scope**: [Phase 4](phases/phase-4.md)
- **Implementation**: [Phase 4 Plan](implementation/phase-4-impl.md)
- **Latest Handover**: _Not created yet_
- **Relevant Docs**:
  - `plans/whole-repo-review-remediation/reviews/impl-review-phase-4-collated-2026-04-19.md`
  - `plans/whole-repo-review-remediation/reviews/impl-review-phase-3-collated-2026-04-17.md`
  - `plans/whole-repo-review-remediation/reviews/impl-plan-review-phase-4-collated-2026-04-17.md`
  - `plans/whole-repo-review-remediation/reviews/impl-review-phase-2-collated-2026-04-16.md`
  - `plans/whole-repo-review-remediation/reviews/impl-review-phase-1-collated-2026-04-16.md`
  - `plans/whole-repo-review-remediation/reviews/impl-plan-review-collated-2026-04-16.md`
  - `docs/reviews/whole-repo-review-2026-04-16.md`
  - `docs/review-remediation-impact-2026-04-16.md`
  - `plans/whole-repo-review-remediation/verification/shell-smoke-matrix.md`

### Pending

- _None._

### In Progress

- _None._

### Completed

- [x] Create the remediation plan scaffold for the whole-repo review findings. <!-- completed: 2026-04-16 -->
- [x] Run parallel review on the remediation plan and incorporate accepted findings. <!-- completed: 2026-04-16 -->
- [x] Author implementation plans for Phases 1-4 against the current code/docs baseline. <!-- completed: 2026-04-16 -->
- [x] Review implementation plans, collate feedback, and revise the phase implementation plans. <!-- completed: 2026-04-16 -->
- [x] Execute Phase 1 persistence hardening and verify durability behavior. <!-- completed: 2026-04-16 -->
- [x] Run Phase 1 implementation reviews (reviewer-1/2/3), collate verdicts, and accept Phase 1 with non-blocking deferred follow-ups only. <!-- completed: 2026-04-16 -->
- [x] Transition active execution from Phase 1 to Phase 2 after implementation review acceptance. <!-- completed: 2026-04-16 -->
- [x] Thread explicit cwd through menu `paths` completion pipeline (`src/cli/menu.rs` → `src/menu/mod.rs` → `src/resolve/completion.rs`) via the shared completion API seam defined in the Phase 2 implementation plan. <!-- completed: 2026-04-16 -->
- [x] Implement parser support for approved flagged forms: `cd -L <path>`, `cd -P <path>`, `cd -- <path>`, including interactive pre-path states (`cd -P ` / `cd -- `). <!-- completed: 2026-04-16 -->
- [x] Add regression coverage proving unsupported/grouped/lone-dash forms (`cd -`, `cd -Q foo`, `cd -LP foo`, `cd -abc foo`) fall back to noop/native behavior. <!-- completed: 2026-04-16 -->
- [x] Add/extend integration coverage showing `dx menu --cwd <path>` in `paths` mode sources candidates from explicit cwd, not process cwd. <!-- completed: 2026-04-16 -->
- [x] Land follow-up Phase 2 test-quality slice: approved-flag no-trailing-space fallback coverage, quoted `-L` coverage parity, stronger explicit-cwd assertions, and CLI-level flagged replace-span test. <!-- completed: 2026-04-16 -->
- [x] Capture Phase 2 shell smoke evidence (Bash/Zsh/PowerShell + Fish pass/not-feasible rationale) in `verification/shell-smoke-matrix.md` for explicit cwd and flagged-`cd` behavior. <!-- completed: 2026-04-16 -->
- [x] Run final Phase 2 implementation re-review (reviewer-1/2/3), collate verdicts, and accept Phase 2 with no blocking implementation issues. <!-- completed: 2026-04-16 -->
- [x] Transition active execution from Phase 2 to Phase 3 after final implementation review acceptance. <!-- completed: 2026-04-16 -->
- [x] Add generated-hook/runtime replacement coverage so `tests/shell_hook_guard.rs` no longer sources `scripts/hooks/*`, using `dx init bash --command-not-found` / `dx init zsh --command-not-found` authoritative surfaces. <!-- completed: 2026-04-17 -->
- [x] Ensure replacement coverage demonstrates Bash guard-equivalent behaviors and validates Zsh deletion rationale via generated-hook tests and smoke evidence. <!-- completed: 2026-04-17 -->
- [x] Delete `scripts/hooks/dx.bash` and `scripts/hooks/dx.zsh` after replacement coverage passed and verification succeeded. <!-- completed: 2026-04-17 -->
- [x] Update Phase 3 shell-smoke matrix rows for Bash/Zsh authority-transition evidence with concrete `Pass` notes. <!-- completed: 2026-04-17 -->
- [x] Reconcile Phase 3 plan/phase/implementation artifacts to the already-landed code state and verified test evidence, including corrected unit verify invocation (`cargo test --lib hooks::tests::all_shells_freeze_command_not_found_guard_contract_markers -- --exact`). <!-- completed: 2026-04-17 -->
- [x] Run Phase 3 implementation review (reviewer-1/2/3), collate verdicts, and accept Phase 3 with no blocking implementation issues. <!-- completed: 2026-04-17 -->
- [x] Execute the bounded Phase 4 hygiene slice from `implementation/phase-4-impl.md` (Resolver visibility/mode cleanup, `env_lock` docs, hook `expect(...)` diagnostics, menu-enabled delimiter coverage) while preserving resolve default/list/json behavior guardrails. <!-- completed: 2026-04-19 -->
- [x] Run the time-boxed PowerShell `ProxyCommand` evaluation and record adopt/reject evidence in `verification/proxycommand-eval-phase-4.md`; keep default posture reject unless gate criteria are met. <!-- completed: 2026-04-19 -->
- [x] Complete Phase 4 shell smoke matrix rows 17-21 with concrete evidence and `Pass`/`Not Feasible`/`Not Applicable` outcomes as required. <!-- completed: 2026-04-19 -->
- [x] Perform Phase 4 closeout synchronization across `implementation/phase-4-impl.md`, `phases/phase-4.md`, `plan.md`, and `todo.md` based on implemented reality and recorded verification evidence. <!-- completed: 2026-04-19 -->
- [x] Run Phase 4 implementation review (reviewer-1/2/3), collate verdicts, and accept Phase 4 with closeout synchronization and smoke-evidence wording clarification complete. <!-- completed: 2026-04-19 -->

### Blocked

- _None._

## Changelog

### 2026-04-19

- Closed Phase 4 and marked the remediation plan completed after synchronizing implementation/phase/plan/todo artifacts to implemented reality.
- Moved all Phase 4 execution items from Pending/In Progress to Completed, including the bounded hygiene slice, `ProxyCommand` evaluation artifact, shell smoke rows 17-21 completion, and closeout synchronization.
- Recorded Phase 4 review outcomes (`reviewer1`/`reviewer2` accepted; `reviewer3` needs-rework on artifact sync/evidence wording), then collated a unified accepted decision in `reviews/impl-review-phase-4-collated-2026-04-19.md` after landing the closeout updates.
- Confirmed baseline verify rerun context with full-suite `cargo test` pass at `293 passed`.

### 2026-04-16

- Created the four-phase remediation plan covering persistence safety, menu/parser correctness, legacy hook prototype deletion, and final hygiene plus PowerShell `ProxyCommand` evaluation.
- Collated plan-review feedback and revised the plan to bound Phase 4 hygiene scope, add concrete smoke-matrix expectations, and tighten Phase 3 prototype-retirement gates.
- Authored implementation plans for Phases 1-4, ran parallel implementation-plan reviews, and revised the implementation plans plus smoke matrix based on the collated findings.
- Accepted Phase 1 implementation after all three reviewers approved with only non-blocking follow-ups; recorded unified decision in `reviews/impl-review-phase-1-collated-2026-04-16.md`.
- Marked Phase 1 artifacts completed, moved Phase 1 execution/review items to Completed, and transitioned active work context to Phase 2 (`Fix Menu CWD and Flagged cd Parsing`).
- Seeded Phase 2 execution backlog with implementation-plan-grounded items covering explicit cwd threading, flagged-`cd` grammar support, fallback behavior, and smoke-matrix evidence capture.
- Recorded final Phase 2 acceptance after follow-up fixes closed initial blockers/test-quality gaps; collated in `reviews/impl-review-phase-2-collated-2026-04-16.md`.
- Moved completed Phase 2 execution/review/verification work into Completed and transitioned active phase tracking to Phase 3 (`Retire Legacy Hook Prototypes`).
- Replaced active backlog with Phase 3 implementation-plan-grounded items covering generated-hook/runtime migration, prototype deletion gates, and Bash/Zsh authority-transition smoke evidence.

### 2026-04-17

- Marked the Phase 3 execution slice complete based on landed code and verified evidence: generated-hook runtime tests (Bash/Zsh), init-cli handler assertion, hook contract marker test, prototype deletion state, and full `cargo test` pass.
- Replaced the Phase 3 active backlog with implementation review/collation as the next in-progress item while keeping overall plan/phase status active pending formal review acceptance.
- Recorded corrected verify invocation guidance (`cargo test --lib hooks::tests::all_shells_freeze_command_not_found_guard_contract_markers -- --exact`) to avoid filter false-pass risk.
- Collated Phase 3 implementation review artifacts, accepted Phase 3 with no blocking issues, and linked the unified decision in `reviews/impl-review-phase-3-collated-2026-04-17.md`.
- Shifted todo context to Phase 4 while leaving kickoff user-gated; Phase 4 remains pending until the user chooses whether to continue.
- User chose to begin Phase 4; replaced kickoff-waiting item with execution backlog aligned to the refreshed implementation plan and marked exactly one execution item in progress.
- Added Phase 4 implementation-plan review references and the collated acceptance-for-execution decision artifact (`reviews/impl-plan-review-phase-4-collated-2026-04-17.md`) to active phase context.
