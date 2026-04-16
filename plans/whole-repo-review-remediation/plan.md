---
type: planning
entity: plan
plan: "whole-repo-review-remediation"
status: completed
created: "2026-04-16"
updated: "2026-04-19"
---

# Plan: whole-repo-review-remediation

## Objective

Implement the highest-value remediation work from the 2026-04-16 whole-repo code review by fixing durability and shell-boundary correctness issues, deleting legacy non-authoritative hook prototypes, and closing adjacent hygiene debt without regressing cross-shell behavior.

## Motivation

The review found a concentrated set of problems at the persistence and shell/menu boundaries rather than broad architectural decay. Two findings affect correctness directly (`write_atomic_replace` durability and `dx menu --cwd` contract drift), two affect long-term trust in the shell layer (flagged `cd` parsing and tests validating legacy hooks instead of generated hooks), and several smaller issues add maintenance drag. Addressing them through a phased plan gives us a reviewable path to improve safety and clarity while preserving the project's thin-wrapper design.

The user also asked that this remediation explicitly consider deleting the legacy hook prototypes and evaluate whether PowerShell should use a different parsing/flag-detection mechanism via `[System.Management.Automation.ProxyCommand]::Create`. That evaluation is included, but adoption is gated on demonstrated net benefit.

## Requirements

### Functional

- [x] Replace the unsafe atomic-write fallback so bookmark and session persistence never delete the last known-good target unless replacement succeeds.
- [x] Add automated durability coverage for bookmark and session storage failure paths.
- [x] Make `dx menu --cwd` authoritative for `paths` mode by threading explicit cwd through the completion pipeline.
- [x] Support correct menu parsing and replacement targeting for approved flagged `cd` forms, at minimum `cd -L <path>`, `cd -P <path>`, and `cd -- <path>`.
- [x] Repoint hook-guard coverage to generated hooks / `dx init` output and delete legacy `scripts/hooks/*` prototypes if they are confirmed non-authoritative.
- [x] Evaluate `[System.Management.Automation.ProxyCommand]::Create` for PowerShell wrapper fidelity and either adopt a bounded improvement or record a justified rejection.
- [x] Land only the explicitly accepted minor hygiene updates from the review: narrow `Resolver.config` visibility if still warranted, remove or justify the dead `ResolveMode` parameter, document the shared `env_lock` contract, replace bare marker-test `unwrap()` calls with diagnostic `expect(...)`, and add balanced-delimiter coverage for menu-enabled generated scripts.

### Non-Functional

- [x] Preserve the established thin-wrapper architecture across Bash, Zsh, Fish, and PowerShell unless the PowerShell evaluation demonstrates a clear net benefit with no contract regression.
- [x] Introduce no new required runtime dependencies, shell helper tools, or external parsers.
- [x] Keep user-facing behavior unchanged outside the targeted fixes and clarified hook authority.
- [x] Require automated Rust/integration tests plus cross-shell smoke evidence where feasible.
- [x] Keep `docs/`, `plans/`, and verification artifacts aligned with the implemented behavior.
- [x] Preserve current PowerShell wrapper semantics, PSReadLine/menu fallback behavior, and `ConvertFrom-Json` action handling unless the `ProxyCommand` evaluation proves a concrete net improvement.

## Scope

### In Scope

- Replacing the unsafe delete-and-retry path in `src/common::write_atomic_replace` and updating bookmark/session callers and tests accordingly.
- Threading explicit cwd through the menu paths completion path so `dx menu --cwd` matches the documented CLI contract.
- Expanding menu buffer parsing to correctly isolate the path argument region for approved flagged `cd` forms.
- Migrating hook-guard validation to generated hooks and deleting legacy hook prototypes from `scripts/hooks/` if they are confirmed obsolete.
- Applying minor API/test hygiene improvements adjacent to the touched resolver/menu/hook/test seams.
- Time-boxing and resolving the PowerShell `ProxyCommand` decision with explicit adopt/reject criteria.
- Recording automated and shell-smoke verification evidence for the changed shell-facing behavior.

### Out of Scope

- Broad redesign of resolver precedence, navigation semantics, bookmarks UX, or non-targeted menu UX behavior.
- Frecency architecture changes or replacement of the zoxide-first strategy.
- Adding new supported shells or changing the single-crate project architecture.
- Adopting a `ProxyCommand`-based PowerShell design without passing the explicit evaluation gate in this plan.
- Unrelated OpenSpec, planning-framework, or documentation overhauls outside the touched surfaces.

## Definition of Done

