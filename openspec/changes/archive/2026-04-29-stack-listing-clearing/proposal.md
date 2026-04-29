## Why

`dx` has stack navigation behavior but lacks first-class visibility and maintenance commands for stack state. Users need an explicit way to inspect available back/forward targets and clear stale history without relying on navigation side effects.

## What Changes

- Add stack inspection via `dx stack --list` to show undo and redo entries in user-readable and machine-readable forms.
- Add stack maintenance via `dx stack --clear` to clear undo and/or redo entries for a session.
- Consolidate all stack operations under `dx stack ...` with `dx stack push`, `dx stack undo`, and `dx stack redo` as canonical command forms.
- Remove top-level `push`, `undo`, and `redo` entrypoints during alpha.
- Define stable output and filtering semantics for stack listing so shell integrations and scripts can depend on it.

## Capabilities

### New Capabilities
- `stack-inspection`: List stack entries (undo/redo) with plain and JSON output suitable for interactive use and scripting.
- `stack-maintenance`: Clear stack entries (undo, redo, or both) for the active session with predictable success/failure behavior.

### Modified Capabilities
- `session-stacks`: Extend session-stack requirements to include listing and clearing operations and their output contracts.
- `shell-hooks`: Update generated hooks to invoke namespaced stack commands (`dx stack push|undo|redo`) rather than removed top-level forms.

## Impact

- Affected code: CLI command parsing/dispatch, stack CLI module, session storage read/write pathways, and stack-focused tests.
- Affected behavior: stack workflows move to the `dx stack` namespace (`--list`, `--clear`, `push`, `undo`, `redo`) as the canonical interface.
- Dependencies: no new third-party dependencies expected.
