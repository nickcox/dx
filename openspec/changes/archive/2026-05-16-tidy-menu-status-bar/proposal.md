## Why

The `dx menu` status row currently repeats the full effective filter query with a literal `filter:` label before the selected path, which makes the most important status information harder to scan. The menu should make the pending selection primary while still surfacing any in-menu refinement in a compact, predictable location.

## What Changes

- Show selected-item context at the left side of the status row.
- Show the in-menu typed refinement only when the user has typed refinement characters after opening the menu.
- Display only the typed refinement, not the initial query parsed from the shell buffer.
- Right-align the refinement indicator using a compact search-style marker instead of the literal `filter:` label.
- Treat overflow text as secondary metadata that may be omitted before selection or refinement context is lost.
- Add status-row truncation rules so long selections and long refinements remain usable in narrow terminals.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dx-menu`: Updates the interactive status area behavior for selected-item context, overflow metadata, and typed refinement display.

## Impact

- Affected code: `src/menu/tui.rs` status rendering and related tests.
- Affected behavior: visual-only interactive menu status row layout; no change to candidate sourcing, filtering semantics, shell replacement JSON, or shell hook protocol.
- Dependencies: none.
