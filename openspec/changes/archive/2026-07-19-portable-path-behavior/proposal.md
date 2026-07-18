## Why

`dx` publishes a Windows binary but its resolver, completion, and menu path handling still assumes Unix roots and `/` separators. This creates incorrect behavior for drive paths, UNC roots, whitespace-bearing names, and filesystem errors that is not covered by native Windows CI.

## What Changes

- Define portable query semantics for Unix and Windows filesystem paths, including native roots, accepted separators, home expansion, and explicit relative paths.
- Define root-anchored fallback behavior for native filesystem roots instead of only `/`.
- Preserve significant query whitespace and `PathBuf` identity through candidate collection, filtering, deduplication, and selection.
- Define exact-resolution filesystem error propagation separately from best-effort interactive completion and menu discovery.
- Add native Windows test coverage in GitHub Actions alongside the existing Unix test coverage.
- **BREAKING** Reject Windows drive-relative queries such as `C:work` rather than resolving them against an implicit per-drive working directory.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `path-resolution`: portable filesystem query interpretation, root-anchored fallback, and resolver error behavior.
- `completions`: portable path filtering, candidate identity, selector whitespace semantics, and Windows test coverage.
- `dx-menu`: portable mapped filesystem completion and shell replacement rendering.

## Impact

- Affects `src/resolve`, `src/complete`, `src/menu`, and `src/cli/menu.rs`.
- Updates resolver and CLI integration tests to use platform-native fixtures and adds Windows-only cases.
- Updates `.github/workflows/push-pr-ci.yml` to run Rust tests on `windows-latest`.
- Uses existing standard-library path APIs and the existing `dirs` dependency; no new runtime dependency is required.