- [x] `write_atomic_replace` no longer has a delete-and-retry data-loss path, and targeted tests prove bookmark/session data survives replacement failures.
- [x] `dx menu --cwd` drives `paths` mode candidate resolution in code and automated coverage.
- [x] Menu parsing correctly handles the approved flagged `cd` forms with regression coverage for replace/query boundaries.
- [x] Hook-guard coverage validates generated hook behavior rather than legacy prototypes.
- [x] Legacy `scripts/hooks/*` prototypes are deleted, or a documented blocker/deferral explains why deletion could not be completed.
- [x] The PowerShell `ProxyCommand` decision is resolved with explicit evidence; if adopted, code/tests/smoke evidence are updated atomically. _(Completed via explicit reject decision; adoption-only test item is N/A.)_
- [x] The accepted minor hygiene follow-ups are completed or explicitly deferred with rationale, limited to: `Resolver.config` visibility, dead `ResolveMode` parameter cleanup/justification, `env_lock` contract documentation, hook-marker test diagnostic `expect(...)` updates, and balanced-delimiter coverage for menu-enabled generated scripts.
- [x] Automated tests covering changed behavior pass.
- [x] Cross-shell smoke evidence is recorded in `plans/whole-repo-review-remediation/verification/shell-smoke-matrix.md` for Bash, Zsh, Fish, and PowerShell with `Pass` / `Not Feasible` status and rationale.
- [x] Plan and phase artifacts are updated to match what was actually implemented and verified.

## Testing Strategy

- [x] Add or extend unit tests around atomic-write failure handling, menu buffer parsing, and explicit-cwd completion behavior.
- [x] Add or extend integration tests for `dx menu --cwd` behavior and generated-hook guard coverage.
- [x] Run focused exact-target tests for the changed areas during phase execution so filter-based false-passes are avoided.
- [x] Run `cargo test` after each implementation phase that changes code.
- [x] Record cross-shell smoke verification in `plans/whole-repo-review-remediation/verification/shell-smoke-matrix.md`, using a minimum scenario set covering: `dx menu --cwd` path sourcing, approved flagged `cd` forms, generated-hook guard/fallback authority, and PowerShell init/menu fallback behavior.
- [x] If a `ProxyCommand`-based PowerShell change is adopted, add explicit PowerShell-focused tests and smoke evidence for the new wrapper path. _(N/A: `ProxyCommand` was evaluated and rejected.)_

## Phases

| Phase | Title | Scope | Status |
|-------|-------|-------|--------|
| 1 | Harden Persistence Writes | [Detail](phases/phase-1.md) | completed |
| 2 | Fix Menu CWD and Flagged `cd` Parsing | [Detail](phases/phase-2.md) | completed |
| 3 | Retire Legacy Hook Prototypes | [Detail](phases/phase-3.md) | completed |
| 4 | Finalize Hygiene and PowerShell Decision | [Detail](phases/phase-4.md) | completed |

## Risks & Open Questions

| Risk/Question | Impact | Mitigation/Answer |
|---------------|--------|-------------------|
| A safe atomic-write fix must preserve same-directory replacement semantics on macOS/Linux without introducing new partial-write or cross-device failure modes. | High | Phase 1 implementation plan must define a same-directory replacement strategy and verify failure behavior directly in tests. |
| `dx menu --cwd` may be best fixed by generalizing resolver completion APIs rather than adding a menu-only override path. | Medium | Phase 2 implementation planning must explicitly choose the API shape and justify it against future embeddability. |
| Flagged `cd` grammar differs across shells and may be ambiguous for unsupported/grouped flags. | Medium | Gate Phase 2 on a clearly documented supported grammar and explicit fallback behavior for unsupported forms. |
| Deleting legacy `scripts/hooks/*` may break ad hoc local workflows if any still rely on them. | Medium | Before deletion, migrate remaining tests to generated-hook coverage and confirm no authoritative docs still point to the legacy scripts. |
| `ProxyCommand` may add PowerShell-only complexity without solving the Rust-side menu parsing problem. | Medium | Treat `ProxyCommand` as an evidence-driven evaluation in Phase 4; default to rejecting it unless it beats the current `src/hooks/pwsh.rs` wrapper baseline on real correctness cases while preserving PSReadLine/menu fallback behavior, explicit-wrapper readability, and no-new-dependency constraints. |

## Changelog

### 2026-04-19

- Completed Phase 4 and closed the overall remediation plan.
- Confirmed all five accepted hygiene items are implemented in code (`Resolver.config` visibility tightening, resolver dead `ResolveMode` parameter removal with enum retained where used, `env_lock()` contract docs, hook-marker `expect(...)` diagnostics, and menu-enabled balanced-delimiter tests across Bash/Zsh/Fish/Pwsh).
- Recorded explicit PowerShell `ProxyCommand` reject decision in `plans/whole-repo-review-remediation/verification/proxycommand-eval-phase-4.md` (baseline explicit wrapper preserved).
- Completed Phase 4 smoke rows 17-21 (`Pass` for Bash/Zsh/Fish/Pwsh contract checks; row 21 `Not Applicable` due to reject decision) and clarified evidence wording to match performed verification.
- Re-ran the baseline verify chain and full-suite verification with `cargo test` passing at `293 passed`.
- Finalized Phase 4 implementation review collation and recorded unified acceptance in `reviews/impl-review-phase-4-collated-2026-04-19.md`.

