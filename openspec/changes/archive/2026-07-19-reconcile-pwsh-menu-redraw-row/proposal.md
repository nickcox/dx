## Why

Interactive PowerShell menus scroll and clean up the terminal through `/dev/tty`, but PSReadLine redraws from its cached pre-scroll prompt coordinate. This coordinate disagreement can leave blank rows after selection or cancellation even though the menu itself reserved space correctly.

## What Changes

- Preserve the TUI's post-reservation redraw row and scroll amount through interactive menu results.
- Extend dirty menu action payloads with terminal geometry that a shell hook can use to reconcile its prompt position.
- Include dirty terminal geometry for cancellation as well as interactive replacement.
- Make the PowerShell hook redraw with PSReadLine's explicit-Y `InvokePrompt` overload instead of the no-argument stale-coordinate path.
- Preserve current behavior for clean single-candidate replacements, noop/fallback outcomes, and non-PowerShell shell hooks.
- Add focused coverage for selection, cancellation, prompt rows near the terminal bottom, and wrapped or multiline PowerShell input.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dx-menu`: Extend dirty interactive outcomes with post-scroll terminal redraw geometry.
- `shell-hooks`: Use returned terminal geometry to reconcile PowerShell prompt redraw after interactive menu cleanup.

## Impact

- Affects TUI reservation/result propagation in `src/menu/tui.rs`, menu action serialization, and `dx menu` tests.
- Affects the generated PSReadLine handler in `src/hooks/pwsh.rs` and shell-hook tests.
- Changes the private dx-to-shell JSON contract for dirty interactive outcomes while leaving clean replacement and noop behavior unchanged.
- Does not change candidate sourcing, menu layout, terminal scrolling, or Bash/Zsh/Fish redraw behavior.
