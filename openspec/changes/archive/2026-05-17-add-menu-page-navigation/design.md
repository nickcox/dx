## Context

The interactive menu currently maps keyboard input to a small internal action enum in `src/menu/tui.rs`. Arrow keys and Tab move selection one candidate at a time, with separate grid-aware handling for vertical movement in multicolumn layout. PageUp and PageDown are currently unmapped and therefore ignored.

The menu already computes `layout.visible_rows` and `layout.metrics.columns` on each render cycle. Those values are sufficient to define a visible page without adding state or changing rendering.

## Goals / Non-Goals

**Goals:**

- Map PageDown to move selection forward by one visible page.
- Map PageUp to move selection backward by one visible page.
- Define page size as visible rows in single-column layout.
- Define page size as visible rows multiplied by active columns in multicolumn layout.
- Clamp page movement at list boundaries.
- Preserve existing line movement, filtering, selection, and action output behavior.

**Non-Goals:**

- No configurable keybindings.
- No half-page or Home/End navigation in this change.
- No changes to shell hooks or JSON action protocol.
- No changes to candidate sourcing or ordering.

## Decisions

### Decision: Add a Page Movement Key Action

Represent PageUp/PageDown as a distinct key action rather than overloading linear movement.

Alternatives considered:
- Convert PageUp/PageDown directly into a fixed linear delta in key mapping: rejected because page size depends on current layout and can change after filtering or resize.
- Treat PageUp/PageDown as repeated arrow movement: rejected because it would be less direct to test and may accidentally inherit wrap behavior.

### Decision: Clamp Instead of Wrap

Page navigation SHALL clamp to the first or last candidate when the requested page movement would go past the bounds.

Alternatives considered:
- Wrap like single-step arrow movement: rejected because page navigation conventionally moves within the current result set and stopping at boundaries is less surprising.

### Decision: Use Visible Cells as the Grid Page Size

In grid mode, page size is `visible_rows * columns`, which corresponds to one visible gridful of candidates in row-major order.

Alternatives considered:
- Move only by visible rows in grid mode: rejected because row-major selection would advance only part of a visible page.
- Move by terminal height independent of columns: rejected because the layout already exposes the exact visible row and column count.

## Risks / Trade-offs

- [Risk] Filtering can shrink visible rows and change page size while the menu is open. -> Mitigation: compute page size from the current layout each key event.
- [Risk] Existing wrap behavior for arrows could be confused with page clamping. -> Mitigation: keep page movement in a separate helper with focused tests.
- [Risk] Very small menu layouts could produce zero visible rows. -> Mitigation: page size SHALL have a minimum effective movement of one candidate when candidates exist.

## Migration Plan

No migration is required. PageUp/PageDown currently do nothing, so enabling them is additive and does not change existing key behavior.
