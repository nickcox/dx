## Why

Single-candidate `dx menu` completions can return a replacement without ever opening or drawing the TUI, but menu-enabled shell hooks currently redraw the prompt after every replacement. That conservative redraw is useful after interactive TUI selection, but it creates visible jank on fast-path completions where the terminal was never disturbed.

## What Changes

- Keep a single replacement action shape while distinguishing whether the terminal is `clean` or `dirty` after `dx menu` returns.
- Keep the existing fast-path single-candidate behavior, but make its output communicate that the terminal is clean and no prompt redraw is required.
- Preserve prompt redraw after interactive menu rendering, cancellation, or any path where the TUI may have left terminal presentation dirty.
- Update shell hooks to condition redraw/repaint behavior on the new action contract while preserving native fallback behavior.
- Treat the action protocol as private to `dx menu` and generated shell hooks; correctness and clarity matter more than compatibility with hypothetical external JSON consumers.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dx-menu`: clarify the menu action contract so replace results report whether terminal presentation is `clean` or `dirty`.
- `shell-hooks`: update menu-enabled shell replacement handling so redraw happens only for replacement results with dirty terminal state.

## Impact

- Affected code: `src/menu/action.rs`, `src/cli/menu.rs`, `src/menu/tui.rs`, and menu blocks in `src/hooks/{zsh,fish,pwsh,bash}.rs`.
- Affected tests: menu JSON contract tests, shell hook generation tests, and shell-specific integration tests for single-candidate replacement behavior.
- Affected docs/specs: `openspec/specs/dx-menu/spec.md` and `openspec/specs/shell-hooks/spec.md`.
- Dependencies: none expected.
