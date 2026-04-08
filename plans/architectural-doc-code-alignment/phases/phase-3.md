---
type: planning
entity: phase
plan: "architectural-doc-code-alignment"
phase: 3
status: completed
created: "2026-04-08"
updated: "2026-04-08"
---

# Phase 3: Verify Across Shells and Finalize Documentation

> Part of [architectural-doc-code-alignment](../plan.md)

## Objective

Verify the refreshed architecture end to end across the supported shells, make any final documentation adjustments from real verification results, and close out the plan state cleanly.

## Scope

### Includes

- Running the agreed automated Rust test suite after code changes are complete.
- Recording and performing smoke verification for Bash, Zsh, Fish, and PowerShell via `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md`.
- Validating init usage and fallback behavior in real shell contexts, including the PowerShell single-script-block invocation guidance.
- Making final documentation adjustments that are directly driven by the verification results.
- Updating plan artifacts to reflect completion status, verification outcomes, and residual follow-up items if any remain.

### Excludes (deferred to later phases)

- New functional changes discovered during smoke testing unless they are required to fix a blocker introduced by this plan.
- Broader documentation generation outside the touched architecture surfaces.

## Prerequisites

- [x] Phase 2 code changes are complete and stable.
- [x] A reviewed implementation plan exists for this phase if verification steps require additional orchestration.

## Deliverables

- [x] Recorded automated test results.
- [x] `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` completed with per-shell status and evidence notes.
- [x] Final documentation cleanup based on verified behavior.
- [x] Updated plan/todo state showing completed work and any explicit residual follow-ups.

## Acceptance Criteria

- [x] `cargo test` passes.
- [x] `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` captures Bash, Zsh, Fish, and PowerShell scenarios with expected outcomes and status.
- [x] The smoke matrix covers init usage, menu-disabled behavior, successful replacement, cancel-with-query-change, noop/error fallback, and no-TTY or degraded behavior for each shell where feasible.
- [x] The PowerShell smoke matrix includes single-script-block init (`Out-String`) and PSReadLine-driven menu behavior, with any untestable degraded path explicitly noted.
- [x] Final docs describe verified behavior rather than inferred behavior.
- [x] The plan artifacts reflect the completed or intentionally deferred outcomes.

## Dependencies on Other Phases

| Phase | Relationship | Notes |
|-------|-------------|-------|
| 2 | blocked-by | Requires the implementation and automated test coverage from Phase 2. |

## Notes

This phase should record command snippets or concise evidence notes for each matrix row so the verification work is reproducible rather than implied.

Recorded automated verification evidence:

- `cargo test --test menu_cli` => 20 passed (1 suite)
- `cargo test` => 263 passed (13 suites)
- `cargo test --test init_cli` => 7 passed
- `cargo test key_event_mapping_` => 4 passed
- `cargo test --test menu_cli hook_scripts_contain_fallback_on_noop -- --exact` => 1 passed
- `cargo test --test menu_cli hook_scripts_apply_replace_action_contract -- --exact` => 1 passed

Not Feasible allowances captured in the smoke matrix:

- Fish scenarios: shell binary unavailable (`command -v fish` => missing).
- Zsh interactive widget scenarios requiring active ZLE: non-interactive invocation emits `widgets can only be called when ZLE is active`.
- PowerShell interactive PSReadLine replace/cancel/noop-fallback scenarios: not reproducible in this non-interactive harness.
