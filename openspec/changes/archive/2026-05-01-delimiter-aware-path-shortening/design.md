## Context

`dx` currently implements abbreviated matching as simple segment-start prefix checks. That works for queries like `pro/sr/com`, but it does not support the `cd-extras` path-shortening patterns the project is explicitly borrowing from, especially delimiter-aware fragments such as `cd-e` and `.sdk`, or doubled-period gaps such as `p..shell` and `s..32`.

This gap shows up in three places that should behave consistently: `dx resolve`, `dx complete paths`, and shell auto-cd flows that forward path-like command-not-found inputs into `dx resolve`. The current code also has a conservative shell heuristic that ignores delimiter-bearing command words, so even if resolver matching improves, auto-cd would still miss those inputs unless the heuristic is updated.

## Goals / Non-Goals

**Goals:**
- Support `cd-extras`-style shortening within a single path segment using the default word delimiters `.`, `_`, and `-`.
- Support doubled periods (`..`) inside a segment as an ordered in-segment gap operator.
- Keep resolver, completion, and auto-cd behavior aligned by reusing one matching model.
- Ensure delimiter-aware and doubled-period matching obey the existing resolver case-sensitivity setting.
- Preserve existing precedence: direct paths and multi-dot step-up aliases continue to win before abbreviated matching.
- Preserve existing ambiguity behavior: if the richer matcher produces multiple candidates, resolution still fails unless the caller requested list/json style output.

**Non-Goals:**
- Introducing general fuzzy matching, regex syntax, or scoring beyond the existing deterministic candidate ordering.
- Making delimiter configuration user-customizable in this change.
- Changing bookmark precedence, fallback root selection, or direct filesystem-prefix completion behavior.
- Reworking shell wrappers beyond the command-not-found path-like heuristic.

## Decisions

### D1: Segment matching becomes operator-aware, with a fast prefix path retained

The resolver will keep today's fast prefix behavior for query segments that contain no shortening operators.

If a query segment contains a supported word delimiter (`.`, `_`, `-`) or an in-segment doubled-period sequence (`..`), matching will use an operator-aware matcher instead of `starts_with`.

Operator tokenization is left-to-right, with `..` recognized before single `.` delimiter parsing so mixed inputs such as `a..b.c` are interpreted deterministically.

Rationale: most queries remain simple prefixes, so the fast path preserves current performance. Only the smaller set of delimiter/gap queries pay the extra matching cost.

Alternatives considered:
- Translate every segment into a regex-like matcher: rejected because it adds complexity and overhead to the common case.
- Replace prefix matching with full fuzzy matching: rejected because it would weaken predictability and increase ambiguity.

### D2: Word delimiters preserve their literal identity but allow omitted text around them

Delimiter-aware shortening follows the `cd-extras` mental model: the query keeps the literal delimiter character and matches ordered fragments around it while allowing skipped text before and/or after the delimiter boundary.

Examples:
- `cd-e` matches `cd-extras`
- `.sdk` matches `Microsoft.PowerShell.SDK`
- `foo_bar` matches `foo_long_bar`

The delimiter character itself remains significant: `cd-e` does not implicitly match `cd_extras`.

Delimiter-aware matching uses the same case-sensitivity behavior as existing prefix matching by honoring `resolve.case_sensitive`.

Rationale: this matches the documented `cd-extras` behavior more closely than treating every delimiter as interchangeable.

Alternatives considered:
- Normalize all word delimiters to one generic separator: rejected because it over-matches and diverges from the source behavior.

### D3: Doubled periods act as in-segment wildcard gaps only during abbreviated matching

Within a query segment, each `..` sequence matches an arbitrary substring in the same directory name while preserving left-to-right fragment order. That substring may include delimiter characters because the operator spans any interior text within the segment.

Examples:
- `p..shell` matches `PowerShell`
- `s..32` matches `System32`

This operator applies only during abbreviated matching. Queries that are entirely multi-dot tokens (for example `...`, `....`) continue to use the existing step-up alias logic because that stage runs earlier in precedence.

Rationale: this adds the desired shorthand without disturbing established traversal behavior.

Alternatives considered:
- Treat any repeated dots as wildcard syntax everywhere: rejected because it would conflict with existing multi-dot navigation aliases.

### D4: One shared segment matcher is used by abbreviation and fallback-root collection

The implementation should centralize segment matching in one helper used by:
- multi-segment abbreviation traversal
- fallback-root single-segment matching
- fallback-root multi-segment matching

This keeps `dx resolve` and `dx complete paths` aligned because both already reuse the resolver pipeline.

Rationale: today, abbreviation and fallback single-segment matching use separate `starts_with` paths. Reusing one helper avoids semantic drift and duplicate edge-case logic.

Alternatives considered:
- Update only abbreviation traversal and leave fallback matching prefix-only: rejected because it would create inconsistent behavior for otherwise equivalent queries.

### D5: Shell command-not-found heuristics broaden to include delimiter-shortened tokens

The command-not-found path-like heuristic will expand beyond slash/dot/home/multi-dot inputs to also attempt `dx resolve` for single command words that contain supported shortening signals:
- a supported word delimiter (`.`, `_`, `-`)
- or an in-segment doubled-period sequence (`..`)

If `dx resolve` fails, the handler still falls back to the shell's standard command-not-found behavior.

This change applies only when users opt into generated command-not-found handlers via `dx init --command-not-found`; default hook generation remains unchanged.

Rationale: without this change, auto-cd would continue to miss queries like `cd-e` and `p..shell` even after resolver support exists.

Alternatives considered:
- Keep shell heuristics unchanged and require explicit `cd <query>` for delimiter-aware shortcuts: rejected because it leaves an avoidable behavior gap between explicit cd and auto-cd flows.

## Risks / Trade-offs

- [Risk] Delimiter-aware matching broadens the candidate set and may surface new ambiguities → Mitigation: keep existing ambiguity failure/list behavior and add targeted tests for ambiguous delimiter queries.
- [Risk] Command-not-found handlers will invoke `dx resolve` for some unknown hyphenated/underscored commands that are not intended as paths → Mitigation: this only happens after the shell already failed command lookup, and failure still returns the shell's normal command-not-found result.
- [Risk] Matcher complexity could regress interactive latency → Mitigation: preserve the simple prefix fast path and keep operator-aware matching scoped to single-segment name checks, not whole-tree fuzzy scans.
- [Trade-off] Literal delimiter preservation is stricter than a generic separator wildcard → This is intentional for predictability and closer `cd-extras` parity.

## Migration Plan

1. Add unit tests describing delimiter-aware and doubled-period segment matching semantics.
2. Implement a shared segment matcher and wire it into abbreviation and fallback-root resolution paths.
3. Extend completion-path tests to verify `dx complete paths` returns the same new matches.
4. Update shell hook heuristic tests so delimiter-shortened auto-cd inputs are forwarded to `dx resolve`.
5. Update OpenSpec delta specs for `path-resolution`, `completions`, and `shell-hooks` to codify the new behavior.
6. Update user-facing docs/examples for path shortening.
7. Rollback strategy: restore prefix-only matching and revert the shell heuristic expansion.

## Open Questions

- Should a future change make supported word delimiters configurable, or is fixed parity with `cd-extras` defaults sufficient long-term?
- If command-not-found over-matching proves noisy for hyphenated commands, should delimiter-aware auto-cd become configurable per shell?
