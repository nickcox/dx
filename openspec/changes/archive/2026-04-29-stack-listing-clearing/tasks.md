## 1. CLI Surface and Parsing

- [x] 1.1 Replace top-level `push`, `undo`, and `redo` commands with a unified `stack` command namespace
- [x] 1.2 Add `dx stack --list` and `dx stack --clear` flags with `--direction` support (`undo|redo|both`, default `both`)
- [x] 1.3 Add nested `dx stack push`, `dx stack undo`, and `dx stack redo` subcommands and wire dispatch in `run()`

## 2. Session Access and Error Handling Foundations

- [x] 2.1 Reuse existing session resolution and storage read/write pathways for stack list/clear/push/undo/redo
- [x] 2.2 Ensure missing session identity fails consistently across all `dx stack ...` operations

## 3. Stack Inspection Implementation

- [x] 3.1 Implement `dx stack --list` plain output using nearest-first ordering per selected direction
- [x] 3.2 Implement `dx stack --list --direction both` ordering as undo nearest-first then redo nearest-first
- [x] 3.3 Implement `dx stack --list --json` output with `path`, `label`, and `rank` fields
- [x] 3.4 Implement `label` derivation logic for JSON output consistent with existing completion-style labels

## 4. Stack Maintenance Implementation

- [x] 4.1 Implement `dx stack --clear` scoped behavior for undo-only, redo-only, and both
- [x] 4.2 Preserve `cwd` during all clear operations
- [x] 4.3 Ensure clear is idempotent and succeeds when selected stacks are already empty
- [x] 4.4 Implement clear output contract (no stdout on success, stderr diagnostic on failure)

## 5. Tests and Documentation

- [x] 5.1 Add integration tests for `dx stack --list` plain output across undo/redo/both and empty results
- [x] 5.2 Verify `dx stack --list` is read-only by asserting persisted session file is unchanged after listing
- [x] 5.3 Add integration tests for `dx stack --list --json` structure, label derivation, and deterministic ranking
- [x] 5.4 Add integration tests for `dx stack --clear` scope behavior, idempotency, cwd preservation, and atomic-write-path reuse
- [x] 5.5 Add regression tests for `dx stack push|undo|redo` behavior parity with previous stack semantics
- [x] 5.6 Update CLI help, shell hooks, and shell completion dispatch to use namespaced `dx stack` commands only
- [x] 5.7 Run full test suite and strict OpenSpec validation for this change
