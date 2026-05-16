## 1. Status Path Source

- [x] 1.1 Update menu status rendering to derive selected-item context from the selected resolved candidate path rather than the compact display label.
- [x] 1.2 Preserve existing compact labels for candidate list and grid rendering.

## 2. Replacement Behavior

- [x] 2.1 Verify accepted-selection replacement still uses the existing replacement formatter and remains unchanged for relative path-style queries.
- [x] 2.2 Ensure the status display path does not affect selected candidate identity, filtering, ordering, or JSON action shape.

## 3. Verification

- [x] 3.1 Add unit coverage showing status text receives/displays a full resolved path even when the candidate label is compact.
- [x] 3.2 Add regression coverage that Enter replacement behavior remains unchanged when status displays a full path.
- [x] 3.3 Run the relevant Rust test suite.
