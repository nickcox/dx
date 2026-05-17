## 1. Key Mapping

- [x] 1.1 Add a page movement action to the menu key action model
- [x] 1.2 Map PageDown to forward page movement
- [x] 1.3 Map PageUp to backward page movement

## 2. Selection Movement

- [x] 2.1 Implement clamped page movement helper for candidate selection
- [x] 2.2 Use visible row count as page size in single-column layout
- [x] 2.3 Use visible row count multiplied by active column count as page size in multicolumn layout
- [x] 2.4 Ensure page movement has a minimum effective step of one candidate when candidates exist

## 3. Tests

- [x] 3.1 Add key mapping tests for PageUp and PageDown
- [x] 3.2 Add single-column page movement tests for forward, backward, and boundary clamping
- [x] 3.3 Add multicolumn page movement tests for visible-grid-capacity movement and boundary clamping
- [x] 3.4 Verify existing arrow and Tab navigation tests still pass unchanged

## 4. Verification

- [x] 4.1 Run relevant menu TUI tests
- [x] 4.2 Run the full test suite
