---
type: planning
entity: phase
plan: "user-facing-docs-overhaul"
phase: 2
status: completed
created: "2026-04-09"
updated: "2026-04-09"
---

# Phase 2: Author Core User-Facing Docs for New Users

> Part of [user-facing-docs-overhaul](../plan.md)

## Objective

Create the first user-facing documentation bundle in `docs/` for new users, focused on a clear quickstart and shell setup path, plus foundational skeletons for common follow-up needs.

## Scope

### Includes

- Creating/authoring user-facing docs in `docs/` with new-user-first language and task flow.
- Core content:
  - Quickstart guide
  - Shell setup guide
  - Command guide skeleton
  - Troubleshooting skeleton
  - FAQ skeleton
- Applying Phase 1 IA and link strategy to all new docs.

### Excludes (deferred to later phases)

- Final README entry-point integration and full cross-link verification (Phase 3).
- Deep technical internals that belong in `tech-docs/`.

## Prerequisites

- [x] Phase 1 relocation baseline and IA decisions complete.
- [x] `docs/` is prepared for user-facing structure.

## Deliverables

- [x] User-facing quickstart doc in `docs/`.
- [x] User-facing shell setup doc in `docs/`.
- [x] Initial skeleton docs for commands, troubleshooting, and FAQ in `docs/`.
- [x] Internal links between bundle pages for a coherent newcomer path.

## Acceptance Criteria

- [x] New users can go from zero context to basic usage via quickstart + shell setup docs.
- [x] Skeleton pages exist for common next questions (commands/troubleshooting/FAQ).
- [x] Content tone and structure are user-facing and avoid unnecessary implementation detail.

## Dependencies on Other Phases

| Phase | Relationship | Notes |
|-------|-------------|-------|
| 1 | blocked-by | Requires IA split and technical doc relocation baseline. |

## Notes

Bias toward practical examples and concrete command paths. Defer edge-case depth to troubleshooting/FAQ expansions in later iterations.
