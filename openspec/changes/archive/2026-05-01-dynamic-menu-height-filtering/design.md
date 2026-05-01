## Context

`dx menu` currently computes a render area once at startup and keeps that fixed viewport through the interactive loop. Live filtering already re-queries candidates per keystroke, but the menu body does not shrink when candidate count drops. This produces extra empty rows and can look visually stale, especially in narrow terminal windows and in borderless mode where separators/scrollbars are tightly coupled to list height.

The menu must also keep strong terminal safety guarantees: stdout remains reserved for JSON action output, interaction occurs on TTY channels, and all exits restore terminal state. Any dynamic resizing must preserve those guarantees and avoid redraw artifacts (stale lines, prompt overlap, half-cleared borders).

## Goals / Non-Goals

**Goals:**
- Shrink menu height dynamically during interactive filtering as candidate count narrows.
- Preserve interactive loop behavior (selection, cancel, no-match editing, query refinement) while resizing.
- Keep cleanup and redraw terminal-safe in both bordered and borderless modes.
- Ensure multicolumn mode recomputes layout deterministically with reduced height.

**Non-Goals:**
- Expanding beyond initially reserved vertical space (no upward growth contract in this change).
- Changing JSON action schema or shell-hook protocol.
- Redesigning filtering semantics, scoring, or candidate sourcing behavior.

## Decisions

1. **Height becomes frame-derived from current filtered candidate set**
   - Compute effective visible rows each frame from current candidate count/layout rows and configured max rows.
   - Keep a minimal interactive footprint when there are no matches (status row + minimal body).
   - **Why:** keeps UI responsive and avoids dead space while preserving existing max-row ceiling.
   - **Alternative considered:** fixed startup height with only content redraw. Rejected because it keeps empty area and does not satisfy dynamic shrink UX.

2. **Constrain dynamic resize to shrink-only within pre-reserved area**
   - Initial reservation remains the upper bound; runtime only reduces rendered height within that reserved block.
   - **Why:** avoids complex runtime scrolling/re-reservation logic and prompt-jump risk while still delivering the requested shrink behavior.
   - **Alternative considered:** bidirectional resize (grow/shrink). Deferred due to higher risk of terminal glitches and scroll accounting bugs.

3. **Explicit clear of vacated rows on every shrink transition**
   - Track previously rendered height; when new height is smaller, clear rows in the trailing region before/after draw as needed.
   - Apply same rule for bordered and borderless separators/scrollbars.
   - **Why:** prevents stale borders/scrollbar glyphs and visual ghosting.
   - **Alternative considered:** rely on widget redraw alone. Rejected because reduced viewport may not overwrite old rows.

4. **Keep no-match state interactive with minimal stable layout**
   - No-match keeps menu open and editable; render minimal body plus status line instead of collapsing to zero-height list.
   - **Why:** preserves current interaction contract and avoids flicker between "open" and "disappeared" states.
   - **Alternative considered:** auto-close on zero matches. Rejected; contradicts existing no-match interactivity requirements.

5. **Reuse existing layout planner for bordered/borderless and grid/single-column**
   - Dynamic height feeds into existing `build_menu_layout` path so divider/scrollbar/grid semantics remain consistent.
   - **Why:** minimizes behavioral drift across modes and leverages existing tested layout logic.
   - **Alternative considered:** separate shrink-specific rendering path. Rejected due to duplicate logic and divergence risk.

## Risks / Trade-offs

- **[Risk] Extra per-frame layout recomputation may increase render overhead** → **Mitigation:** keep computations linear in visible candidates and retain current candidate-limit cap.
- **[Risk] Shrink transitions can leave stale glyphs in borderless scrollbar column** → **Mitigation:** clear vacated rows and ensure reserved scrollbar column is cleared when not needed.
- **[Risk] Height oscillation around threshold can feel jittery during rapid typing** → **Mitigation:** enforce deterministic row calculation and minimal no-match floor to avoid collapse jitter.
- **[Risk] Terminal edge cases differ across shell integrations (e.g., PSReadLine mode)** → **Mitigation:** preserve existing TTY backend split and validate shrink behavior in both standard and PSReadLine-compatible paths.
