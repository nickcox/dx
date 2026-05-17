## Why

`dx menu` item labels should reflect the path style the user is typing. Today cwd-local candidates are displayed with a `./` prefix even for bare `cd <tab>` or `cd s<tab>` input, which makes the menu feel less like a continuation of the command line.

## What Changes

- Render menu item labels according to the active query style for filesystem path modes.
- Use bare cwd-relative labels for empty or bare relative path input, such as `cd <tab>` and `cd s<tab>`.
- Preserve explicit `./`, `../`, `~/`, and absolute input styles in menu item labels.
- Keep the full-path status-row behavior unchanged.
- Keep Enter/selection replacement behavior unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dx-menu`: Defines query-style-aware candidate item labels for filesystem menu modes.

## Impact

- Affected code: `src/menu/tui.rs` candidate label rendering and related tests.
- Affected behavior: visual-only menu item labels for filesystem path modes.
- Unaffected behavior: candidate sourcing, ranking, filtering, selected candidate identity, status-row full path display, JSON action schema, and Enter replacement formatting.
- Dependencies: none.
