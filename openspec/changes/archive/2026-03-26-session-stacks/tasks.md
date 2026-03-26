## 1. Module Scaffold and Data Model

- [x] 1.1 Create `src/stacks/mod.rs` with `SessionStack` struct (cwd, undo vec, redo vec) and `pub mod storage`
- [x] 1.2 Create `src/stacks/storage.rs` with session directory resolution (XDG_RUNTIME_DIR -> temp_dir fallback) and directory auto-creation
- [x] 1.3 Add `pub mod stacks` to `src/lib.rs`
- [x] 1.4 Add `serde` derive on `SessionStack` and confirm JSON round-trip with unit test

## 2. Session File I/O

- [x] 2.1 Implement `storage::read_session(dir, session_id)` — returns `SessionStack` or default-empty on missing/corrupt file
- [x] 2.2 Implement `storage::write_session(dir, session_id, stack)` — write-to-temp-then-rename atomic write pattern
- [x] 2.3 Unit test: read missing file returns empty session
- [x] 2.4 Unit test: read corrupt file returns empty session and subsequent write overwrites it
- [x] 2.5 Unit test: write-rename produces valid JSON readable by read_session

## 3. Stack Operations

- [x] 3.1 Implement `SessionStack::push(path)` — set cwd, push old cwd to undo, clear redo; no-op on duplicate
- [x] 3.2 Implement `SessionStack::pop()` — pop top of undo to cwd, fail if undo empty; redo untouched
- [x] 3.3 Implement `SessionStack::undo()` — move cwd to redo, pop undo to cwd, fail if undo empty
- [x] 3.4 Implement `SessionStack::redo()` — move cwd to undo, pop redo to cwd, fail if redo empty
- [x] 3.5 Unit test: push onto empty session sets cwd, undo/redo empty
- [x] 3.6 Unit test: push with existing history moves old cwd to undo and clears redo
- [x] 3.7 Unit test: push duplicate of current cwd is no-op (redo preserved)
- [x] 3.8 Unit test: pop with entries returns top of undo, redo unchanged
- [x] 3.9 Unit test: pop with empty undo returns error
- [x] 3.10 Unit test: undo moves cwd to redo and restores top of undo
- [x] 3.11 Unit test: consecutive undos traverse full undo stack and build redo
- [x] 3.12 Unit test: undo with empty undo stack returns error
- [x] 3.13 Unit test: redo after undo restores forward position
- [x] 3.14 Unit test: redo with empty redo stack returns error
- [x] 3.15 Unit test: push after undo clears redo (branch semantics)

## 4. CLI Wiring

- [x] 4.1 Create `src/cli/stacks.rs` with clap subcommands: `push <path>`, `pop`, `undo`, `redo`, each accepting `--session <id>`
- [x] 4.2 Wire session ID resolution: `--session` flag takes precedence over `DX_SESSION` env var; error if neither provided
- [x] 4.3 Register stacks subcommands in `src/cli/mod.rs` and `src/main.rs`
- [x] 4.4 Implement output contract: success prints one absolute path + newline to stdout (exit 0); failure prints diagnostic to stderr (exit non-zero)

## 5. Stale Session Cleanup

- [x] 5.1 Implement `storage::cleanup_stale(dir, ttl)` — scan sibling *.json, delete files older than TTL, ignore errors, skip non-session-pattern files
- [x] 5.2 Call cleanup opportunistically from read/write path (best effort, never fail active command)
- [x] 5.3 Unit test: files older than TTL are deleted
- [x] 5.4 Unit test: files newer than TTL are preserved
- [x] 5.5 Unit test: non-session files (e.g. `.lock`, `.tmp`) are not deleted
- [x] 5.6 Unit test: cleanup errors (permission denied) do not propagate

## 6. Integration Tests

- [x] 6.1 CLI integration test: full push/undo/redo/push cycle via binary invocation, verify stdout and session file state
- [x] 6.2 CLI integration test: pop with history and pop with empty stack (exit code + stderr)
- [x] 6.3 CLI integration test: missing session ID produces error
- [x] 6.4 CLI integration test: DX_SESSION env var is respected; --session flag overrides it
- [x] 6.5 CLI integration test: session directory auto-created on first use

## 7. Cross-Platform Verification

- [x] 7.1 Verify session dir resolution uses temp_dir() correctly on macOS (canonicalize /var vs /private/var in assertions)
- [x] 7.2 Verify build and tests pass on current Rust 1.77 toolchain with pinned dependencies
