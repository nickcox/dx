---
type: planning
entity: implementation-plan
plan: "user-facing-docs-overhaul"
phase: 3
status: completed
created: "2026-04-09"
updated: "2026-04-09"
---

# Implementation Plan: Phase 3 - README, Cross-Linking, and Final Verification

> Implements [Phase 3](../phases/phase-3.md) of [user-facing-docs-overhaul](../plan.md)

## Approach

Complete the documentation IA by adding a minimal, navigational root `README.md` that routes new users to `docs/quickstart.md` and `docs/shell-setup.md`, then close the loop with reciprocal links from user docs back to README and pointers to technical references in `tech-docs/`. Finish with a single phase-level verification pass that checks file presence and key link targets textually, including correction of the known stale configuration reference in `tech-docs/shell-hook-guarding.md`.

## Affected Modules

| Module | Change Type | Description |
|--------|-------------|-------------|
| repo root (`README.md`) | create | Add top-level user entry point with project overview and doc navigation links. |
| `docs/` | modify | Add/verify reciprocal links between user onboarding pages and README; keep newcomer-first flow intact. |
| `tech-docs/` | modify | Fix stale `docs/configuration.md` reference and verify technical-doc pointers remain accurate. |
| `plans/user-facing-docs-overhaul/` | reference-only in this phase execution plan | Use plan/phase/todo artifacts as gating context; update closure status during execution step after verification passes. |

## Required Context

| File | Why |
|------|-----|
| `plans/user-facing-docs-overhaul/plan.md` | Confirms Phase 3 scope includes README creation, cross-link strategy completion, and final verification (`plan.md:42-47`, `plan.md:56-61`). |
| `plans/user-facing-docs-overhaul/phases/phase-3.md` | Defines exact Phase 3 deliverables and acceptance criteria for README and link consistency (`phase-3.md:23-30`, `phase-3.md:43-53`). |
| `plans/user-facing-docs-overhaul/todo.md` | Provides active checklist items and sequencing, including closure-update task that happens after successful verification (`todo.md:24-30`). |
| `plans/user-facing-docs-overhaul/implementation/phase-2-impl.md` | Carries forward verified Phase 2 baseline and known mismatch note about stale configuration link to fix now (`phase-2-impl.md:150-152`). |
| `docs/quickstart.md` | Existing onboarding page that currently links to shell setup and skeleton pages but not README/tech-doc entry points (`quickstart.md:22`, `quickstart.md:40-45`). |
| `docs/shell-setup.md` | Existing setup page with related-links section to update/verify for reciprocal navigation (`shell-setup.md:36-40`). |
| `docs/command-guide.md` | Skeleton doc currently linking to quickstart/shell setup; verify whether README backlink is needed for consistency (`command-guide.md:5-9`). |
| `docs/troubleshooting.md` | Skeleton doc currently linking to quickstart/shell setup; verify consistent user-nav mesh (`troubleshooting.md:5-9`). |
| `docs/faq.md` | Skeleton doc currently linking to quickstart/shell setup; verify consistent user-nav mesh (`faq.md:5-9`). |
| `tech-docs/cd-extras-cli-prd.md` | Technical reference target to optionally list from README/user docs when pointing advanced readers to internals (`cd-extras-cli-prd.md:1-6`, `cd-extras-cli-prd.md:100-104`). |
| `tech-docs/configuration.md` | Correct configuration reference destination that should replace stale path mention in shell-hook doc (`configuration.md:1-4`). |
| `tech-docs/shell-hook-guarding.md` | Contains known stale path reference to `docs/configuration.md` to be fixed in this phase (`shell-hook-guarding.md:5`). |
| repo root listing (`/`) | Confirms `README.md` is currently absent and must be created as a Phase 3 deliverable. |

## Implementation Steps

### Step 1: Create root README as the documentation entry point

