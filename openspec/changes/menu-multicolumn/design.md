## Context

`dx menu` currently renders candidates in a single vertical list. This is simple but inefficient on wide terminals where many rows are available and candidate scanning becomes slower as list length grows. We want an optional multicolumn layout that improves density while preserving current defaults and action/output contracts.

The menu runtime already owns keyboard handling, filtering, and candidate selection in `src/menu/tui.rs`. The safest path is to add a layout mode switch and keep selection semantics and JSON actions unchanged.

## Goals / Non-Goals

**Goals:**
- Add an opt-in multicolumn layout for menu candidate display.
- Use `DX_MENU_ITEM_MAX_LEN` to control cell truncation width and dynamic column count.
- Keep single-column as default behavior.
- Preserve existing replace/noop JSON protocol and shell hook interaction.
- Define deterministic cursor movement and wrapping/clamping in a multi-column grid.

**Non-Goals:**
- Changing shell hook protocol or adding per-shell parsing logic.
- Replacing existing filtering semantics.
- Adding complex user theme/tooltip customization.
- Making multicolumn automatic by default when configuration is absent.

## Decisions

### Decision 1: Opt-in via `DX_MENU_ITEM_MAX_LEN`
Use `DX_MENU_ITEM_MAX_LEN` as the sole multicolumn configuration input.
- If unset, empty, non-numeric, or `< 1`: render single-column (current behavior).
- If `>= 1`: enable multicolumn calculations with this value as max visible item length before truncation.

- Rationale: user intent is usually about visible label width, not explicit column count; width-driven behavior adapts naturally to terminal resize.
- Alternative considered: explicit `DX_MENU_COLUMNS=<n>`. Rejected because fixed columns are brittle across terminal sizes and produce less predictable truncation.

### Decision 2: Dynamic columns from width budget
When multicolumn is enabled, compute:
- `cell_width = item_max_len + padding`
- `columns = max(1, floor(terminal_width / cell_width))`

If computed columns is `1`, rendering naturally behaves like single-column for that frame.

- Rationale: deterministic and adaptive without additional user tuning.
- Alternative considered: pre-defined breakpoints. Rejected as less precise and harder to reason about.

### Decision 3: Grid is row-major with stable source ordering
Candidates keep their source order; layout maps candidates into a row-major grid. Selection index remains a single linear index and navigation maps arrow/tab movements onto grid neighbors.

- Rationale: preserves deterministic ordering and avoids re-ranking side effects.
- Alternative considered: column-major fill. Rejected as less intuitive with existing nearest-first ordering.

### Decision 4: Width-aware truncation with full path in status line
Each cell truncates labels to `DX_MENU_ITEM_MAX_LEN` (when enabled), while status line continues to show full selected path.

- Rationale: keeps layout stable while retaining full-value context.
- Alternative considered: wrapping cell text across lines. Rejected due to variable row heights and navigation complexity.

## Risks / Trade-offs

- [Risk] Navigation confusion in 2D movement at list boundaries.
  - Mitigation: specify deterministic wrap/clamp behavior and add focused integration tests.
- [Risk] Reflow jitter when filtering changes candidate count.
  - Mitigation: reset selection consistently on filter changes and keep stable row-major mapping.
- [Trade-off] Truncation can hide distinguishing suffixes.
  - Mitigation: keep full selected path in status row and deterministic selection motion.

## Migration Plan

1. Add `DX_MENU_ITEM_MAX_LEN` parsing with invalid-value fallback to default single-column.
2. Implement dynamic geometry calculation (`cell_width`, `columns`) for multicolumn mode.
3. Implement row-major grid rendering with truncation.
4. Implement navigation mappings for left/right/up/down/tab flows in grid mode.
5. Add tests for activation/fallback, ordering, truncation, and behavior parity.
6. Document usage and examples for `DX_MENU_ITEM_MAX_LEN`.

Rollback: unset/ignore `DX_MENU_ITEM_MAX_LEN` and route all renders through existing single-column path.

## Answered Questions

- Should horizontal movement wrap across rows when stepping left/right off edges, or clamp within row?
answer: yes, it should wrap

- Should multicolumn mode keep scrollbar semantics or suppress scrollbar when grid is active?
answer: yes, we should still show a scrollbar
