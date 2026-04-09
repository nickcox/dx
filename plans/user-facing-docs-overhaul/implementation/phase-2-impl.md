---
type: planning
entity: implementation-plan
plan: "user-facing-docs-overhaul"
phase: 2
status: completed
created: "2026-04-09"
updated: "2026-04-09"
---

# Implementation Plan: Phase 2 - Author Core User-Facing Docs for New Users

> Implements [Phase 2](../phases/phase-2.md) of [user-facing-docs-overhaul](../plan.md)

## Approach

Author a first-pass, newcomer-focused documentation bundle in `docs/` by creating two substantive entry guides (`quickstart.md`, `shell-setup.md`) and three intentionally lightweight expansion skeletons (`command-guide.md`, `troubleshooting.md`, `faq.md`). Establish a minimal but explicit cross-link mesh among these five pages so users can move from first run to next questions without requiring `README.md` integration yet (deferred to Phase 3).

## Affected Modules

| Module | Change Type | Description |
|--------|-------------|-------------|
| `docs/` | create | Add five new user-facing documents for onboarding and follow-up navigation. |
| `plans/user-facing-docs-overhaul/` | create | Add Phase 2 implementation plan artifact grounded in current repo state. |
| `tech-docs/` | reference-only | Use technical docs as source material for accurate user-facing setup language without relocating or rewriting technical internals in this phase. |

## Required Context

| File | Why |
|------|-----|
| `plans/user-facing-docs-overhaul/plan.md` | Confirms overall scope split (`docs/` user-facing, `tech-docs/` technical) and that README work belongs to Phase 3. |
| `plans/user-facing-docs-overhaul/phases/phase-2.md` | Defines exact Phase 2 deliverables: quickstart, shell setup, and three skeleton docs with initial internal linking. |
| `plans/user-facing-docs-overhaul/todo.md` | Provides the active checklist and confirms `implementation/phase-2-impl.md` is the missing planning artifact. |
| `plans/user-facing-docs-overhaul/implementation/phase-1-impl.md` | Carries forward Phase 1 IA baseline and link-strategy constraints (no README introduction yet). |
| `tech-docs/cd-extras-cli-prd.md` | Source of current CLI/shell contract details to keep quickstart and setup guidance behavior-accurate. |
| `tech-docs/configuration.md` | Source of current configuration/env variable defaults relevant to shell setup guidance. |
| `tech-docs/shell-hook-guarding.md` | Source of shell-init and fallback behavior details that should be summarized for users (without deep internals). |
| repo root listing (`/`) | Confirms `README.md` is currently absent; validates deferment to Phase 3 remains consistent with current state. |
| `docs/` directory listing | Confirms `docs/` is currently empty and ready for Phase 2 document creation. |
| `tech-docs/` directory listing | Confirms relocated technical docs are present at root `tech-docs/` as expected from Phase 1. |

## Implementation Steps

### Step 1: Define newcomer information architecture and page contracts

- **What**: Establish explicit role boundaries for each Phase 2 page before drafting content.
- **Where**: `docs/quickstart.md`, `docs/shell-setup.md`, `docs/command-guide.md`, `docs/troubleshooting.md`, `docs/faq.md`.
- **Why**: Prevent overlap and keep each page concise, task-oriented, and easy to expand in later phases.
- **Considerations**: Keep technical implementation depth in `tech-docs/`; user docs should link out rather than duplicate internals.

Proposed page contracts:

- `docs/quickstart.md`: first-run path (install/init basic flow, first successful navigation action, and “what next”).
- `docs/shell-setup.md`: shell-specific setup and verification steps with lightweight troubleshooting pointers.
- `docs/command-guide.md`: skeleton headings for command families and future detailed examples.
- `docs/troubleshooting.md`: skeleton headings grouped by setup/runtime/navigation symptom classes.
- `docs/faq.md`: skeleton headings for common onboarding and behavior questions.

### Step 2: Author core content pages (quickstart + shell setup)

- **What**: Draft practical, command-oriented content for the two primary onboarding pages.
- **Where**: `docs/quickstart.md`, `docs/shell-setup.md`.
- **Why**: Meets Phase 2 acceptance criteria that a new user can reach basic usage from zero context.
- **Considerations**: Use behavior wording aligned with current technical docs (for example shell init patterns and configuration defaults), but avoid deep internals and defer README entry-point framing to Phase 3.

Authoring minimums:

1. Quickstart includes prerequisites, install/init path, first command flow, and short “next docs” section.
2. Shell setup includes Bash/Zsh/Fish/PowerShell setup blocks and at least one verification check path.
3. Both pages use newcomer-first tone and explicit outcome statements (what success looks like).

### Step 3: Create expansion skeleton docs and seed cross-links

- **What**: Create three skeleton docs with starter headings and add initial cross-links among all five Phase 2 pages.
- **Where**: `docs/command-guide.md`, `docs/troubleshooting.md`, `docs/faq.md`, and link sections in `docs/quickstart.md` + `docs/shell-setup.md`.
- **Why**: Provides coherent navigation for immediate follow-up questions while preserving scope for future content expansion.
- **Considerations**: Link strategy should be intentionally minimal and stable; avoid introducing README links or claiming final link completeness in this phase.

Required initial link mesh (relative links):

- `docs/quickstart.md` links to:
  - `./shell-setup.md`
  - `./command-guide.md`
  - `./troubleshooting.md`
  - `./faq.md`
