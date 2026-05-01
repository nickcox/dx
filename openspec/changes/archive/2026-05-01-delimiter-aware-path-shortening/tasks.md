## 1. Shared Segment Matcher

- [x] 1.1 Add unit tests for segment matching that cover plain prefixes, delimiter-aware fragments (`cd-e`, `.sdk`, `foo_bar`), doubled-period gaps (`p..shell`, `s..32`) including delimiter-bridging cases (`f..bar` -> `foo-bar`), mixed tokenization (`a..b.c`), and case-sensitivity behavior.
- [x] 1.2 Implement a shared segment-matching helper in `src/resolve/abbreviation.rs` that keeps the existing fast prefix path for simple segments and uses operator-aware matching for delimiter/gap queries.
- [x] 1.3 Ensure doubled periods are tokenized before single-dot delimiter parsing and that delimiter identity (`.`, `_`, `-`) remains significant during matching.

## 2. Resolver and Completion Integration

- [x] 2.1 Replace existing `starts_with`-based segment checks in abbreviation traversal with the shared matcher.
- [x] 2.2 Replace existing fallback-root single-segment and multi-segment matching checks with the shared matcher so `dx resolve` and `dx complete paths` stay aligned.
- [x] 2.3 Add resolver and completion tests for delimiter-aware and doubled-period queries, including ambiguous-match cases and precedence over step-up aliases/direct paths.

## 3. Shell Hook Heuristic Updates

- [x] 3.1 Update command-not-found path-like heuristics in generated shell hooks to forward delimiter-shortened and doubled-period queries to `dx resolve` only when `--command-not-found` is enabled.
- [x] 3.2 Add shell-hook tests covering `cd-e` and `p..shell` forwarding, plus confirmation that plain words still fall through to standard command-not-found behavior.
- [x] 3.3 Add Fish-specific shell-hook tests to confirm literal existing directories remain handled by Fish auto-cd while abbreviated and delimiter-shortened non-literal inputs are forwarded to `dx resolve`.

## 4. Documentation and Verification

- [x] 4.1 Update user-facing path-shortening documentation/examples to describe delimiter-aware matching and doubled-period gaps.
- [x] 4.2 Run targeted resolver, completion, and shell-hook tests and adjust any affected expectations.
- [x] 4.3 Run the full test suite to confirm no regressions in path resolution, completion ordering, or shell integration behavior.
