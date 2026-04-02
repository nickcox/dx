## 1. Protocol and Runtime Foundations

- [ ] 1.1 Define incremental menu action schema (typed replace updates, selection-only updates, final accept/cancel events).
- [ ] 1.2 Implement protocol serialization/deserialization in Rust menu action layer.
- [ ] 1.3 Add protocol version/feature guard for hook compatibility checks.

## 2. Menu Runtime Incremental Emission

- [ ] 2.1 Update `src/menu/tui.rs` to emit incremental replace actions for printable input and Backspace edits.
- [ ] 2.2 Ensure selection navigation keys (arrow, Tab/Shift+Tab, j/k) do not emit buffer-mutating actions.
- [ ] 2.3 Implement final accept/cancel event semantics with typed-query persistence guarantees.

## 3. CLI and Session Plumbing

- [ ] 3.1 Update `dx menu` execution flow to support live action streaming/session handling.
- [ ] 3.2 Preserve existing non-interactive fallback behavior and stdout/stderr contracts.
- [ ] 3.3 Add diagnostics/debug paths for incremental protocol troubleshooting.

## 4. Shell Hook Integration

- [ ] 4.1 Update zsh hook integration to consume incremental actions and apply immediate query-token replacements.
- [ ] 4.2 Update bash hook integration to consume incremental actions and apply immediate query-token replacements.
- [ ] 4.3 Update fish hook integration to consume incremental actions and apply immediate query-token replacements.
- [ ] 4.4 Update PowerShell hook integration to consume incremental actions and apply immediate query-token replacements.
- [ ] 4.5 Ensure all shells fail safely to native completion on protocol mismatch/malformed payload.

## 5. Verification and Documentation

- [ ] 5.1 Add integration tests for per-keypress typed sync behavior.
- [ ] 5.2 Add integration tests proving selection-only navigation does not mutate shell buffers.
- [ ] 5.3 Add integration tests for accept/cancel final outcomes (typed cancel persistence vs unchanged cancel noop).
- [ ] 5.4 Update docs to describe live buffer sync behavior and shell-specific fallback semantics.
- [ ] 5.5 Run full test suite and fix any regressions.
