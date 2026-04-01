## 1. CLI and Menu Core

- [x] 1.1 Add `dx menu` subcommand parsing with required inputs (`--buffer`, `--cursor`, `--cwd`, `--session`) and validation/error output contract.
- [x] 1.2 Implement menu action model (`replace` and `noop`) with JSON serialization and deterministic stdout/stderr exit behavior.
- [x] 1.3 Implement command-buffer parsing that maps supported contexts (`cd`, `up`, `cdf`, `z`, `cdr`, `back`, `forward`, `cd-`, `cd+`) to completion modes and query token ranges.
- [x] 1.4 Integrate candidate sourcing by reusing existing `dx complete` mode pipelines and session-aware stack/recents behavior.

## 2. Interactive Runtime and Fallbacks

- [x] 2.1 Add MSRV-compatible TUI dependencies (`ratatui`/`crossterm`) and implement the interactive selection loop that accepts ranked candidates and returns a user selection or cancellation.
- [x] 2.2 Implement non-interactive fallback to `noop` when TTY is unavailable, without stderr noise.
- [x] 2.3 Implement robust failure handling for interactive initialization/rendering errors (diagnostic + non-zero exit).

## 3. Shell Hook Integration

- [x] 3.1 Add opt-in menu integration toggle wiring in generated hooks for Bash, Zsh, Fish, and PowerShell; when disabled, hooks SHALL NOT invoke `dx menu` and existing completion/wrapper behavior is unchanged.
- [x] 3.2 Add hook-side invocation plumbing scoped to dx navigation commands (`cd`, `up`, `cdf`, `z`, `cdr`, `back`, `forward`, `cd-`, `cd+`) to pass buffer, cursor, cwd, and session context to `dx menu`; non-dx command buffers bypass menu and use native completion.
- [x] 3.3 Implement hook-side JSON action application for `replace`/`noop` with safe fallback to existing completion behavior on parse/command failure or non-zero exit.

## 4. Verification and Documentation

- [x] 4.1 Add unit tests for buffer parsing, mode mapping, replacement range computation, and action serialization.
- [x] 4.2 Add integration tests for non-interactive/noop behavior, selection output contract, and shell hook invocation/action application contracts.
- [x] 4.3 Add regression tests ensuring existing `dx complete` and navigation wrapper semantics remain unchanged when menu is disabled.
- [x] 4.4 Update user-facing docs for enabling/disabling menu integration and fallback behavior across supported shells.


## 5. Completion-Context Hardening

- [x] 5.1 Ensure shell completion invocations for menu use controlling TTY stdin (`</dev/tty`) while preserving stdout JSON capture.
- [x] 5.2 Implement terminal cleanup guard logic that restores raw mode, alternate screen, and mouse capture on all exit paths (Esc, Ctrl+C, read/draw error).
- [x] 5.3 Add regression tests for completion-context interactivity (`cd M<Tab>` path) proving menu stays open until selection/cancel.
- [x] 5.4 Add regression tests for terminal recovery after cancellation/error to prevent escaped control-sequence leakage.
- [x] 5.5 Add opt-in debug instrumentation contract (env-gated) and document how to troubleshoot menu no-op/flash behavior.
