---
type: planning
entity: plan
plan: "user-facing-docs-overhaul"
status: completed
created: "2026-04-09"
updated: "2026-04-09"
---

# Plan: user-facing-docs-overhaul

## Objective

Restructure project documentation so `docs/` becomes a user-facing documentation hub for new users, while technical/development materials move to root `tech-docs/`, and publish a new root `README.md` that introduces the project and points users to the right getting-started guides.

## Motivation

Current documentation is technical-first and not friendly for onboarding. New users need a clear starting path (quickstart + shell setup) and predictable navigation. Separating user docs from technical docs reduces confusion and makes future documentation growth easier to manage.

## Requirements

### Functional

- [x] Treat `docs/` as user-facing docs only.
- [x] Relocate existing technical docs from `docs/` to root `tech-docs/`.
- [x] Create a new root `README.md` with high-level overview and links to user docs in `docs/`.
- [x] Deliver first user bundle for new users with emphasis on quickstart and shell setup.
- [x] Add core user-doc skeletons needed for expansion (command guide, troubleshooting, FAQ).
- [x] Define and apply a consistent cross-link strategy between README, user docs, and tech docs.

### Non-Functional

- [x] Keep the lifecycle lightweight: plan → implementation plan → execute, with no mandatory review trios.
- [x] Prefer concise, task-oriented writing for first-time users.
- [x] Keep link structure stable and easy to extend.
- [x] Avoid unnecessary code changes; focus on documentation IA and content quality.

## Scope

### In Scope

- Information architecture restructuring for docs location and audience split.
- Moving current technical docs (`cd-extras-cli-prd.md`, `configuration.md`, `shell-hook-guarding.md`) from `docs/` to `tech-docs/`.
- Creating first-pass user docs in `docs/` centered on quickstart and shell setup.
- Creating `README.md` with overview and clear doc entry points.
- Final consistency, cross-link, and verification pass.

### Out of Scope

- Large rewrites of technical doc content beyond relocation and link updates.
- New CLI feature work unrelated to documentation.
- Full documentation program for all audiences beyond the initial new-user bundle.

## Definition of Done

- [x] `docs/` contains user-facing docs only.
- [x] Root `tech-docs/` exists and contains relocated technical/development docs.
- [x] Root `README.md` exists with high-level overview and links to user-facing docs.
- [x] New-user core docs exist in `docs/`: quickstart, shell setup, and skeletons for command guide, troubleshooting, and FAQ.
- [x] Internal links resolve correctly across README, `docs/`, and `tech-docs/`.
- [x] Plan, phases, and todo artifacts are updated to reflect completion state.

## Testing Strategy

- [x] Verify expected files/folders exist in `docs/` and `tech-docs/`.
- [x] Perform link walkthrough from `README.md` to quickstart and shell setup.
- [x] Check user-doc pages for audience fit (new-user-first language and flow).
- [x] Run a final manual consistency pass for naming, headings, and cross-links.

## Phases

| Phase | Title | Scope | Status |
|-------|-------|-------|--------|
| 1 | IA Restructure and Technical Doc Relocation Baseline | [Detail](phases/phase-1.md) | completed |
| 2 | Author Core User-Facing Docs for New Users | [Detail](phases/phase-2.md) | completed |
| 3 | README, Cross-Linking, and Final Verification | [Detail](phases/phase-3.md) | completed |

## Risks & Open Questions

| Risk/Question | Impact | Mitigation/Answer |
|---------------|--------|-------------------|
| Moving docs may break existing references. | Medium | Create explicit redirect/update map and verify all links in Phase 3. |
| Scope may expand beyond lightweight lifecycle. | Medium | Keep strict phase boundaries and postpone non-critical expansions. |
| New-user guidance could still read too technical. | Medium | Use quickstart-first flow and plain-language checks during drafting. |
| Should technical docs keep original filenames after relocation? | Low | Default to preserving filenames initially for simpler migration and references. |

## Changelog

### 2026-04-09

- Created initial active plan for user-facing documentation overhaul.
- Established lightweight three-phase lifecycle and baseline requirements/scope.

### 2026-04-09 (Phase transition)

- Completed Phase 1: created root `tech-docs/` and relocated the three baseline technical docs from `docs/`.
- Transitioned execution to Phase 2 (core user-facing docs authoring) and marked Phase 2 as in progress.

### 2026-04-09 (Phase 2 completion / Phase 3 start)

- Completed Phase 2 deliverables by creating newcomer-facing docs bundle in `docs/`: `quickstart.md`, `shell-setup.md`, and skeletons for `command-guide.md`, `troubleshooting.md`, and `faq.md`.
- Verified required Phase 2 cross-links among quickstart, shell setup, and skeleton pages.
- Transitioned execution to Phase 3 and marked README/cross-link finalization as in progress.

### 2026-04-09 (Plan completion)

- Created root `README.md` with concise project overview and direct links to quickstart, shell setup, command guide, troubleshooting, FAQ, and `tech-docs/` references.
- Added reciprocal README backlinks in `docs/quickstart.md` and `docs/shell-setup.md` and included technical-doc pointers for advanced readers.
- Fixed stale config-doc path in `tech-docs/shell-hook-guarding.md` from `docs/configuration.md` to `./configuration.md`.
- Executed the Phase 3 verification intent successfully and marked all plan artifacts completed.