### 2026-04-17

- Reconciled Phase 3 planning artifacts to already-landed code and verified evidence while keeping Phase 3 status `in_progress` pending formal implementation review/collation.
- Recorded that generated-hook runtime coverage now replaces legacy prototype sourcing in `tests/shell_hook_guard.rs` (Bash + Zsh command-not-found guard tests plus Bash `cd` wrapper behavior), with supporting `tests/init_cli.rs` and `src/hooks/mod.rs` exact tests passing.
- Captured prototype-retirement outcome in plan artifacts: `scripts/hooks/dx.bash` and `scripts/hooks/dx.zsh` are deleted and `scripts/hooks/` is now empty.
- Updated shell smoke matrix Phase 3 rows (15-16) to `Pass` with concrete test/deletion evidence for Bash and Zsh authority-transition checks.
- Noted carry-forward verification context for later phase closure: `cargo clippy --all-targets -- -D warnings` currently fails on pre-existing `clippy::let_and_return` at `src/common/mod.rs:104-105` outside the Phase 3 diff.
- Collated Phase 3 implementation review and accepted the phase with no blocking issues; the unified decision is recorded in `plans/whole-repo-review-remediation/reviews/impl-review-phase-3-collated-2026-04-17.md`.
- Marked Phase 3 completed in the plan/phase artifacts while leaving Phase 4 pending user kickoff.
- Collated final Phase 4 implementation-plan review verdicts in `plans/whole-repo-review-remediation/reviews/impl-plan-review-phase-4-collated-2026-04-17.md`; classified reviewer split as minor and resolved by incorporating dissenting actionable concerns directly into `implementation/phase-4-impl.md`.
- Updated the refreshed Phase 4 implementation plan with closeout-context completeness (`plan.md`/`todo.md`), named resolve behavior guardrails, corrected shell-smoke row references (17-21), corrected menu-cli exact-target verify command for the PowerShell handler test, and explicit conditional adopt-path exact-target regression test requirements.
- Marked Phase 4 `in_progress` based on explicit user kickoff decision.

### 2026-04-16

- Plan created from `docs/reviews/whole-repo-review-2026-04-16.md` and grounded with `docs/review-remediation-impact-2026-04-16.md`.
- User selected complex-feature flow, chose to include major findings plus minor hygiene, preferred deleting legacy hook prototypes if obsolete, and requested an evidence-driven evaluation of PowerShell `ProxyCommand`.
- Incorporated parallel plan-review findings: bounded the accepted minor-hygiene list, added a concrete smoke-matrix artifact/scenario baseline, and sharpened the PowerShell `ProxyCommand` evaluation gate.
- Authored and reviewed all four phase implementation plans, collated reviewer findings, and revised the implementation plans plus smoke matrix to close actionability, verification, and shell-scoping gaps before execution.
- Executed Phase 1 persistence hardening and verification work, then completed implementation review with acceptance/approval from all three reviewers.
- Recorded the unified implementation-review decision in `plans/whole-repo-review-remediation/reviews/impl-review-phase-1-collated-2026-04-16.md`, with only minor non-blocking follow-ups explicitly deferred.
- Transitioned execution to Phase 2 (`Fix Menu CWD and Flagged cd Parsing`) and updated phase/todo tracking accordingly while keeping overall plan status active.
- Landed Phase 2 implementation across the planned resolver/menu/parser/test surfaces and verified phase-targeted behavior end-to-end.
- Applied follow-up Phase 2 fixes that resolved the initial review blockers and test-quality gaps (missing smoke evidence, no-trailing-space approved-flag coverage, quoted `-L` parity, stronger explicit-cwd assertions, and higher-level flagged replace-span CLI coverage).
- Recorded Phase 2 shell smoke evidence in `plans/whole-repo-review-remediation/verification/shell-smoke-matrix.md` (rows 9-14), including Bash/Zsh/PowerShell `Pass` evidence and Fish `Not Feasible` rationale.
- Completed final Phase 2 implementation review round; all three final reviewers accepted/approved Phase 2, and the unified decision is recorded in `plans/whole-repo-review-remediation/reviews/impl-review-phase-2-collated-2026-04-16.md`.
- Transitioned active execution to Phase 3 (`Retire Legacy Hook Prototypes`) and updated phase/todo tracking while keeping plan status active.
