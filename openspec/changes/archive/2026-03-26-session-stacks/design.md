## Context

`dx` is a compiled Rust CLI for directory navigation. It currently has one implemented capability — `path-resolution` — which translates queries (abbreviations, traversals, step-up aliases) to absolute paths. The binary is structured as a single crate with `src/cli/`, `src/resolve/`, and `src/config/` modules.

This design adds per-session undo/redo stacks so users can retrace multi-step navigation. The parent design (dx-cli D3) already decided on PID-keyed temp JSON files for session state. This document refines that decision into concrete implementation choices.

## Goals / Non-Goals

**Goals:**

- Per-terminal undo/redo of directory changes, beyond shell-native `cd -`
- Zero-latency stack operations (read/write local JSON, no database)
- Session isolation — each terminal's stack is independent, keyed by PID
- Ephemeral lifecycle with cross-platform behavior — rely on temp/runtime dirs and include best-effort stale-file pruning where reboot cleanup is not guaranteed
- Output contract consistent with `dx resolve` — print one absolute path to stdout on success

**Non-Goals:**

- Cross-session directory history (that's frecency — a later change)
- Persistent undo history that survives reboots
- Shell hook integration (that's the `shell-hooks` change — this change only provides the subcommands)
- Stack entry limits or in-stack content pruning (stacks are small and ephemeral)

## Decisions

### D1: Session directory — XDG runtime when available, otherwise `std::env::temp_dir()`

**Choice:** Resolve the session directory with this order:

1. If `XDG_RUNTIME_DIR` is set and non-empty, use `$XDG_RUNTIME_DIR/dx-sessions/`.
2. Otherwise, use `std::env::temp_dir().join("dx-sessions")`.

Session files are `<session-id>.json` under that directory.

**Why:**
- Preserves XDG correctness on Linux for runtime data.
- Uses Rust's cross-platform temp API as the universal fallback, including Windows.
- Avoids hand-rolled env var logic (`TMPDIR`, `TEMP`, `TMP`) while still honoring platform conventions.

**Windows note:** `std::env::temp_dir()` resolves from Windows temp conventions (`TEMP`/`TMP`) and gives an appropriate path under the user profile.

**Alternatives considered:**
- Fixed `/tmp/dx-sessions/`: Unix-only and incorrect on Windows.
- Manual env var chain (`XDG_RUNTIME_DIR` -> `TMPDIR` -> `TEMP` -> `TMP` -> `/tmp`): more branching and platform-specific edge cases than needed.
- `~/.local/state/dx/sessions/`: persistent across reboots, opposite of intended ephemeral behavior.

### D2: JSON file schema — single object with undo/redo arrays

**Choice:** Each session file contains:
```json
{
  "cwd": "/current/directory",
  "undo": ["/previous/dir", "/before/that"],
  "redo": ["/next/dir"]
}
```

`undo` is a stack (last element = most recent). `redo` is a stack that gets cleared on any new `push` (standard undo/redo semantics).

**Why:** Flat, simple, human-readable. Single atomic read/write per operation. No schema versioning needed for ephemeral data.

**Alternatives considered:**
- Single interleaved list with cursor position: harder to reason about, more error-prone on partial writes.
- Binary format (bincode, MessagePack): unnecessary complexity for small ephemeral files.

### D3: Subcommand semantics

**Choice:** Four subcommands with clear, distinct roles:

| Command | Action | Stdout on success |
|---------|--------|-------------------|
| `dx push <path>` | Record `<path>` as current directory, push old cwd onto undo stack, clear redo | The pushed path |
| `dx pop` | Pop the most recent entry from undo stack, make it current | The popped path |
| `dx undo` | Move current to redo stack, restore top of undo stack | The restored path |
| `dx redo` | Move current to undo stack, restore top of redo stack | The restored path |

All require `--session <PID>` (or `DX_SESSION` env var) to identify which session file to operate on.

**`push` vs `pop` vs `undo`/`redo`:**
- `push`/`pop` are explicit stack operations — `pop` destructively removes the entry.
- `undo`/`redo` are non-destructive — entries move between stacks and can be re-traversed.
- Shell hooks will typically call `dx push` on every `cd`, then the user invokes `undo`/`redo` to navigate history.

**Why separate `push` from `undo`:** `push` is called by shell hooks on every directory change. `undo` is called by the user. They have different semantics — `push` clears redo (new navigation branch), `undo` preserves it.

**Alternatives considered:**
- Combined `dx stack push/pop/undo/redo` subcommand group: more typing, no benefit.
- Only `undo`/`redo` without explicit push/pop: loses the ability to directly manipulate the stack.

### D4: Session identification — PID passed explicitly

**Choice:** The shell PID is passed via `--session <PID>` flag or `DX_SESSION` env var. The binary does not attempt to detect the parent shell PID.

**Why:** The binary cannot reliably determine which shell session it belongs to — `getppid()` may return a tmux server, a wrapper script, or other intermediary. The shell hook knows its own `$$` and passes it explicitly.

**Alternatives considered:**
- Auto-detect via `getppid()`: unreliable in tmux, screen, nested shells.
- Use terminal TTY path as key: not portable, collides with terminal reuse.

### D5: Atomic writes — write-rename pattern

**Choice:** Write session state to a temp file in the same directory, then rename over the target. This ensures readers never see a partially-written file.

**Why:** JSON files are small (typically <4KB). A same-directory rename provides crash-safe replacement semantics on supported filesystems and avoids partial-read corruption. No file-locking needed for single-session writers.

**Alternatives considered:**
- Advisory file locking (`flock`): overkill for single-writer-per-PID scenario.
- In-place write with fsync: risks partial reads on crash.

### D6: Module placement — `src/stacks/`

**Choice:** New `src/stacks/` module with:
- `mod.rs` — public API (`SessionStack` struct, `push`/`pop`/`undo`/`redo` methods)
- `storage.rs` — file I/O (session dir resolution, read/write/delete)

CLI dispatch lives in `src/cli/stacks.rs`, following the existing `src/cli/resolve.rs` pattern.

**Why:** Keeps stack logic testable independent of CLI. Mirrors the `src/resolve/` structure.

### D7: Stale-session cleanup — best effort, age-based

**Choice:** On each session read/write, optionally scan sibling `*.json` files in the session directory and remove files whose modification time is older than a conservative TTL (for example, 7 days).

Cleanup is best effort:
- Ignore errors reading metadata or deleting files.
- Never fail the active command due to cleanup issues.
- Only delete files matching the expected session filename pattern.

**Why:** Reboot cleanup is common on Unix temp dirs but not guaranteed everywhere (especially Windows). Age-based pruning keeps temp state bounded without introducing a daemon or persistent index.

**Alternatives considered:**
- No cleanup at all: simple, but temp clutter can accumulate indefinitely on platforms without aggressive temp cleanup.
- Startup-only cleanup: misses long-running shells that never restart hooks.
- Explicit `dx stack gc` command: adds user-facing complexity for low-value maintenance.

## Risks / Trade-offs

**[Risk] PID reuse after shell exit** -> If a shell exits and a new process inherits its PID before stale files are removed, the new process could inherit old stack state. Mitigation: shell hooks should call `dx push` for initial cwd on startup, and stale-file pruning reduces long-lived leftovers.

**[Risk] Concurrent writes from same session id** -> If two processes accidentally share a session id, last-writer-wins can drop one update. Mitigation: shell hooks pass process-specific ids by default; write-rename preserves file integrity even if one update is lost.

**[Risk] Temp directory semantics differ across platforms** -> Windows and some managed environments do not guarantee cleanup on reboot. Mitigation: use cross-platform `temp_dir()` and perform best-effort age-based pruning (D7).

**[Trade-off] No stack size limit** -> Stacks could grow large in long-running sessions. Accepted because entries are small path strings and typical sessions stay modest; revisit only if real telemetry shows growth issues.

**[Trade-off] No `dx history` command yet** -> Users can't list their stack without popping. Accepted for now; a read-only `dx history --session <id>` can be added later without changing the data model.

## Migration Plan

1. Add `src/stacks/` module and JSON model with cross-platform session directory resolution (D1, D2).
2. Implement `push`/`pop`/`undo`/`redo` semantics and CLI wiring with explicit `--session` / `DX_SESSION` input (D3, D4).
3. Add atomic persistence layer and corruption/empty-file recovery behavior (D5).
4. Add best-effort stale-session pruning and platform coverage tests (D7).
5. Integrate with shell-hooks change later (out of scope here).

## Open Questions

- Should stale-file pruning TTL be configurable now, or fixed until usage data exists?
- Should we allow non-PID session ids immediately (for shell multiplexers that want stable logical session names)?
