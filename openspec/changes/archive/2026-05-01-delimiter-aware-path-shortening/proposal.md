## Why

`dx` currently resolves abbreviated paths using only segment-start prefixes, which misses common `cd-extras` shortcuts like delimiter-based fragments (`cd-e`, `.sdk`) and double-period gaps inside a name (`p..shell`, `s..32`). Adding these shortening rules closes a noticeable migration gap and makes `dx` feel closer to the path-shortening behavior users already expect.

## What Changes

- Extend abbreviated path matching so a query fragment can match across word delimiters inside a directory name, using `cd-extras`-style delimiter expansion.
- Treat pairs of periods inside a query segment as an in-segment gap operator, allowing the matcher to skip arbitrary characters between adjacent fragments during abbreviated matching only.
- Apply the same shortening semantics to `dx resolve`, `dx complete paths`, and shell auto-cd flows that forward explicit path-shortening patterns to `dx resolve`.
- Preserve existing precedence, ambiguity handling, root selection, and direct filesystem path behavior; only abbreviation matching semantics and shell forwarding heuristics change.
- Keep direct path resolution and multi-dot step-up aliases higher precedence than delimiter-aware abbreviation matching so existing traversal behavior remains unchanged.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `path-resolution`: extend abbreviated segment matching so queries can expand around word delimiters and across doubled-period gaps within a single path segment.
- `completions`: update `paths` mode candidate matching so it returns delimiter-aware and doubled-period abbreviation matches via the resolver's collection pipeline.
- `shell-hooks`: broaden command-not-found path-like detection so delimiter-shortened queries can use the same auto-cd resolution path as explicit `cd` invocations.

## Impact

- Affected code: resolver abbreviation/fallback matching, candidate collection for `dx complete paths`, and shell hook command-not-found heuristics.
- Affected tests/docs: path resolution and shell-hook behavior tests, plus documentation/examples for shortened path syntax.
- No new external services or runtime dependencies are required.
