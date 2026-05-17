## 1. Display Style Model

- [x] 1.1 Add an internal query display-style model for filesystem menu labels, covering bare relative, dot-relative, parent-relative, home-relative, and absolute input.
- [x] 1.2 Derive the display style from the current effective query and menu mode, applying it only to filesystem path menu modes.

## 2. Candidate Label Rendering

- [x] 2.1 Update candidate item label rendering to use bare cwd-relative labels for empty and bare relative input.
- [x] 2.2 Preserve explicit `./`, `../`, `~/`, and absolute input styles in item labels.
- [x] 2.3 Keep non-filesystem modes on their existing compact display behavior.
- [x] 2.4 Keep status-row full-path display unchanged.

## 3. Behavior Preservation

- [x] 3.1 Verify candidate sourcing, filtering, ranking, and selected candidate identity are unchanged by label style.
- [x] 3.2 Verify accepted replacement formatting remains unchanged.

## 4. Verification

- [x] 4.1 Add unit coverage for empty, bare relative, dot-relative, parent-relative, home-relative, and absolute query label styles.
- [x] 4.2 Add regression coverage that status-row selected path remains full resolved path while item labels are query-style-aware.
- [x] 4.3 Run the relevant Rust test suite.
