## Context

`dx menu` currently formats directory replacements that need quoting as `'<path>'/`, placing the appended drill-in slash outside the quoted path. That style works in POSIX-style shells, and the parser explicitly supports it for repeated expansion. PowerShell path commands can reject that shape because the slash may be treated as a separate path fragment, while `'<path>/'` works in PowerShell and remains valid in Bash and Zsh for the intended replacement token.

## Goals / Non-Goals

**Goals:**

- Emit quoted directory replacements with the trailing slash inside the quoted path.
- Preserve compatibility with existing buffers that contain the old outside-slash shape.
- Keep replacement action schema and shell hook behavior unchanged.
- Keep non-directory and non-slashed replacement formatting unchanged.

**Non-Goals:**

- Moving all quoting responsibility into shell hooks.
- Adding a shell selector to `dx menu`.
- Changing candidate sourcing, filtering, ranking, or replacement bounds.
- Removing parser support for the old outside-slash representation.

## Decisions

1. Change formatting at the shared replacement formatter.

The replacement formatter should decide whether a trailing slash is needed before applying shell quoting. For a quoted directory replacement, the slash becomes part of the string being quoted, producing `'/path with spaces/'` instead of `'/path with spaces'/`.

Alternative considered: make shell hooks own quoting. That is architecturally cleaner but larger because it changes the JSON action contract and requires shell-specific escaping in every hook.

2. Keep parser compatibility for both quote shapes.

`unquote_shell_quoted` should continue to parse `'/path with spaces'/` and should also parse `'/path with spaces/'` to the same raw query. This avoids breaking repeated expansions in existing shell buffers and makes the change safely forward-compatible.

3. Coordinate with path-mode directory slash behavior.

The active `slash-path-mode-directories` change should use this same quote placement rule for mapped `path` mode directories. It should not introduce a separate requirement that mandates slash outside quotes.

## Risks / Trade-offs

- Some POSIX users may be used to the visible outside-slash style -> Bash and Zsh accept the inside-slash style, and the resulting token is more portable across supported shells.
- Existing tests encode the outside-slash behavior -> Update tests to assert new output and add compatibility tests for parsing old buffers.
- Parser ambiguity around embedded quotes remains -> Preserve the existing escaping approach and only extend slash handling.
