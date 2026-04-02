## Context

Today, many path completion and root-based resolution flows depend on configured search roots (`DX_SEARCH_ROOTS` or config file roots). In fresh environments this often results in empty completion lists for obvious relative project paths, despite the current working directory being the natural context users expect.

This change modifies default root sourcing so cwd is always considered for root-based matching while preserving existing precedence and explicit configuration behavior.

## Goals / Non-Goals

**Goals:**
- Include cwd as an implicit default root for root-driven completion and resolution paths.
- Preserve existing precedence for direct path resolution and step-up aliases.
- Avoid duplicate candidate emissions when cwd is already present in configured roots.
- Keep behavior deterministic across shells and OSes.

**Non-Goals:**
- Changing ranking/scoring algorithms beyond root inclusion.
- Adding new user-facing config flags in this change.
- Altering stack/frecent/bookmark semantics.

## Decisions

### D1: Canonical root set includes configured roots + cwd
Build an effective root list as:
1) configured roots/environment override roots (existing behavior)
2) cwd appended if not already represented after normalization

Rationale: explicit roots remain primary while cwd becomes a universal fallback.

Alternative considered: prepend cwd before configured roots. Rejected to avoid surprising reordering for users with intentional root priority.

### D2: Reuse existing normalization and dedup semantics
Apply existing path normalization/canonicalization and dedup strategy to cwd inclusion, including macOS path normalization concerns.

Rationale: avoids platform-specific duplicate mismatches (`/var` vs `/private/var`) and keeps candidate lists stable.

### D3: Scope applies to completion and root-based resolution only
Direct-path and step-up alias precedence remain unchanged. cwd root inclusion only affects root/abbreviation/fallback candidate stages.

Rationale: preserves existing explicit precedence contract while fixing empty-default UX.

## Risks / Trade-offs

- [cwd can broaden candidate set unexpectedly] -> Keep existing max-list limits and deterministic ordering.
- [duplicate paths from cwd + configured roots] -> Normalize and dedup before downstream matching.
- [behavior shift for users relying on empty defaults] -> Document the new default and keep configured-root ordering intact.

## Migration Plan

1. Update resolver root construction to merge cwd into effective root list by default.
2. Ensure completion and resolve paths consuming root lists use the merged set.
3. Add/adjust tests for default behavior with no `DX_SEARCH_ROOTS` configured.
4. Update docs to state cwd participates by default in root-based path discovery.
5. Rollback strategy: remove implicit cwd merge and restore previous root sourcing logic.

## Open Questions

- Should there be a future opt-out flag to disable implicit cwd root inclusion for power users?
- Should cwd inclusion be conditional when cwd is outside any configured root list, or always unconditional as proposed?
