## 1. Portable Path Foundations

- [x] 1.1 Add a shared internal path-query classifier for home-relative, absolute, root-relative, explicit-relative, drive-relative, trailing-separator, root-anchor, and abbreviation-segment handling.
- [x] 1.2 Rewrite lexical path normalization to preserve native prefixes and roots, prevent traversal above roots, and preserve empty relative normalization.
- [x] 1.3 Add platform-gated unit tests for Unix backslash names, Unix roots, Windows drive roots, root-relative paths, UNC lexical roots, and rejected drive-relative paths.

## 2. Resolver Semantics And Errors

- [x] 2.1 Migrate direct resolution, home expansion, and fallback-policy construction to the shared path-query classifier.
- [x] 2.2 Preserve native root anchors for root-scoped fallback and use separator-aware abbreviation and fallback segment parsing.
- [x] 2.3 Add fallible exact-resolution metadata and traversal handling that distinguishes not-found misses from actionable filesystem errors while retaining independently skippable configured roots.
- [x] 2.4 Update resolver unit and CLI tests for portable direct paths, root-anchored fallback, whitespace-bearing names, and filesystem error propagation.

## 3. Completion Identity And Discovery

- [x] 3.1 Remove query and selector trimming from resolver completion, completion filtering, session filtering, and navigation selection while retaining zero-length empty semantics.
- [x] 3.2 Migrate filesystem-prefix expansion, trailing-separator handling, and candidate filtering to shared native path-query semantics.
- [x] 3.3 Preserve `PathBuf` identity for completion deduplication, ordering tie-breaks, labels, and index selection; keep lossy rendering only at display boundaries.
- [x] 3.4 Keep completion discovery best-effort for unreadable, invalid, and disappearing entries and add regression tests for available sibling candidates.

## 4. Menu And Shell Formatting

- [x] 4.1 Migrate mapped filesystem candidate parent extraction and query expansion to the shared path-query classifier.
- [x] 4.2 Extract portable path-label and relative-rendering helpers from Unix-only TUI code without enabling the Windows TUI.
- [x] 4.3 Update shell replacement formatting to preserve drive and UNC prefixes, use native trailing separators, and fail safely for paths that cannot cross the UTF-8 action boundary.
- [x] 4.4 Add Unix and Windows tests for mapped completion, labels, duplicate labels, whitespace-bearing paths, PowerShell quoting, drive paths, and UNC lexical behavior.

## 5. Native CI And Verification

- [x] 5.1 Convert Unix-specific integration fixtures and shell tests to platform-native paths or `cfg(unix)` gates.
- [x] 5.2 Run `cargo test --locked` in a GitHub Actions Ubuntu and Windows core-test matrix while retaining native Windows PowerShell smoke coverage.
- [x] 5.3 Run formatting, focused resolver/completion/menu tests, the full Rust suite, Clippy, and OpenSpec strict validation; confirm native Windows CI passes.
