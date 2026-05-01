## 1. Seed-Clamped Filter State

- [x] 1.1 Refactor menu TUI filter state to track immutable initial query plus mutable typed refinement rather than one freely editable filter string.
- [x] 1.2 Update Backspace handling so it removes only typed refinement characters and becomes a no-op at the initial-query boundary.
- [x] 1.3 Ensure the effective query passed to candidate re-querying is always `initial_query + typed_refinement`.

## 2. Exit and Interaction Semantics

- [x] 2.1 Update cancel/changed-query logic so net-zero edits relative to the initial query return `noop`.
- [x] 2.2 Preserve existing selection and replacement behavior when the final refined query differs from the initial query.
- [x] 2.3 Verify empty-initial-query flows still allow normal type/backspace interaction back to the empty seed.

## 3. Verification and Documentation

- [x] 3.1 Add or update menu TUI tests for boundary Backspace behavior, net-zero cancel behavior, and empty-seed filtering.
- [x] 3.2 Add or update CLI/menu tests to cover the revised filtering semantics where needed, including regression coverage that the menu remains interactive on open and during no-match states.
- [x] 3.3 Update any docs or inline comments that describe menu filtering as arbitrary query editing rather than refinement from the initial query.
