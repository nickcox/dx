---
type: planning
entity: phase
plan: "user-facing-docs-overhaul"
phase: 1
status: completed
created: "2026-04-09"
updated: "2026-04-09"
---

# Phase 1: IA Restructure and Technical Doc Relocation Baseline

> Part of [user-facing-docs-overhaul](../plan.md)

## Objective

Define and apply the documentation information architecture split so `docs/` is user-facing and technical content is relocated to root `tech-docs/`, with a clear link strategy for the next phases.

## Scope

### Includes

- Confirming audience split policy: `docs/` for users, `tech-docs/` for technical/development references.
- Creating root `tech-docs/` and relocating current technical docs from `docs/`:
  - `cd-extras-cli-prd.md`
  - `configuration.md`
  - `shell-hook-guarding.md`
- Defining baseline navigation/link conventions among `README.md`, `docs/`, and `tech-docs/`.
- Capturing relocation mapping for Phase 2 and Phase 3 execution.

### Excludes (deferred to later phases)

- Writing full new-user content (handled in Phase 2).
- Creating/finalizing root `README.md` (handled in Phase 3).
- Broad technical content rewrites beyond relocation-level updates.

## Prerequisites

- [ ] Top-level plan is active and approved for execution.
- [ ] Current technical docs inventory is confirmed.

## Deliverables

- [ ] `tech-docs/` directory with relocated technical docs.
- [ ] Updated baseline notes describing IA split and link strategy.
- [ ] Confirmed file mapping from old `docs/` paths to new `tech-docs/` paths.

## Acceptance Criteria

- [ ] `docs/` no longer hosts technical/development-only documents from the initial set.
- [ ] `tech-docs/` contains all three relocated baseline technical docs.
- [ ] Link strategy is documented clearly enough for authoring in Phase 2 and README wiring in Phase 3.

## Dependencies on Other Phases

| Phase | Relationship | Notes |
|-------|-------------|-------|
| None | blocked-by | This is the execution baseline phase. |

## Notes

Keep relocation conservative: preserve filenames where practical, then update links centrally during Phase 3 consistency pass.
