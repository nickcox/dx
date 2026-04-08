---
type: planning
entity: plan
plan: "architectural-doc-code-alignment"
status: completed
created: "2026-04-08"
updated: "2026-04-08"
---

# Plan: architectural-doc-code-alignment

## Objective

Bring the project's documented architecture and shell-integration behavior back into alignment by refreshing current-contract documentation, hardening cross-shell menu/hook boundaries, and verifying the resulting behavior across Bash, Zsh, Fish, and PowerShell.

## Motivation

The repository currently has drift between legacy planning docs, current architectural decisions, and some shell-hook runtime behavior. That drift makes it harder to reason about the canonical command surface, menu fallback semantics, PowerShell initialization, and the contract between thin shell wrappers and the Rust binary. This plan fixes the drift case by case, updates the current docs to match intended architecture, and closes the remaining code/documentation gaps.

## Known Starting Conflicts

- `docs/cd-extras-cli-prd.md` still presents obsolete `dx add` / `dx undo` / `dx redo` / generic `dx complete <type> <word>` contracts as if they were current.
- `docs/cd-extras-cli-prd.md` still shows PowerShell init as `Invoke-Expression (& dx init pwsh)` instead of single-script-block evaluation with `Out-String`.
- `docs/shell-hook-guarding.md` says stack-transition wrappers use `dx undo` / `dx redo`, while generated hooks and the CLI use `dx stack undo` / `dx stack redo`.
- Zsh menu noop/error behavior currently differs from Bash, Fish, and PowerShell fallback behavior and must be adjudicated explicitly before implementation.
- Bash, Zsh, and Fish currently consume `dx menu` replace payloads with string/regex slicing, while PowerShell uses structured JSON parsing.

## Requirements

### Functional

- [ ] Adjudicate targeted architecture conflicts case by case using current implementation, current docs, and existing decisions as evidence rather than assuming a single global source of truth.
- [ ] Maintain `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` as the authoritative Phase 1 conflict inventory and decision log for this plan.
- [ ] Refresh `docs/cd-extras-cli-prd.md` so it reflects the current command surface and architecture rather than obsolete `add`/`undo`/`redo`/generic-complete contracts.
- [ ] Refresh shell-hook documentation so stack navigation, menu fallback behavior, PowerShell init guidance, and completion/navigation semantics match the intended current contract.
- [ ] Align menu noop/error handling across supported shells, including Zsh, to the approved fallback semantics.
- [ ] Define the approved shell-to-`dx menu` boundary contract during Phase 1 before code changes begin, and if the payload format changes, land that change atomically across `dx menu`, all affected hook generators, tests, and docs.
- [ ] Harden the shell-to-`dx menu` replacement boundary without introducing new required shell dependencies; per-shell parsing approaches are allowed when they improve safety and portability.
- [ ] Preserve the established thin-wrapper architecture: shell hooks change directories locally, Rust resolves paths/state, selector resolution remains in Rust, and current `dx stack` / `dx navigate` / mode-based `dx complete` contracts remain intact unless explicitly revised by the plan.
- [ ] Add or update tests and smoke procedures covering the changed behavior on Bash, Zsh, Fish, and PowerShell, and record smoke outcomes in `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md`.

### Non-Functional

- [ ] Introduce no new required shell/runtime dependencies such as `jq` or Python helpers.
- [ ] Prefer minimal deltas that preserve existing user-facing behavior outside the targeted architectural corrections.
- [ ] Keep documentation clear about what is current contract versus historical background.
- [ ] Ensure the verification bar includes automated Rust tests plus manual or smoke validation for all four supported shells.
- [ ] Keep the final shell boundary machine-readable, dependency-free, and explicit about escaping/quoting expectations.
- [ ] Maintain graceful behavior when menu mode is disabled, no TTY is available, or the shell cannot apply an interactive replacement.

## Scope

### In Scope

- Maintaining a conflict inventory and decision log in `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md`.
- Refreshing `docs/shell-hook-guarding.md` and `docs/cd-extras-cli-prd.md`, plus `docs/configuration.md` only if a contradiction is discovered during Phase 1.
- Aligning shell-hook generator behavior with the adjudicated menu and navigation contracts.
- Hardening menu action exchange/parsing between `dx menu` and generated shell hooks.
- Adding or adjusting automated tests for hook generation, menu action contracts, and related shell behavior.
- Recording and executing a shell-by-shell smoke matrix in `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` for Bash, Zsh, Fish, and PowerShell init/menu flows.

### Out of Scope

