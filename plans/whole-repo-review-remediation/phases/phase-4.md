---
type: planning
entity: phase
plan: "whole-repo-review-remediation"
phase: 4
status: completed
created: "2026-04-16"
updated: "2026-04-19"
---

# Phase 4: Finalize Hygiene and PowerShell Decision

> Part of [whole-repo-review-remediation](../plan.md)

## Objective

Close directly-adjacent hygiene debt, resolve the PowerShell `ProxyCommand` question with evidence, and finalize cross-shell verification and planning artifacts.

## Scope

### Includes

- Applying only these accepted minor hygiene fixes, unless explicitly deferred with rationale: `Resolver.config` visibility tightening (if still appropriate after implementation), dead `ResolveMode` parameter cleanup/justification, `env_lock` contract documentation, hook-marker test `expect(...)` diagnostics, and balanced-delimiter coverage for menu-enabled generated scripts.
- Running a time-boxed evaluation of `[System.Management.Automation.ProxyCommand]::Create` against the project's PowerShell wrapper needs.
- Adopting a bounded PowerShell wrapper improvement only if the evaluation demonstrates clear net benefit; otherwise documenting rejection.
- Completing cross-shell smoke verification and final plan/documentation updates.

### Excludes (deferred to later phases)

- Large new shell architecture work beyond the time-boxed evaluation.
- Any PowerShell change that cannot be verified without expanding scope beyond this plan.

## Prerequisites

- [x] Phase 3 is completed and verified.
- [x] The `ProxyCommand` evaluation criteria are explicitly documented in the Phase 4 implementation plan.

## Deliverables

- [x] Implemented or explicitly deferred accepted minor hygiene fixes adjacent to the main remediation work.
- [x] A documented `ProxyCommand` decision with supporting evidence.
- [x] Cross-shell smoke matrix / verification evidence for the completed changes.
- [x] Finalized plan/todo/implementation artifacts reflecting completion state.

## Acceptance Criteria

- [x] Minor hygiene items included in scope are either completed or explicitly deferred with rationale.
- [x] The `ProxyCommand` evaluation ends in an explicit adopt/reject decision tied to tests and smoke evidence.
- [x] Any adopted `ProxyCommand` change demonstrably outperforms the current `src/hooks/pwsh.rs` baseline on real correctness cases, preserves PSReadLine/menu fallback behavior, does not replace the Rust parser fix from Phase 2, and adds no new runtime dependencies. _(N/A: `ProxyCommand` was explicitly rejected; explicit-wrapper baseline preserved.)_
- [x] Bash, Zsh, Fish, and PowerShell verification results are recorded with `Pass` / `Not Feasible` evidence. _(Rows 17-20 `Pass`; row 21 `Not Applicable` by design after rejection.)_
- [x] The plan can be marked completed with artifacts aligned to implemented reality.

## Dependencies on Other Phases

| Phase | Relationship | Notes |
|-------|-------------|-------|
| 1 | blocked-by | Finalization occurs only after the persistence fix lands. |
| 2 | blocked-by | Shell smoke and PowerShell evaluation depend on the settled menu/parser behavior. |
| 3 | blocked-by | Hook authority and legacy cleanup must already be complete. |

## Notes

- Current recommended posture is to keep the explicit PowerShell wrapper unless `ProxyCommand` proves a concrete correctness/maintainability win.
- This phase owns the final smoke-evidence bar for shell-facing changes.
- The comparison baseline for the `ProxyCommand` decision is the current explicit generated wrapper in `src/hooks/pwsh.rs`, including `ConvertFrom-Json` action handling, native fallback behavior, and PSReadLine integration.

## Status Notes

- 2026-04-19: Phase 4 completed. Landed all five accepted hygiene items (`Resolver.config` visibility tightening, dead `ResolveMode` resolver parameter removal while retaining enum usage in output mode handling, `env_lock()` contract docs, hook marker `expect(...)` diagnostics, and menu-enabled balanced-delimiter tests across Bash/Zsh/Fish/Pwsh).
- 2026-04-19: Recorded explicit `ProxyCommand` rejection in `../verification/proxycommand-eval-phase-4.md` (no net correctness win; unsupported-flag behavior risk), preserving current `src/hooks/pwsh.rs` baseline.
- 2026-04-19: Completed Phase 4 smoke rows 17-21 in `../verification/shell-smoke-matrix.md` (`Pass` for Bash/Zsh/Fish/Pwsh contract checks, conditional row 21 set to `Not Applicable` due to rejection).
- 2026-04-19: Re-ran baseline verify command and full suite; `cargo test` passed with `293 passed`.
- 2026-04-19: Final implementation-review outcome collated in `../reviews/impl-review-phase-4-collated-2026-04-19.md` with unified acceptance.
- 2026-04-17: Implementation-plan prerequisite satisfied via `../reviews/impl-plan-review-phase-4-collated-2026-04-17.md`; user selected immediate Phase 4 kickoff and phase state is now `in_progress`.
