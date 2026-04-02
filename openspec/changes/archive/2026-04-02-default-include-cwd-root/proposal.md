## Why

Users generally expect completion and resolution to work relative to where they currently are, even before configuring custom search roots. Requiring explicit `DX_SEARCH_ROOTS` setup for common in-repo navigation creates surprising empty results and unnecessary setup friction.

## What Changes

- Include the current working directory as an implicit search root by default when evaluating path completion and path resolution candidates.
- Preserve explicit configuration precedence: configured search roots and environment overrides still apply, but cwd is always considered unless explicitly disabled in the future.
- Keep existing direct-path and step-up precedence behavior unchanged; this change affects root-based candidate sourcing defaults.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `completions`: update default candidate sourcing behavior so cwd participates in root-based path completions without extra user configuration.
- `path-resolution`: update fallback/abbreviation search behavior so cwd is considered as a default root for query resolution.

## Impact

- Affected code: resolver root collection/normalization logic and completion candidate sourcing paths.
- Affected tests/docs: completion and resolution default-behavior tests; environment/config documentation for search roots.
- No new dependencies or external APIs.