- `docs/shell-setup.md` links to:
  - `./quickstart.md`
  - `./troubleshooting.md`
  - `./faq.md`
- Each skeleton page links back to:
  - `./quickstart.md`
  - `./shell-setup.md`

### Step 4: Run phase-scoped verification for artifacts and links

- **What**: Validate presence of all five docs and presence of required cross-links only for Phase 2 scope.
- **Where**: `docs/` plus repo root for scope guard (`README.md` deferment).
- **Why**: Ensures Phase 2 deliverables are complete and linked without prematurely performing Phase 3 finalization.
- **Considerations**: Verification must not require README existence; README should remain absent or otherwise ignored until Phase 3 execution.

## Testing Plan

Primary verify command (post-implementation):

```bash
bash -lc 'set -euo pipefail; for f in quickstart.md shell-setup.md command-guide.md troubleshooting.md faq.md; do test -f "docs/$f"; done; rg -F "./shell-setup.md" docs/quickstart.md >/dev/null; rg -F "./command-guide.md" docs/quickstart.md >/dev/null; rg -F "./troubleshooting.md" docs/quickstart.md >/dev/null; rg -F "./faq.md" docs/quickstart.md >/dev/null; rg -F "./quickstart.md" docs/shell-setup.md >/dev/null; rg -F "./troubleshooting.md" docs/shell-setup.md >/dev/null; rg -F "./faq.md" docs/shell-setup.md >/dev/null; for f in command-guide.md troubleshooting.md faq.md; do rg -F "./quickstart.md" "docs/$f" >/dev/null; rg -F "./shell-setup.md" "docs/$f" >/dev/null; done; test ! -f README.md'
```

| Test Type | What to Test | Expected Outcome |
|-----------|--------------|------------------|
| Documentation artifact and link verification | All five Phase 2 docs exist; required relative links between quickstart/shell setup/skeleton pages are present; README creation remains deferred. | Command exits 0; any missing file/link or premature README creation fails non-zero. |

### Test Integrity Constraints

- This is a docs-only phase: no Rust source (`src/`), tests (`tests/`), hook templates, or CLI behavior files should be modified.
- Existing automated test suites must not be disabled, removed, or weakened to satisfy documentation progress.
- Verify checks must be phase-scoped: validate only Phase 2 artifacts/links and defer full repository-wide cross-link closure to Phase 3.
- Do not treat README presence as required for Phase 2 success; README authoring/integration remains out of scope until Phase 3.

## Rollback Strategy

If drafted user docs create confusion or link regressions, remove or revert only the five Phase 2 `docs/*.md` files as a batch (or restore prior versions if partially edited), then rerun a reduced baseline check to confirm `docs/` returns to its pre-Phase-2 empty state and `tech-docs/` remains unchanged.

## Open Decisions

| Decision | Options | Chosen | Rationale |
|----------|---------|--------|-----------|
| Skeleton depth in Phase 2 | Full content now vs lightweight scaffolds with headings | Lightweight scaffolds | Matches gated scope and keeps focus on onboarding-critical pages first. |
| README linkage timing | Include README links now vs defer to Phase 3 | Defer to Phase 3 | Preserves phase boundaries from `plan.md` and `phase-2.md`. |

## Reality Check

### Code/Doc Anchors Used

| File | Symbol/Area | Why it matters |
|------|-------------|----------------|
| `plans/user-facing-docs-overhaul/plan.md` | Objective, Scope, DoD, phase table | Confirms audience split and that README/cross-link finalization belongs to Phase 3. |
| `plans/user-facing-docs-overhaul/phases/phase-2.md` | Includes/Excludes, Deliverables, Acceptance Criteria | Defines exact in-scope doc outputs and explicit Phase 3 deferrals. |
| `plans/user-facing-docs-overhaul/todo.md` | Active phase + pending checklist | Confirms pending file list and cross-link task to be implemented in this phase. |
| `plans/user-facing-docs-overhaul/implementation/phase-1-impl.md` | Baseline IA and deferred-link strategy notes | Ensures continuity with completed relocation baseline and sequencing expectations. |
| `tech-docs/cd-extras-cli-prd.md` | CLI/shell contract sections | Grounds user guidance in current implemented command behavior. |
| `tech-docs/configuration.md` | Precedence, env vars, command-level overrides | Grounds shell setup/config references in current technical source-of-truth. |
| `tech-docs/shell-hook-guarding.md` | Init/menu/fallback and wrapper contracts | Provides implementation-accurate shell setup and troubleshooting context. |
| repo root listing (`/`) | top-level file inventory | Confirms `README.md` absent and presence of `docs/`, `plans/`, and `tech-docs/`. |
| `docs/` directory listing | current content state | Confirms `docs/` is empty before Phase 2 authoring. |
| `tech-docs/` directory listing | relocated technical docs | Confirms technical set exists in expected location after Phase 1. |

### Mismatches / Notes

- `todo.md` currently labels the Phase 2 implementation artifact as “Not created yet”; this file resolves that planning gap.
- Root `README.md` is absent in current repo state; this matches the gated Phase 2 exclusion and should remain deferred.
- `tech-docs/shell-hook-guarding.md` references `docs/configuration.md` (line 5) even though configuration docs now live under `tech-docs/`; this pre-existing doc-link inconsistency should be handled in a later consistency pass (Phase 3 or separate fix), not by expanding Phase 2 scope.
