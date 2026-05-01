## Why

The interactive `dx menu` currently lets users backspace past the initial prompt-derived query, which broadens the candidate set and keeps the menu height logically tied to a wider result universe than the one the user started from. Clamping live filtering to append-only refinement from the initial query simplifies the interaction model and avoids needing dynamic menu growth while the user edits.

## What Changes

- Change in-menu filtering so the active filter query cannot become broader than the initial query derived from the shell buffer.
- Restrict Backspace/editing behavior to only remove characters that were typed during the current interactive filtering session; once the menu returns to the initial query, further broadening edits are ignored.
- Update cancel semantics so returning to the initial query counts as no net filter delta and continues to produce `noop` on cancel; only net refinements beyond the initial query produce a replacement on cancel.
- Treat an initially empty query as the natural lower bound, so typing/backspacing still works normally within the session as long as editing does not go past the empty seed.
- Preserve current re-query semantics and selection behavior for typed refinements.
- Keep menu sizing behavior effectively monotonic during interactive filtering because candidate sets may narrow relative to the initial query but not expand beyond it.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `dx-menu-filtering`: update filter editing rules so live filtering is clamped to the initial prompt-derived query and cannot broaden beyond it during the interactive session.
- `dx-menu`: clarify the interactive session contract so typed filtering is treated as refinement of the initial query rather than arbitrary query rewriting.

## Impact

- Affected code: `src/menu/tui.rs` filter input/backspace handling and related menu tests.
- Affected tests/docs: `tests/menu_cli.rs` / menu TUI tests and any documentation describing live filter editing behavior.
- No new dependencies or external integrations are required.
