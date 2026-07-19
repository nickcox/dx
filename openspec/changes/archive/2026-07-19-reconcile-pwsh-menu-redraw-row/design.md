## Context

The inline TUI reserves space by writing `ScrollUp` and cursor movement sequences directly to `/dev/tty`. Its `SpaceReservation` retains the shifted command-line row, but that geometry is discarded before the menu result becomes JSON. In PowerShell, PSReadLine launched the child with an internal prompt origin established before this scroll. After an interactive selection or cancellation, the hook currently calls `PSConsoleReadLine::InvokePrompt()` without arguments, causing PSReadLine to erase and redraw from its stale cached `_initialY`.

PSReadLine exposes `InvokePrompt(ConsoleKeyInfo?, object)`. When the second argument is an integer Y coordinate, it positions the cursor there, writes a fresh prompt, resets its internal initial coordinates and prior render state, and renders the current input buffer. This provides a supported reconciliation seam if dx returns enough information to identify and validate the post-scroll location.

## Goals / Non-Goals

**Goals:**

- Preserve post-reservation terminal geometry through interactive selection and cancellation results.
- Re-anchor PSReadLine at the shifted prompt location after dx scrolls the terminal.
- Avoid accumulated blank reservation rows after PowerShell menu selection or cancellation.
- Preserve the PSReadLine buffer and logical cursor across ordinary, wrapped, and multiline input.
- Keep clean fast-path replacements and non-PowerShell hook behavior unchanged.

**Non-Goals:**

- Changing how the TUI scrolls, renders, shrinks, or clears its reserved area.
- Making PSReadLine aware of every cursor movement performed while the menu is open.
- Replacing the inline PowerShell menu with an alternate-screen or above-prompt renderer.
- Exposing terminal geometry as a stable public API beyond the generated dx shell hooks.
- Adding interactive TUI support to the non-Unix implementation.

## Decisions

1. Return both the post-scroll anchor row and applied scroll amount.

`SpaceReservation` will retain `prompt_row` and `scroll_rows`. Interactive `MenuResult` variants will carry a terminal presentation value containing a zero-based viewport `redraw_row` and `scroll_rows`; the single-candidate fast path remains clean with no geometry. Dirty replace and cancel JSON actions will expose top-level `redrawRow` and `scrollRows` integer fields.

The two fields are intentionally redundant. `redrawRow` is the geometry dx actually used for rendering and cleanup. `scrollRows` lets the PowerShell hook translate its pre-menu buffer coordinates without assuming viewport and console-buffer coordinates are identical. Their expected relationship also permits validation before applying a replacement.

Alternative considered: return only `redrawRow`. This is sufficient in terminals whose buffer and viewport origins are identical, but loses the coordinate translation already known to PowerShell. Returning only `scrollRows` would hide the final geometry used by dx and make mismatches harder to detect.

2. Treat geometry as terminal-presentation metadata, not replacement metadata.

Geometry will be available on both interactive selection and cancellation. Clean replacement and noop actions will not require it. Cancellation will therefore gain `terminal=dirty` and geometry instead of relying on an implicit unconditional redraw rule.

Alternative considered: add geometry only to replace actions. That would leave the current cancellation redraw on stale coordinates and preserve one of the two visible failure paths.

3. Capture the PowerShell redraw origin before launching dx.

The generated PSReadLine handler already captures the physical cursor and window origin. It will additionally capture the data needed to determine the row at which a fresh prompt can safely be rendered: console width, cursor column, buffer content and logical cursor, PSReadLine prompt metadata, continuation prompt metadata, and `ExtraPromptLineCount` where available.

For a single physical input line, the current command-line row is the redraw origin. For wrapped input, the handler will account for display-cell width rather than string length. For explicit multiline input, it will account for continuation-prompt rows. Geometry derivation will be isolated in a small PowerShell helper and covered independently so the menu handler remains readable.

Alternative considered: use the current cursor row as the prompt origin in all cases. That is adequate for the common one-line case but can leave stale wrapped or multiline prompt content above the newly rendered line.

4. Reconcile and validate geometry after dx returns.

For dirty outcomes, the hook will validate that `redrawRow` and `scrollRows` are integer, non-negative, within the terminal viewport, and consistent with the pre-menu relative cursor row. It will translate the captured pre-menu redraw origin by `scrollRows` to obtain the console-buffer Y coordinate passed to PSReadLine. Window-origin translation will remain explicit rather than assuming a zero window offset.

A dirty replacement with invalid geometry will be rejected before buffer mutation and routed to native completion fallback. A dirty cancellation with invalid geometry will preserve cancellation semantics and use the safest existing redraw fallback without invoking native completion.

Alternative considered: trust `redrawRow` unconditionally. Invalid or mismatched rows could cause `InvokePrompt` to overwrite unrelated terminal content or throw from PSReadLine's bounds validation.

