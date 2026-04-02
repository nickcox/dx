## 1. Menu Filter State and Matching

- [x] 1.1 Add `filter_query` state to menu runtime and recompute visible candidates on each input event.
- [x] 1.2 Re-query the completion pipeline (resolver + `source_candidates`) on each filter change instead of in-memory matching, so path-prefix queries (`~/D`, `/Users/nick/D`) and abbreviations work identically to `dx complete`.
- [x] 1.3 Pass a `QueryFn` callback from `cli/menu.rs` into `tui::select` so the TUI can invoke the resolver without circular dependencies.

## 2. Input Handling and Exit Action Semantics

- [x] 2.1 Handle printable character input to append to `filter_query` and re-render immediately.
- [x] 2.2 Handle Backspace to edit `filter_query`, including empty-query reset behavior.
- [x] 2.3 Emit candidate-selection `replace` action on Enter when a filtered selection exists.
- [x] 2.4 Emit typed-filter `replace` action on cancel when query changed; emit `noop` on cancel when unchanged.

## 3. UI Feedback

- [x] 3.1 Update status bar to show active filter text and selected full path.
- [x] 3.2 Add explicit no-match UI state while preserving interactive input handling.
- [x] 3.3 Keep arrow, Tab/Shift+Tab, and j/k navigation operating on filtered candidates.

## 4. Shell Hook and Contract Verification

- [x] 4.1 Verify shell hooks continue to apply only the final menu JSON action.
- [x] 4.2 Add integration tests covering typing + cancel persistence, noop-cancel, and selected replace flow.
- [x] 4.3 Update docs/tests to clarify filter persistence is delivered via final `dx menu` action (including cancel replace path).
