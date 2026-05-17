## Context

`dx menu` currently renders candidate item labels with a compact display function based primarily on whether the candidate is under the current working directory, under the user's home directory, or elsewhere. This produces labels such as `./src` for cwd-local candidates even when the user typed a bare command like `cd <tab>` or `cd s<tab>`.

The menu already preserves query style for accepted replacements. Candidate labels should follow the same general principle visually, while remaining independent from the status row, which now displays the full resolved selected path by default.

## Goals / Non-Goals

**Goals:**

- Render filesystem candidate labels in the same path style the user is typing.
- Use bare cwd-relative labels for empty and bare relative input.
- Preserve explicit `./`, `../`, `~/`, and absolute styles in candidate labels.
- Apply this behavior consistently to list and grid item labels.

**Non-Goals:**

- Change candidate sourcing, filtering, ranking, or selected candidate identity.
- Change status-row selected path display.
- Change accepted replacement text or shell buffer replacement bounds.
- Add user configuration for label style.

## Decisions

### Replace Boolean Relative Rendering with Query Display Style

The existing `prefer_relative_paths` boolean is too coarse for item labels. Introduce a small internal display-style model derived from the active query and mode:

- Bare relative: empty query or relative token without explicit `./` or `../`, rendered as `src`.
- Dot relative: query starts with `./`, rendered as `./src`.
- Parent relative: query starts with one or more `../` segments, rendered as `../sibling` or `../../outer` where appropriate.
- Home relative: query starts with `~` or `~/`, rendered as `~/path` for candidates under home.
- Absolute: query starts with `/`, rendered as the absolute candidate path.

This keeps label rendering explicit and avoids deriving multiple user-visible behaviors from a single boolean.

### Limit Scope to Filesystem Path Modes

Query-style label rendering should apply to filesystem path modes: `cd` paths and mapped `path`, `directory`, and `file` modes. Non-filesystem menu modes such as ancestors, frecents, recents, and stack should continue using their existing compact labels unless a future change scopes them explicitly.

### Keep Status and Replacement Separate

The status row continues to show the full resolved selected path. Accepting a candidate continues to use the existing replacement formatter, so label display changes do not alter shell insertion behavior.

## Risks / Trade-offs

- Bare labels can be less explicit than `./` labels. Mitigation: they match the user's bare input style and the status row still confirms the full resolved path.
- Parent-relative rendering can be tricky when candidates are not naturally representable under the typed parent prefix. Mitigation: only use parent-relative labels when a relative path from cwd can be computed; otherwise fall back to an existing safe absolute or home display.
- Absolute-prefix casing and symlink spelling may differ from filesystem canonicalization on some platforms. Mitigation: preserve the candidate path display as provided by the resolver; do not introduce canonicalization solely for labels.
