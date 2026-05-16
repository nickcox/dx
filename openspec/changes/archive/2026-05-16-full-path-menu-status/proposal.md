## Why

The `dx menu` status row is the user's confirmation surface for the currently selected destination, but it currently reuses compact display labels that may be cwd-relative. Showing the full resolved path by default makes the status row answer exactly what Enter will choose without changing shell-friendly replacement behavior.

## What Changes

- Display the full resolved selected path in the interactive menu status row by default.
- Keep candidate list labels compact and unchanged.
- Keep Enter/selection replacement behavior unchanged, including existing relative-style insertion for path queries.
- Do not add user configuration for status path style in this change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dx-menu`: Clarifies and changes selected-item status context to use the full resolved selected path by default, while preserving replacement semantics.

## Impact

- Affected code: `src/menu/tui.rs` status-row selected path source and related tests.
- Affected behavior: visual-only status display in the interactive menu.
- Unaffected behavior: candidate ordering, candidate labels, filtering, selected candidate identity, JSON action schema, and Enter replacement formatting.
- Dependencies: none.
