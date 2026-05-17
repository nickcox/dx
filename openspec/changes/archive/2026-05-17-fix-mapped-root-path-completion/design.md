## Context

Mapped external commands use `dx menu --mode path|directory|file` and are sourced through `source_mapped_filesystem_candidates` in `src/menu/mod.rs`. That path combines dx-smart directory candidates with direct filesystem child listing so mapped commands can surface both directories and files.

For the query `/`, the mapped parent parser trims the trailing slash, obtains an empty parent query, and currently treats that empty parent as cwd. This is correct for empty or bare relative queries, but incorrect for rooted input: `/` should refer to the filesystem root.

## Goals / Non-Goals

**Goals:**

- Treat mapped-command queries beginning with `/` as rooted filesystem queries.
- Make `/` list children of `/` only, subject to mapped mode filtering.
- Make `/<prefix>` filter children under `/` by basename prefix.
- Preserve cwd child listing for empty query and bare relative query.
- Preserve existing behavior for `./`, `../`, and `~/` path forms.

**Non-Goals:**

- No changes to generated shell hooks or `DX_MENU_COMMAND_MAPPINGS` parsing.
- No changes to built-in `dx complete paths` behavior.
- No changes to replacement action shape or query-style label rendering.
- No command-specific parsing beyond the active token.

## Decisions

### Decision: Fix Parent Resolution in Mapped Filesystem Sourcing

Adjust `mapped_parent_directories` so an empty `parent_query` is interpreted based on the original query shape:

- `query == ""` remains cwd.
- bare relative query such as `src` remains cwd plus leaf prefix.
- rooted query `/` uses `/` with an empty leaf prefix.
- rooted query `/U` uses `/` with leaf prefix `U`.

Alternatives considered:
- Filter cwd children after combining candidates: rejected because parent resolution should avoid producing incorrect cwd candidates in the first place.
- Reuse `dx complete paths` output directly for mapped `path`: rejected because mapped `path` must include files as well as directories, while `paths` completion is directory-oriented.

### Decision: Preserve dx-smart Directory Injection

Keep the existing dx-smart candidate phase for mapped `path` and `directory` modes, but rely on de-duplication and corrected filesystem parent resolution so rooted queries do not add cwd children.

Alternatives considered:
- Disable dx-smart for all rooted mapped queries: rejected because existing smart behavior may still be useful for explicit roots and should only be constrained where it causes incorrect cwd injection.

## Risks / Trade-offs

- [Risk] Rooted absolute path parsing can regress non-root absolute prefixes. -> Mitigation: add tests for `/`, `/<prefix>`, and cwd-relative forms.
- [Risk] File-mode behavior may differ subtly because `dx complete paths` only lists directories. -> Mitigation: test through mapped `path`, `directory`, and/or `file` candidate sourcing rather than comparing directly to `paths` completion for every mode.
- [Risk] macOS path canonicalization may differ for `/var` and `/private/var`. -> Mitigation: avoid brittle canonical path equality for this fix; assert candidate parent/prefix membership instead.

## Migration Plan

No migration is required. Existing mapped commands keep their configuration. Rooted mapped-command completions become narrower and more correct by no longer mixing cwd children into `/` results.
