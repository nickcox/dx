---
type: planning
entity: implementation-plan
plan: "user-facing-docs-overhaul"
phase: 1
status: completed
created: "2026-04-09"
updated: "2026-04-09"
---

# Implementation Plan: Phase 1 - IA Restructure and Technical Doc Relocation Baseline

> Implements [Phase 1](../phases/phase-1.md) of [user-facing-docs-overhaul](../plan.md)

## Approach

Execute a documentation-only IA baseline change: create root `tech-docs/`, relocate the three existing technical documents from `docs/` to `tech-docs/` with filenames preserved, and capture the old→new mapping plus baseline link strategy notes that Phase 2 and Phase 3 will consume. Keep this phase strictly scoped away from new user-doc authoring and `README.md` creation.

## Affected Modules

| Module | Change Type | Description |
|--------|-------------|-------------|
| `docs/` | modify | Remove the initial technical-doc set from user-doc location after relocation. |
| `tech-docs/` | create | Introduce technical-doc home and place relocated files with original names. |
| `plans/user-facing-docs-overhaul/` | modify | Record relocation mapping and link-strategy baseline for downstream phases. |

## Required Context

| File | Why |
|------|-----|
| `plans/user-facing-docs-overhaul/plan.md` | Confirms lightweight lifecycle, phase boundaries, and no code/runtime-change intent. |
| `plans/user-facing-docs-overhaul/phases/phase-1.md` | Defines Phase 1 objective, deliverables, and acceptance criteria. |
| `plans/user-facing-docs-overhaul/todo.md` | Tracks current Phase 1 relocation checklist and active-phase context. |
| `docs/cd-extras-cli-prd.md` | Baseline technical doc to relocate unchanged (filename preserved). |
| `docs/configuration.md` | Baseline technical doc to relocate unchanged (filename preserved). |
| `docs/shell-hook-guarding.md` | Baseline technical doc to relocate unchanged (filename preserved). |
| `/` repo root listing | Confirms current baseline: no root `README.md` and no `tech-docs/` yet. |

## Implementation Steps

### Step 1: Establish relocation map and IA baseline notes

- **What**: Define and lock the Phase 1 old→new path mapping and the baseline cross-link strategy notes for later phases.
- **Where**: `plans/user-facing-docs-overhaul/` artifacts (implementation/todo execution notes).
- **Why**: Prevent drift between relocation execution and later README/user-doc wiring.
- **Considerations**: Keep strategy notes minimal and forward-looking only; do not introduce Phase 2/3 authoring work in this phase.

Relocation mapping to apply:

| Old Path | New Path |
|----------|----------|
| `docs/cd-extras-cli-prd.md` | `tech-docs/cd-extras-cli-prd.md` |
| `docs/configuration.md` | `tech-docs/configuration.md` |
| `docs/shell-hook-guarding.md` | `tech-docs/shell-hook-guarding.md` |

Baseline link strategy notes (for later phases):

1. Phase 2 user docs should avoid deep technical duplication and instead link to `tech-docs/*` where appropriate.
2. Phase 3 README should provide top-level split navigation: user onboarding links into `docs/` and technical references into `tech-docs/`.
3. During this phase, prioritize path stability (preserved filenames) over content rewrites.

### Step 2: Create technical-doc root and relocate files conservatively

- **What**: Create root `tech-docs/` and move the three target documents from `docs/` to `tech-docs/` while preserving filenames.
- **Where**: Filesystem paths `tech-docs/` and `docs/`.
- **Why**: Implements the IA audience split baseline with lowest migration risk.
- **Considerations**: Use moves (not rewrites) to preserve content/history continuity; do not alter runtime code or command behavior.

### Step 3: Verify post-move baseline state

- **What**: Confirm all three files exist under `tech-docs/`, are absent from `docs/`, and baseline scope remains doc-only.
- **Where**: Repo root + `docs/` + `tech-docs/`.
- **Why**: Satisfies Phase 1 acceptance criteria and ensures readiness for Phase 2/3.
- **Considerations**: `README.md` creation remains out of scope in this phase and should not be introduced as part of relocation execution.

## Testing Plan

Primary verify command (post-implementation):

```bash
bash -lc 'set -euo pipefail; test -d tech-docs; for f in cd-extras-cli-prd.md configuration.md shell-hook-guarding.md; do test -f "tech-docs/$f"; test ! -f "docs/$f"; done; test ! -f README.md'
```

| Test Type | What to Test | Expected Outcome |
|-----------|--------------|------------------|
| Filesystem baseline verification | IA baseline after relocation: `tech-docs/` exists, all three technical docs moved with same filenames, originals removed from `docs/`, and `README.md` still absent in Phase 1 scope. | Command exits 0 with no output; any mismatch fails non-zero. |

### Test Integrity Constraints

- This phase is documentation-only; no Rust source, shell hook templates, or runtime behavior files should be modified.
- Existing test suites are not to be disabled/edited to satisfy this phase; behavior remains unchanged.
- Verification focuses on path/state correctness (relocation + scope boundaries), not content rewrites.

## Rollback Strategy

If relocation introduces issues, move each file back to its original `docs/` path, remove `tech-docs/` only if empty, and re-run the same verify command adapted for pre-move expectations to restore the prior baseline.

## Open Decisions

| Decision | Options | Chosen | Rationale |
|----------|---------|--------|-----------|
| Filename policy during relocation | Preserve filenames vs rename to new taxonomy | Preserve filenames | Minimizes migration risk and keeps links/mapping deterministic for later-phase rewiring. |
| Link updates timing | Update all links now vs defer to consistency pass | Defer full link rewiring to Phase 3 | Matches gated scope and keeps Phase 1 as IA baseline only. |

## Reality Check

### Code/Doc Anchors Used

| File | Symbol/Area | Why it matters |
|------|-------------|----------------|
| `plans/user-facing-docs-overhaul/plan.md` | Scope + DoD + phase table | Confirms phase sequencing and non-code focus. |
| `plans/user-facing-docs-overhaul/phases/phase-1.md` | Includes/Excludes + acceptance criteria | Defines exact relocation baseline and out-of-scope limits. |
| `plans/user-facing-docs-overhaul/todo.md` | Active phase + pending checklist | Shows implementation file was missing and tasks are relocation-centric. |
| `docs/cd-extras-cli-prd.md` | Current technical PRD content | Confirms this doc belongs to technical set targeted for move. |
| `docs/configuration.md` | Current technical config reference | Confirms this doc belongs to technical set targeted for move. |
| `docs/shell-hook-guarding.md` | Current technical shell-integration reference | Confirms this doc belongs to technical set targeted for move. |
| repo root directory listing | Top-level baseline | Confirms `README.md` absent and `tech-docs/` not yet created. |

### Mismatches / Notes

- Repo currently has no root `README.md`; this matches Phase 1 exclusions and should remain unchanged in this phase.
- `tech-docs/` does not exist yet; creation is a required Phase 1 deliverable.
- The three baseline technical docs are still under `docs/` at planning time; relocation is pending and explicitly tracked in `todo.md`.
