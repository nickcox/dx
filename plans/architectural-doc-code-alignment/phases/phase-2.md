---
type: planning
entity: phase
plan: "architectural-doc-code-alignment"
phase: 2
status: completed
created: "2026-04-08"
updated: "2026-04-08"
---

# Phase 2: Harden Cross-Shell Menu and Hook Boundaries

> Part of [architectural-doc-code-alignment](../plan.md)

## Objective

Bring the generated shell hooks and `dx menu` boundary into line with the approved contract from Phase 1, with special focus on consistent fallback behavior and safer replacement parsing.

## Scope

### Includes

- Aligning cross-shell menu noop/error handling, including Zsh, to the approved fallback semantics.
- Implementing the boundary contract recorded in `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` without adding new required dependencies.
- Hardening the machine-readable payload exchanged between `dx menu` and shell hooks without adding new required dependencies.
- Updating shell-specific parsing logic where needed, allowing different implementations per shell when that improves safety.
- Adding or updating automated tests for hook generation, parsing behavior, and the approved shell/menu contract.
- Updating docs if implementation details require clarifying the final boundary contract.

### Excludes (deferred to later phases)

- Additional menu UX enhancements unrelated to the fallback/parsing architecture.
- Broad refactors of unrelated shell wrappers or command families.
- Final cross-shell smoke reporting and plan closeout, which belong to Phase 3.

## Prerequisites

- [x] Phase 1 is completed and its refreshed docs are accepted as the implementation baseline.
- [x] A reviewed implementation plan exists for this phase.

## Deliverables

- [x] Code changes in the relevant hook/menu modules implementing the approved contract.
- [x] Automated tests covering the changed fallback and parsing behavior.
- [x] Any supporting documentation updates required by the final implementation choices.

## Acceptance Criteria

- [x] Generated hooks for Bash, Zsh, Fish, and PowerShell apply the approved noop/error/replace contract consistently.
- [x] Any payload or parsing change is implemented atomically across `dx menu`, affected hook generators, tests, and docs.
- [x] Automated tests cover quoting-sensitive replacement, cancel-with-query-change, noop/error handling, and each affected shell parsing path.
- [x] The shell boundary handles supported quoting/escaping safely under the approved contract.
- [x] `cargo test` passes after the phase is complete.

## Dependencies on Other Phases

| Phase | Relationship | Notes |
|-------|-------------|-------|
| 1 | blocked-by | Uses the refreshed docs/contracts from Phase 1 as the source set for code changes. |

## Notes

Likely touch points include `src/hooks/bash.rs`, `src/hooks/zsh.rs`, `src/hooks/fish.rs`, `src/hooks/pwsh.rs`, `src/hooks/mod.rs`, `src/menu/action.rs`, and `src/cli/menu.rs`. The per-phase implementation plan may split this phase into smaller work packages, but all work packages must implement the same Phase 1 contract baseline.

Phase 2 closure evidence carried into Phase 3 preflight/verification:

- `cargo test --test menu_cli` => 20 passed (1 suite)
- `cargo test --test init_cli` => 7 passed
- `cargo test key_event_mapping_` => 4 passed
- `cargo test --test menu_cli hook_scripts_contain_fallback_on_noop -- --exact` => 1 passed
- `cargo test --test menu_cli hook_scripts_apply_replace_action_contract -- --exact` => 1 passed
