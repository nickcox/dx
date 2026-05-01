## 1. Dynamic Height Computation and Runtime Wiring

- [x] 1.1 Refactor menu runtime sizing to derive current render height from filtered candidate/layout row count on each interactive loop iteration.
- [x] 1.2 Enforce shrink-only behavior within initially reserved terminal space and keep a minimal interactive no-match height floor.
- [x] 1.3 Preserve existing selection/filter state transitions while recomputing layout metrics for single-column and multicolumn modes.

## 2. Safe Redraw and Cleanup for Shrink Transitions

- [x] 2.1 Track prior rendered height and clear vacated rows whenever the menu shrinks to prevent stale item, border, divider, or scrollbar artifacts.
- [x] 2.2 Ensure bordered and borderless paths both clear trailing rows correctly, including borderless reserved scrollbar/divider regions.
- [x] 2.3 Keep terminal lifecycle guarantees intact for resize-time failures (return noop and restore raw mode/cursor/TTY state).

## 3. No-Match and Interactivity Semantics

- [x] 3.1 Implement/verify minimal no-match interactive layout that remains open for additional typing, backspace, selection attempts, or cancel.
- [x] 3.2 Ensure dynamic height changes do not alter cancel/select JSON action semantics or stdout-only action output contract.
- [x] 3.3 Validate completion-context behavior (captured stdout + TTY interaction) remains interactive while dynamic shrinking occurs.

## 4. Verification Coverage

- [x] 4.1 Add/update unit tests for height recomputation and shrink transitions across candidate-count changes, including zero-match states.
- [x] 4.2 Add/update layout tests covering bordered and borderless shrink behavior with multicolumn and single-column rendering.
- [x] 4.3 Add/update failure-path tests asserting terminal-safe noop fallback during dynamic resize draw errors.
