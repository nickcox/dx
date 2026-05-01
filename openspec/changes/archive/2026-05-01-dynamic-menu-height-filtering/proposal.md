## Why

`dx menu` currently reserves and renders a mostly fixed menu height based on initial candidates. During interactive filtering, the candidate list can become much smaller, leaving unnecessary empty rows and making the UI feel less responsive. We need dynamic height reduction that preserves interactivity and terminal safety across shells and rendering modes.

## What Changes

- Add dynamic menu height behavior so the visible menu body shrinks as filtered candidates narrow, while preserving a stable status row and interaction loop.
- Keep interactive behavior intact in completion contexts (stdout captured, `/dev/tty` input/output), including cancel/select semantics and no-match interactivity.
- Define anti-glitch rendering expectations so resizing does not leave stale lines, overlap prompt content, or cause border artifacts.
- Specify sensible no-match behavior with a minimal interactive layout that remains open and editable.
- Require equivalent dynamic-height behavior for both bordered and borderless rendering modes.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `dx-menu`: Add requirements for dynamic interactive viewport resizing, terminal-safe redraw/cleanup during size changes, and visual-stability expectations.
- `dx-menu-filtering`: Add requirements for height reduction tied to filtered candidate count and for no-match layout behavior during active filtering.
- `dx-menu-multicolumn`: Update layout requirements so dynamic height reduction remains deterministic and visually correct in bordered and borderless multicolumn rendering.

## Impact

- Affected code: `src/menu/tui.rs` (layout, redraw, cleanup, resizing), and potentially `src/cli/menu.rs` for config/limits integration.
- Affected tests: menu TUI unit tests and integration tests covering filtering, no-match behavior, and bordered vs borderless rendering.
- No protocol/API changes expected for shell hooks (`replace`/`noop` JSON remains unchanged).
