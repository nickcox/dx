## Context

`dx` currently has stack mutation/navigation behavior but no explicit read/maintenance surface for stack state. Users can infer state only by repeatedly navigating, which is slow and makes scripting brittle. The existing session model already stores `cwd`, `undo`, and `redo` in one JSON file per session, so list and clear operations can be implemented as read/targeted-write operations over the same structure.

This change is constrained by stack and hook contracts:
- Stack semantics for `push`, `undo`, and `redo` remain stable even as command names move under `dx stack`.
- Session selection rules (`--session` then `DX_SESSION`) remain unchanged.
- Output formatting should remain shell-friendly by default while supporting machine-readable JSON for tooling.

## Goals / Non-Goals

**Goals:**
- Add explicit stack inspection via `dx stack --list`.
- Add explicit stack maintenance via `dx stack --clear`.
- Consolidate stack mutation/navigation under `dx stack push|undo|redo`.
- Define deterministic output contracts for plain and JSON list output.
- Preserve existing stack navigation semantics and file schema.

**Non-Goals:**
- Changing stack mutation rules (e.g., how undo/redo are populated).
- Adding interactive UI behavior for stack listing/clearing.
- Introducing new storage backends or dependencies.

## Decisions

### Decision 1: Use `dx stack` as the single stack namespace
Stack operations are grouped under `dx stack`: inspection/maintenance via `--list` and `--clear`, plus explicit actions via `push`, `undo`, and `redo` subcommands.

- Rationale: This gives one coherent CLI surface for stack behavior (`dx stack --help`) and avoids mixed top-level/namespaced mental models.
- Alternative considered: retain top-level `push`/`undo`/`redo` as compatibility aliases. Rejected because alpha stage does not require compatibility retention.

### Decision 2: Reuse existing session file model and stack module APIs
List and clear operations will read/write the existing session JSON (`cwd`, `undo`, `redo`) via current stack persistence paths.

- Rationale: No migration is required; behavior remains consistent with existing cleanup/atomic-write expectations.
- Alternative considered: maintain separate list cache or metadata file. Rejected as unnecessary complexity and consistency risk.

### Decision 3: Plain output is path-per-line; JSON uses completion-style fields
`dx stack --list` plain output will print one absolute path per line in display order. JSON output will include `path`, `label`, and `rank` fields.

- Rationale: line-oriented plain output composes with shell tools; JSON serves automation without parsing display text.
- Alternative considered: table-formatted plain output. Rejected because tables are harder to pipe and parse.

### Decision 4: `--clear` supports scoped deletion (`undo`, `redo`, or both)
`dx stack --clear` supports explicit scope selection; default clears both stacks while preserving `cwd`.

- Rationale: users often need targeted reset (e.g., clear redo after experimentation) without losing current location context.
- Alternative considered: clear-all only. Rejected because it removes useful control and forces destructive behavior.

## Risks / Trade-offs

- [Risk] Ambiguous list ordering between undo/redo views could confuse selector use.
  - Mitigation: define explicit ordering in spec (nearest-first per stack) and keep it stable.
- [Risk] Empty-stack clear/list behavior may diverge from existing success/failure conventions.
  - Mitigation: specify explicit contracts (list succeeds with empty output; clear succeeds even when already empty).
- [Trade-off] Removing top-level stack commands is an intentional breaking change during alpha.
  - Mitigation: update hooks and docs in the same change so behavior remains consistent for users who re-run `dx init`.

## Migration Plan

1. Add new CLI parsing/dispatch for `dx stack --list`, `dx stack --clear`, and `dx stack push|undo|redo`.
2. Implement stack module functions for list views and scoped clear writes.
3. Migrate shell hooks from `dx push|undo|redo` to `dx stack push|undo|redo`.
4. Add/adjust tests for plain+JSON output, scope semantics, and empty-state behavior.
5. Update docs and shell-completion guards to reflect the namespaced stack CLI.
6. Release with no data migration.

Rollback: restore top-level stack commands and old hook invocation paths in a follow-up revert.

## Open Questions

- Resolved: JSON list output includes `path`, `label`, and `rank` to match existing completion-style JSON contracts.
- Resolved: `dx stack --clear` defaults to clearing both stacks when direction is omitted.
- Deferred: built-in limits (e.g., `--first N`) are out of scope for v1 and may be added in a follow-up change.
