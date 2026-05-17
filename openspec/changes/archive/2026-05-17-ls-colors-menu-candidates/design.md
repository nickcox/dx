## Context

The interactive menu currently builds candidate display labels as plain strings and applies Ratatui styles only for selection highlights, status text, and scrollbar chrome. Candidate identity and shell action output are path-based, so visual styling can be added in the renderer without changing candidate sourcing, filtering, replacement formatting, or the shell hook protocol.

The desired behavior is explicitly opt-in: `DX_MENU_LS_COLORS=1` plus a present `LS_COLORS` value enables candidate color styling. Any other environment state keeps the existing monochrome candidate rendering.

## Goals / Non-Goals

**Goals:**

- Parse `LS_COLORS` only when `DX_MENU_LS_COLORS=1` and `LS_COLORS` is present.
- Apply LS_COLORS-derived styling to non-selected candidate labels in both single-column and multicolumn menu rendering.
- Preserve the existing selected-candidate highlight by letting it override LS_COLORS styling.
- Keep all behavioral outputs unchanged: candidate ordering, filtering, selected path, status row text, JSON action shape, and shell replacement value.
- Document `DX_MENU_LS_COLORS` with the existing menu environment variables.

**Non-Goals:**

- No automatic color enabling from `LS_COLORS` alone.
- No theming system beyond LS_COLORS candidate styling.
- No colorization of the status row, border, scrollbar, or replacement text.
- No shell-hook changes.

## Decisions

### Decision: Make LS_COLORS Styling Explicitly Opt-In

Use `DX_MENU_LS_COLORS=1` as the feature gate and require `LS_COLORS` to be present.

Alternatives considered:
- Auto-enable when `LS_COLORS` exists: rejected because the current menu is monochrome and color is a visible behavior change.
- Use a broader `DX_MENU_COLORS=never|ls` setting: rejected for this change because only LS_COLORS behavior is currently needed.

### Decision: Treat Color as Presentation Only

Build candidate presentation from the existing absolute path plus display label, but do not store color in candidate sourcing or action data.

Alternatives considered:
- Add style metadata to `CompletionCandidates`: rejected because completion providers should remain independent of TUI presentation concerns.
- Emit ANSI escape sequences inside label strings: rejected because it would interfere with width calculations, truncation, tests, and Ratatui's native styling model.

### Decision: Selected Highlight Overrides LS_COLORS

When a candidate is selected, render it with the existing selected style rather than merging LS_COLORS foreground/background with the selection style.

Alternatives considered:
- Merge LS_COLORS foreground with selected background: rejected because many color combinations are unreadable.
- Use different selected colors per file type: rejected as unnecessary complexity.

### Decision: Resolve Styles Per Render Cycle from Paths

For each render cycle, derive candidate label text from query style and derive candidate style from the corresponding `PathBuf`. The label is used for layout; the style is applied only when building Ratatui spans/list items.

Alternatives considered:
- Cache style metadata across filter updates: deferred until there is evidence of performance issues.
- Classify by displayed label: rejected because query-style labels (`foo`, `./foo`, `../foo`) should not affect LS_COLORS matching.

## Risks / Trade-offs

- [Risk] Metadata checks for many candidates may add latency on slow filesystems. -> Mitigation: only perform LS_COLORS classification when explicitly enabled; keep existing monochrome path as the default.
- [Risk] Full LS_COLORS semantics are nuanced, especially symlinks, executable bits, and extended ANSI color forms. -> Mitigation: prefer a focused parser dependency if it reduces custom parsing risk; otherwise define and test the supported subset clearly.
- [Risk] Selected-row readability can regress if styles are merged incorrectly. -> Mitigation: selected candidates always use the existing selected style and ignore LS_COLORS style.
- [Risk] Styled text can break layout if ANSI codes are embedded in strings. -> Mitigation: use Ratatui `Style` and `Span` objects, never ANSI escape sequences in candidate labels.

## Migration Plan

No migration is required. Users get current monochrome behavior by default. To enable candidate colors, set `DX_MENU_LS_COLORS=1` in an environment where `LS_COLORS` is present.
