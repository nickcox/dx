---
type: planning
entity: phase
plan: "architectural-doc-code-alignment"
phase: 1
status: completed
created: "2026-04-08"
updated: "2026-04-08"
---

# Phase 1: Refresh Architecture Docs and Contracts

> Part of [architectural-doc-code-alignment](../plan.md)

## Objective

Resolve the targeted documentation and architecture-contract drift so the project has a clear current baseline for implementation work.

## Scope

### Includes

- Creating and maintaining `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` as the authoritative inventory and decision log for adjudicated conflicts.
- Auditing the current known conflicts: obsolete PRD command surface, stale PowerShell init instructions, stack-wrapper command wording drift, Zsh-vs-other-shell menu fallback divergence, and menu payload parsing asymmetry.
- Refreshing `docs/shell-hook-guarding.md` to match current stack/navigation/menu semantics.
- Refreshing `docs/cd-extras-cli-prd.md` to the current architecture and command surface.
- Updating `docs/configuration.md` only if the phase uncovers a real contradiction there.
- Recording the case-by-case decisions that Phase 2 should treat as the approved contract.

### Excludes (deferred to later phases)

- Runtime changes to hook generation or menu behavior.
- New features or unrelated CLI/documentation cleanup.
- End-to-end shell smoke verification beyond quick sanity checks needed while editing docs.

## Prerequisites

- [ ] The top-level plan has been reviewed and accepted for execution.
- [ ] The targeted conflict areas remain limited to the agreed doc/code alignment scope.

## Deliverables

- [ ] `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` updated with the initial inventory, adjudications, and any explicit deferrals.
- [ ] Updated docs that describe the current shell-hook and navigation architecture.
- [ ] Refreshed PRD content aligned with the current command surface and architecture.
- [ ] A documented contract baseline that the Phase 2 implementation plan can reference.

## Acceptance Criteria

- [ ] Every inventory item in `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` is resolved in docs or explicitly deferred with rationale.
- [ ] The conflict inventory records the approved menu fallback behavior and the approved shell-to-`dx menu` boundary contract that Phase 2 must implement.
- [ ] Phase 2 can cite the Phase 1 docs and conflict inventory as the current source set for implementation planning.
- [ ] Any intentionally retained historical context in the PRD is clearly distinguishable from current requirements.

## Dependencies on Other Phases

| Phase | Relationship | Notes |
|-------|-------------|-------|
| None | blocked-by | Can begin once the plan review gate is cleared. |

## Notes

Primary references for this phase are expected to include `docs/shell-hook-guarding.md`, `docs/configuration.md`, `docs/cd-extras-cli-prd.md`, `src/hooks/*.rs`, `src/hooks/mod.rs`, `src/cli/complete.rs`, `src/cli/stacks.rs`, `src/cli/menu.rs`, and `src/menu/action.rs`.
