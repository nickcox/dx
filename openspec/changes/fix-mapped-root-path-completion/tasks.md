## 1. Parent Resolution

- [x] 1.1 Update mapped filesystem parent parsing so `/` resolves to root `/` rather than cwd
- [x] 1.2 Update mapped filesystem parent parsing so `/<prefix>` resolves to parent `/` with `<prefix>` as the leaf prefix
- [x] 1.3 Preserve cwd parent resolution for empty and bare relative mapped queries

## 2. Tests

- [x] 2.1 Add mapped `path` mode tests proving `/` does not include cwd children
- [x] 2.2 Add mapped `path` mode tests proving `/<prefix>` filters root children and does not include cwd children
- [x] 2.3 Add regression tests proving empty and bare relative mapped queries still use cwd

## 3. Verification

- [x] 3.1 Run relevant menu candidate sourcing tests
- [x] 3.2 Run the full test suite
