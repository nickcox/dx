## Why

`dx menu` currently renders candidate labels in a monochrome style, even when the user's shell environment already defines file-type colors through `LS_COLORS`. Opt-in LS_COLORS styling would make directory, executable, symlink, and extension cues visible inside the menu without changing selection behavior.

## What Changes

- Add an opt-in menu candidate styling mode enabled by `DX_MENU_LS_COLORS=1` when `LS_COLORS` is present.
- Style candidate labels according to each candidate path's LS_COLORS match during interactive rendering.
- Keep current monochrome rendering when `DX_MENU_LS_COLORS` is unset, not `1`, or `LS_COLORS` is absent.
- Preserve the existing selected-item highlight; selected candidates override LS_COLORS-derived styling for readability.
- Preserve candidate ordering, filtering, status text, JSON actions, and shell replacement text.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `dx-menu`: Add opt-in LS_COLORS candidate styling behavior for the interactive menu renderer.

## Impact

- Affected code: `src/cli/menu.rs`, `src/menu/tui.rs`, and menu rendering tests.
- Affected docs: menu configuration documentation for the new `DX_MENU_LS_COLORS` environment variable.
- Possible dependency impact: an LS_COLORS parser may be added if it provides correct conversion to terminal styles with less custom parsing.
- No shell hook protocol changes and no JSON output shape changes.
