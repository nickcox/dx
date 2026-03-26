## Why

Path resolution is the core user-facing behavior of `dx`: every `cd` shortcut, abbreviation, and auto-cd flow depends on fast, predictable translation from a user query to an absolute directory. Defining this capability first reduces ambiguity for shell hook behavior, completion/menu integration, and migration from `cd-extras`.

## What Changes

- Define a dedicated `path-resolution` capability spec for `dx resolve`, including deterministic parsing and ranking rules for traditional traversal (`..`, `~`), step-up aliases, abbreviated segments, and fallback search paths.
- Specify how `resolve` behaves across direct `cd` invocation and command-not-found forwarding (auto-cd style usage), including success and failure semantics.
- Define output and error contracts so shell hooks can safely consume `resolve` results without guessing.
- Define matching precedence to avoid surprising resolutions when multiple candidates are possible.
- Define performance and safety expectations for interactive shell usage (low latency, no recursion loops from shell handlers).

## Capabilities

### New Capabilities
- `path-resolution`: Query-to-absolute-path resolution rules, precedence, edge-case handling, and shell-consumable output contracts for `dx resolve`.

### Modified Capabilities
_(none — no existing specs)_

## Impact

- Introduces `openspec/changes/path-resolution/specs/path-resolution/spec.md` as a focused requirement contract for `dx resolve`.
- Establishes required behavior consumed by shell hooks (`cd` wrapper and command-not-found forwarding) in Bash, Zsh, Fish, and PowerShell.
- Constrains CLI interface and testing strategy for resolution correctness, ambiguity handling, and interactive latency.