5. Invoke PSReadLine with an explicit Y coordinate.

After applying a valid dirty replacement, or immediately for valid dirty cancellation, the hook will call:

```powershell
[Microsoft.PowerShell.PSConsoleReadLine]::InvokePrompt($null, [int]$targetY)
```

The explicit-Y path resets PSReadLine's internal prompt coordinates to the post-scroll location. The no-argument path remains available only as a compatibility fallback when explicit geometry cannot safely be used. Terminal testing showed that explicit redraw alone can leave menu glyphs below the reconstructed prompt because PSReadLine does not clear that region in its explicit-Y branch. The hook will therefore move to the reconciled prompt origin and clear from that row to the end of the display immediately before invoking PSReadLine. Content above the prompt remains untouched, while the region owned by the active input line, predictions, and dx menu is rebuilt from a clean state.

Alternative considered: move the physical cursor and continue calling no-argument `InvokePrompt()`. PSReadLine's implementation ignores the current cursor row in that path and returns to its cached `_initialY`, so physical cursor movement alone does not reconcile the model.

6. Preserve compatibility for other shell parsers.

Zsh, Fish, and Bash will continue to branch on the existing `action` and `terminal` fields and ignore additional geometry fields. Their generated-hook contract tests will confirm dirty replace and cancel payloads containing the new fields do not alter behavior. PowerShell will be the only consumer of geometry in this change.

Alternative considered: make every shell validate the geometry. The metadata addresses a PSReadLine-specific ownership problem and adds no value to shells whose editor repaint functions already reconcile their own terminal state.

7. Validate terminal behavior with a focused PTY/manual matrix.

Unit tests will cover geometry propagation, serialization, validation, and redraw-origin calculations. Generated-hook tests will lock the explicit `InvokePrompt` call. Because ordinary subprocess tests cannot exercise an active PSReadLine edit loop reliably, verification will include an interactive matrix covering selection and Escape cancellation near the terminal bottom with a one-line prompt, multiline prompt, wrapped command, explicit multiline command, cursor-in-middle edit, prediction enabled, and at least one terminal multiplexer when available.

If wrapped or multiline geometry cannot be derived safely from supported PSReadLine/RawUI APIs, implementation will pause and update this design rather than introducing private reflection into PSReadLine internals.

## Risks / Trade-offs

- PSReadLine prompt geometry can include ANSI escapes, wide characters, continuation prompts, and wrapping -> Use RawUI display-cell measurement and targeted tests; never use Rust/PowerShell string length as terminal width.
- `redrawRow` is viewport-relative while `InvokePrompt` accepts a console-buffer row -> Translate through captured window coordinates and cross-check with `scrollRows`.
- Explicit-Y `InvokePrompt` may differ across PSReadLine versions -> Detect invocation failure and retain a safe fallback; test against the supported local and CI PSReadLine versions.
- The explicit-Y branch does not erase old prompt or menu content automatically -> Clear from the reconciled prompt row to the end of the display immediately before PSReadLine rebuilds that owned region.
- Additional JSON fields expand the private shell protocol -> Keep them required only for dirty interactive outcomes and verify POSIX parsers ignore them.
- Wrapped or multiline input may expose geometry unavailable through public APIs -> Do not use reflection; pause and revise toward above-prompt or alternate-screen rendering if public geometry proves insufficient.

## Migration Plan

The binary and generated hooks are upgraded together when users re-evaluate `dx init pwsh --menu`. No persisted data changes are required. Reverting the binary and regenerating hooks restores the previous no-argument redraw behavior.

## Open Questions

- Which minimum PSReadLine version should be considered supported for explicit-Y `InvokePrompt`, and should unsupported versions disable geometry reconciliation or menu integration?

## Verification Notes

- The implementation environment uses PowerShell 7.6.0 with PSReadLine 2.4.5, whose public `InvokePrompt` signature accepts the explicit Y argument. Generated hooks retain a caught no-argument fallback for versions or hosts where explicit invocation fails.
- Native interactive testing confirmed that explicit-Y redraw reconciles the prompt position but initially left menu artifacts below the redrawn prompt. Clearing from the reconciled prompt row to the end of the display before `InvokePrompt` removes that stale owned region while preserving content above the prompt.
- The updated clear-and-redraw path was verified for both interactive selection and Escape cancellation in the native terminal and a second terminal or multiplexer; both completed without stale artifacts or extra blank rows.
- The remaining interactive matrix passed with wrapped input, a multiline prompt, cursor-in-middle completion, and PSReadLine predictions enabled; buffer content, logical cursor, prompt rendering, and prediction cleanup remained intact.
- Plain `expect` PTYs and `wezterm record` without a real emulator viewport report invalid zero-sized console geometry and corrupt PSReadLine cursor-query input, so they are not valid substitutes for the remaining manual terminal matrix.