- **What**: Author `README.md` with concise project overview, newcomer start path, and explicit links to `docs/quickstart.md` and `docs/shell-setup.md`; optionally list command guide, troubleshooting, and FAQ links.
- **Where**: `README.md` (repo root).
- **Why**: Phase 3 requires a top-level onboarding path and clear routing into user-facing docs.
- **Considerations**: Keep README lightweight and navigational (no deep duplication of detailed content from `docs/` or technical internals from `tech-docs/`).

### Step 2: Add/verify reciprocal user-doc links to README and technical references

- **What**: Update relevant pages in `docs/` to include a clear path back to `README.md` and appropriate pointer(s) to `tech-docs/` for advanced implementation details.
- **Where**: At minimum `docs/quickstart.md` and `docs/shell-setup.md`; evaluate skeleton pages (`docs/command-guide.md`, `docs/troubleshooting.md`, `docs/faq.md`) for consistent navigation treatment.
- **Why**: Completes cross-link mesh so users can move between entry point, task docs, and advanced references without dead ends.
- **Considerations**: Use stable relative links from docs pages (`../README.md`, `../tech-docs/...`) and keep audience boundary explicit (user guidance in `docs/`, implementation details in `tech-docs/`).

### Step 3: Fix known stale technical-doc link introduced by relocation

- **What**: Replace stale `docs/configuration.md` mention in `tech-docs/shell-hook-guarding.md` with the current configuration-doc path.
- **Where**: `tech-docs/shell-hook-guarding.md`.
- **Why**: Current reference is outdated after Phase 1 relocation and would fail consistency/link checks if left unchanged.
- **Considerations**: Prefer local relative path (`./configuration.md`) inside `tech-docs/` for resilience; avoid introducing new absolute URLs.

### Step 4: Run final textual consistency/link verification for Phase 3 outcomes

- **What**: Execute one phase verify command that asserts: (a) `README.md` exists, (b) required README links to quickstart/shell setup exist, (c) reciprocal README backlinks exist in key user docs, and (d) stale configuration link string is removed and corrected target is present.
- **Where**: repo root; checks across `README.md`, `docs/*.md`, and `tech-docs/shell-hook-guarding.md`.
- **Why**: Provides objective acceptance evidence for Phase 3 before artifact-closure updates.
- **Considerations**: This is text-level verification for docs IA; do not claim broader markdown link crawler coverage unless separately run.

### Step 5: Close planning artifacts after verification passes

- **What**: Update plan/todo/phase status artifacts to completed state only after Step 4 succeeds.
- **Where**: `plans/user-facing-docs-overhaul/plan.md`, `plans/user-facing-docs-overhaul/phases/phase-3.md`, `plans/user-facing-docs-overhaul/todo.md`.
- **Why**: Prevents premature closure and keeps lifecycle state aligned with verified outcomes.
- **Considerations**: This step is explicitly deferred to execution; this implementation-plan artifact remains `active` until execution is completed.

## Testing Plan

Primary verify command (post-implementation):

```bash
bash -lc 'set -euo pipefail; test -f README.md; rg -E "(\./)?docs/quickstart\.md" README.md >/dev/null; rg -E "(\./)?docs/shell-setup\.md" README.md >/dev/null; rg -E "\.\./README\.md|README\.md" docs/quickstart.md >/dev/null; rg -E "\.\./README\.md|README\.md" docs/shell-setup.md >/dev/null; rg -F "docs/configuration.md" tech-docs/shell-hook-guarding.md >/dev/null && exit 1 || true; rg -E "\./configuration\.md|tech-docs/configuration\.md" tech-docs/shell-hook-guarding.md >/dev/null'
```

| Test Type | What to Test | Expected Outcome |
|-----------|--------------|------------------|
| Documentation IA/link verification | README exists; README links to quickstart/shell setup; quickstart and shell setup include README backlink; stale `docs/configuration.md` reference removed and corrected config-doc target present in shell-hook technical doc. | Command exits 0 only when all Phase 3 required textual link conditions are satisfied. |

### Test Integrity Constraints

