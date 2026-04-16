---
type: planning
entity: phase
plan: "whole-repo-review-remediation"
phase: 3
status: completed
created: "2026-04-16"
updated: "2026-04-17"
---

# Phase 3: Retire Legacy Hook Prototypes

> Part of [whole-repo-review-remediation](../plan.md)

## Objective

Align the test safety net and repository structure with the documented source of truth by moving hook-guard coverage to generated hooks and deleting obsolete legacy hook prototypes.

## Scope

### Includes

- Reworking `tests/shell_hook_guard.rs` (or equivalent coverage) to validate generated-hook behavior instead of `scripts/hooks/*`.
- Deleting legacy hook prototype files from `scripts/hooks/` once replacement coverage is in place.
- Updating docs or plan references if they still mention the legacy prototypes as live artifacts.

### Excludes (deferred to later phases)

- Broad generated-hook refactors beyond what is needed to migrate coverage and delete prototypes.
- PowerShell wrapper redesign.

## Prerequisites

- [x] Phase 2 is completed and verified.
- [x] Replacement generated-hook coverage exists before prototype deletion.
- [x] Replacement coverage now spans the Phase 3 Bash/Zsh guard behaviors (plus Bash `cd` wrapper path) and the deletion approach for both shells is justified through generated-hook tests and recorded shell-smoke evidence.

## Deliverables

- [x] Generated-hook guard coverage for the behaviors previously validated through legacy scripts.
- [x] Deleted legacy prototype hook files (no blocker encountered).
- [x] Smoke-matrix updates showing the authoritative generated-hook path remains viable after prototype retirement.
- [x] Updated references so repo tests consistently treat generated hooks as authoritative.

## Acceptance Criteria

- [x] No active tests source `scripts/hooks/*` as authoritative hook implementations.
- [x] Generated-hook tests cover the Phase 3 guard expectations previously asserted through the legacy Bash path, and Zsh prototype retirement is evidenced by generated-hook coverage and shell-smoke entries.
- [x] Legacy hook prototypes are deleted with automated coverage still passing.
- [x] The smoke matrix records the generated-hook authority transition and shell-specific notes.

## Completion Notes / Evidence

- Exact runtime integration tests passed for generated-hook guard behaviors in `tests/shell_hook_guard.rs`:
  - `bash_generated_hook_command_not_found_guard_prevents_recursive_resolve_calls`
  - `bash_generated_hook_command_not_found_resolves_path_like_command_once`
  - `bash_generated_hook_cd_wrapper_invokes_dx_once_and_changes_directory`
  - `zsh_generated_hook_command_not_found_guard_prevents_recursive_resolve_calls`
  - `zsh_generated_hook_command_not_found_resolves_path_like_command_once`
- Exact CLI generation test passed: `tests/init_cli.rs::init_zsh_with_command_not_found_flag_includes_handler`.
- Exact generated-hook contract marker test passed via corrected invocation: `cargo test --lib hooks::tests::all_shells_freeze_command_not_found_guard_contract_markers -- --exact`.
- Legacy prototypes retired: `scripts/hooks/dx.bash` and `scripts/hooks/dx.zsh` deleted; `scripts/hooks/` is empty.
- Phase 3 shell smoke evidence captured in `../verification/shell-smoke-matrix.md` rows 15-16 and marked `Pass`.
- Final implementation acceptance is recorded in `../reviews/impl-review-phase-3-collated-2026-04-17.md`.
- Reviewer consensus found no blocking implementation issues; remaining notes are low-severity follow-ups only.

## Dependencies on Other Phases

| Phase | Relationship | Notes |
|-------|-------------|-------|
| 2 | blocked-by | Hook cleanup should follow the settled menu/parser contract from Phase 2. |
| 4 | blocks | Finalization/smoke evidence should reflect the post-cleanup hook authority model. |

## Notes

- The user explicitly asked that deletion of the legacy prototypes be considered; this phase treats deletion as the default unless a verified blocker appears.