- Building a native frecency store or replacing the zoxide-first strategy.
- Redesigning unrelated CLI areas such as bookmarks, resolver matching, or non-targeted menu UX enhancements.
- Adding new shells or changing the project's overall single-binary architecture.
- OpenSpec workflow changes unrelated to this targeted architecture-alignment effort.

## Definition of Done

- [ ] Every entry in `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` is resolved or explicitly deferred with rationale.
- [ ] `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` records the authoritative contract decisions that Phase 2 implements and Phase 3 verifies.
- [ ] `docs/cd-extras-cli-prd.md` describes the current architecture and command surface instead of stale proposals.
- [ ] `docs/shell-hook-guarding.md` and related docs describe the actual stack/menu/init contracts, including user-facing PowerShell init guidance.
- [ ] Generated hook behavior for Bash, Zsh, Fish, and PowerShell matches the approved menu noop/error fallback semantics.
- [ ] Any menu-boundary format change landed atomically across `dx menu`, affected hook generators, tests, and docs without adding new required dependencies.
- [ ] The menu replacement boundary contract explicitly documents supported escaping/quoting behavior and automated tests prove that behavior for the affected shells.
- [ ] Automated Rust tests covering the changed behavior pass.
- [ ] `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` records completed smoke verification for Bash, Zsh, Fish, and PowerShell.
- [ ] Plan and phase artifacts are updated to reflect what was implemented and verified.

## Testing Strategy

- [ ] Add or update unit/integration tests around hook generation, menu action serialization/consumption, and any changed shell-boundary behavior, including each affected shell parsing path.
- [ ] Run `cargo test` after each implementation phase that changes code.
- [ ] Maintain `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` with required scenarios, status, and evidence notes.
- [ ] Perform manual or smoke verification for Bash, Zsh, Fish, and PowerShell covering init output usage, menu-disabled behavior, successful replacement, cancel-with-query-change, noop/error fallback, and no-TTY or degraded behavior where feasible.
- [ ] Explicitly verify the PowerShell init guidance using single-script-block evaluation (`Out-String`) rather than line-by-line execution, plus the PSReadLine-driven menu path when available.

## Phases

| Phase | Title | Scope | Status |
|-------|-------|-------|--------|
| 1 | Refresh Architecture Docs and Contracts | [Detail](phases/phase-1.md) | completed |
| 2 | Harden Cross-Shell Menu and Hook Boundaries | [Detail](phases/phase-2.md) | completed |
| 3 | Verify Across Shells and Finalize Documentation | [Detail](phases/phase-3.md) | completed |

## Risks & Open Questions

| Risk/Question | Impact | Mitigation/Answer |
|---------------|--------|-------------------|
| Refreshing the PRD may blur historical context if legacy proposals are overwritten without explanation. | Medium | Update the document to current architecture while preserving concise historical framing where useful. |
| The safest menu boundary may require changing the emitted payload format rather than continuing with ad hoc shell-side JSON slicing. | High | Phase 1 must record the approved boundary contract in the conflict inventory; any format change in Phase 2 must be implemented atomically across `dx menu`, affected hooks, tests, and docs, while remaining machine-readable and dependency-free. |
| Aligning Zsh fallback semantics with other shells could reveal UX edge cases in completion behavior. | Medium | Add targeted tests and run shell smokes for cancel/error/noop flows before finalizing. |
| PowerShell menu behavior depends on PSReadLine and terminal capabilities that may vary by environment. | Medium | Preserve guards, keep behavior degradable, and include PowerShell-specific smoke verification in Phase 3. |
| Case-by-case source-of-truth decisions may surface additional minor doc drift while Phase 1 is underway. | Low | Capture those decisions in the refreshed docs and keep the plan changelog/todo current rather than blocking progress. |

## Changelog

### 2026-04-08

- Plan created
- Revised after parallel review: added explicit conflict inventory, boundary-contract constraints, and shell smoke matrix requirements.
- Completed Phase 1 docs/contracts refresh, including conflict adjudications and refreshed shell/PRD docs; Phase 1 verify command passed.
- Authored the Phase 2 implementation plan and moved active work into Phase 2 review/execution preparation.
- Reconciled stale Phase 2 artifacts to completed state (phase + implementation + todo preflight gate) before Phase 3 closeout.
- Completed Phase 3 verification/finalization: recorded automated test outcomes (`cargo test --test menu_cli` 20 pass, full `cargo test` 263 pass, targeted `init_cli`/`key_event_mapping_`/exact menu_cli checks pass) and finalized shell smoke matrix with explicit Pass/Not Feasible evidence.