- Docs-only phase: no changes to Rust source (`src/`), automated tests (`tests/`), benchmark code, or shell hook implementation logic are permitted for acceptance.
- Existing test coverage must not be disabled, deleted, or weakened to claim documentation completion.
- Verify command must fail on stale-link regression (`docs/configuration.md` appearing in `tech-docs/shell-hook-guarding.md`).
- Plan/todo closure updates are valid only after the verify command passes in the same working tree state.

## Rollback Strategy

If Phase 3 edits introduce navigation confusion or incorrect cross-links, revert README and affected docs/tech-doc link edits together, restoring the pre-Phase-3 baseline, then re-run the verify command to confirm failure state is understood before reapplying corrected link updates.

## Open Decisions

| Decision | Options | Chosen | Rationale |
|----------|---------|--------|-----------|
| README link breadth | Required minimum only (quickstart + shell setup) vs include skeleton docs too | Include required minimum; optionally include skeleton docs if concise | Meets acceptance criteria with low risk while allowing navigational completeness. |
| Technical-doc pointer placement in user docs | Add only in README vs add in README + key user pages | README + key user pages (`quickstart`, `shell-setup`) | Better bidirectional navigation and clearer audience boundary without heavy content rewrite. |

## Reality Check

### Code/Doc Anchors Used

| File | Symbol/Area | Why it matters |
|------|-------------|----------------|
| `plans/user-facing-docs-overhaul/plan.md` | Objective/Scope/DoD and phase table (`plan.md:14-15`, `plan.md:42-47`, `plan.md:56-61`, `plan.md:74-77`) | Confirms README + cross-link verification is Phase 3 scope and completion gating. |
| `plans/user-facing-docs-overhaul/phases/phase-3.md` | Includes/Deliverables/Acceptance Criteria (`phase-3.md:23-30`, `phase-3.md:43-53`) | Defines required outcomes and closure expectation. |
| `plans/user-facing-docs-overhaul/todo.md` | Active phase context and pending checklist (`todo.md:12-30`) | Confirms pending execution tasks and missing Phase 3 impl artifact now being authored. |
| `plans/user-facing-docs-overhaul/implementation/phase-2-impl.md` | Mismatch note about stale configuration link (`phase-2-impl.md:150-152`) | Carries forward known consistency defect into explicit Phase 3 fix step. |
| `docs/quickstart.md` | Existing links section (`quickstart.md:22`, `quickstart.md:40-45`) | Shows current user-nav mesh and where README backlink can be added. |
| `docs/shell-setup.md` | Related docs section (`shell-setup.md:36-40`) | Identifies reciprocal-link insertion point. |
| `docs/command-guide.md` | Quick Links (`command-guide.md:5-9`) | Confirms skeleton navigation baseline for optional README consistency updates. |
| `docs/troubleshooting.md` | Quick Links (`troubleshooting.md:5-9`) | Confirms skeleton navigation baseline for optional README consistency updates. |
| `docs/faq.md` | Quick Links (`faq.md:5-9`) | Confirms skeleton navigation baseline for optional README consistency updates. |
| `tech-docs/shell-hook-guarding.md` | Stale configuration reference (`shell-hook-guarding.md:5`) | Concrete mismatch to fix in this phase. |
| `tech-docs/configuration.md` | Canonical configuration doc location (`configuration.md:1-4`) | Correct replacement target for stale link. |
| repo root listing (`/Users/nick/code/personal/dx`) | Top-level entries, README absence | Verifies current mismatch with Phase 3 deliverable requirement (README currently absent). |

### Mismatches / Notes

- Root `README.md` absence was resolved during execution by creating a concise top-level entry point with links to user docs and technical references.
- Stale `docs/configuration.md` reference in `tech-docs/shell-hook-guarding.md` was corrected to `./configuration.md`.
- Phase closure updates across `phase-3.md`, `plan.md`, and `todo.md` were completed after verify checks passed.

## Execution Outcome

- Status: Completed (2026-04-09)
- Verify command intent passed for all required checks (README existence, required doc links, reciprocal README backlinks in quickstart/shell setup, stale-link removal + corrected tech-doc path).
