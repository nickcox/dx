## Context

`dx menu` currently receives an initial candidate set from completion pipelines and allows navigation/selection, but it does not support incremental narrowing after the menu opens. Users must dismiss, type more in the shell, and reopen completion. The existing architecture separates concerns cleanly: Rust menu runtime owns interaction and emits a final JSON action; shell hooks only apply that final action.

Constraints:
- Completion hooks run in command-substitution contexts where stdout is captured.
- Interactive input/rendering must continue using `/dev/tty`-safe behavior.
- Typed in-menu filter input should persist back to the shell line even if the user cancels without selecting a candidate.

## Goals / Non-Goals

**Goals:**
- Support live in-menu filtering from printable key input.
- Support backspace editing of the active in-menu filter string.
- Keep navigation semantics (arrow, Tab/Shift+Tab, j/k) operating on the filtered list.
- Match `dx complete`-style prefix filtering expectations.
- Persist typed filter edits to shell buffer on menu exit (confirm or cancel).

**Non-Goals:**
- Multi-column layout or configurable truncation widths.
- Fuzzy ranking/scoring changes to resolver/completion providers.
- Per-keystroke shell hook round-trips while menu is open.

## Decisions

### D1: Add a menu-local filter state layered on top of initial candidates
- Maintain `filter_query: String` inside `tui::run_loop`.
- Derive `visible_candidates: Vec<usize>` each frame by case-insensitive prefix match against display labels/tokens.
- Rationale: keeps resolver/completion behavior unchanged and makes filtering deterministic and low risk.
- Alternative considered: request fresh candidates from `dx complete` on every keypress. Rejected because it adds subprocess overhead and shell coupling.

### D2: Persist filter text to shell buffer via final action
- `dx menu` SHALL still emit exactly one final JSON action on exit.
- On Enter with selection: emit existing selection `replace` action.
- On Esc/Ctrl-C with non-empty typed delta: emit a `replace` action that updates only the active token to the typed filter text.
- On Esc/Ctrl-C with no typed delta: emit `noop`.
- Rationale: preserves single-response hook contract while enabling typed refinement persistence.
- Alternative considered: mutating shell buffer on every keypress via bidirectional protocol. Rejected due to shell complexity and fragile TTY interactions.

### D3: Extend key handling for filter editing with safe defaults
- Printable chars append to `filter_query`.
- Backspace removes one char from `filter_query`.
- Navigation keys unchanged and operate over `visible_candidates`.
- Enter with empty visible set either keeps menu open or exits with typed-query replacement per UX tuning, but SHALL NOT crash.
- Rationale: minimal, predictable editing model that matches terminal selector expectations.

### D4: Surface filter context in UI status region
- Status row shows selected full path plus current filter text (e.g. `filter: do`).
- When no match, show explicit `no matches` indicator while keeping filter visible.
- Rationale: user needs clear feedback about what is being filtered and why list may be empty.

### D5: Keep shell hooks as action appliers only
- Hooks continue invoking `dx menu` once and applying the returned JSON action.
- Hooks do not need to understand intermediate filter state.
- Rationale: all policy remains in Rust menu runtime; shell-specific logic stays thin.

## Risks / Trade-offs

- [Cancel now mutates line in some cases] -> Restrict cancel mutation to typed-delta-only replacement of active token; verify with explicit tests.
- [Selection/index drift after filter changes] -> Track selected item via original candidate index and clamp/reset deterministically when visibility changes.
- [No-match UX ambiguity] -> Show explicit no-match status and keep cancel available.
- [Terminal redraw regressions] -> Reuse existing cleanup/cursor restoration paths and extend menu integration tests.

## Migration Plan

1. Implement menu-local filter state and filtered rendering in `src/menu/tui.rs`.
2. Extend menu result representation so cancel can return typed-filter replacement when query changed.
3. Keep shell hooks unchanged structurally; verify they correctly apply returned replace/noop action.
4. Add integration tests in `tests/menu_cli.rs` for filter typing + cancel persistence behavior.
5. Update shell-hook/menu docs to clarify that filter persistence is delivered via final menu action.
6. Rollback strategy: disable typed-filter persistence on cancel and revert to previous noop-cancel behavior while retaining filtering.

## Open Questions

- Should Space append to filter text or remain ignored to avoid accidental broad queries?
- Should Enter with zero visible candidates commit typed filter immediately or stay in menu until explicit cancel?
- Should prefix matching be basename-only or full-token prefix when labels include absolute paths?
