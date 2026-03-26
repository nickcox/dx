## Why

`dx resolve` can translate queries to absolute paths, but there is no way to navigate backward or forward through a session's directory history. Shells provide `cd -` for the last directory only. A per-session undo/redo stack gives users the ability to retrace multi-step navigation — a core part of the cd-extras experience that `dx` needs before shell hooks can orchestrate it.

## What Changes

- **New subcommands**: `dx push <path>`, `dx pop`, `dx undo`, `dx redo` — all operating on a per-session stack identified by shell PID.
- **Session state files**: JSON files in an OS runtime/temp location, using `$XDG_RUNTIME_DIR/dx-sessions/<PID>.json` when available on Unix and otherwise `std::env::temp_dir()/dx-sessions/<PID>.json` (Windows uses `TEMP`/`TMP` via `temp_dir`).
- **Best-effort cleanup**: Session files are ephemeral runtime data. On platforms where reboot cleanup is not guaranteed, `dx` opportunistically prunes stale session files.
- **Output contract**: Each command prints the target directory path to stdout on success (for shell hooks to consume via `builtin cd`), or exits non-zero with a diagnostic on stderr (e.g., "nothing to undo").

## Capabilities

### New Capabilities

- `session-stacks`: Per-session (PID-keyed) undo/redo directory stacks with push/pop/undo/redo operations, temp-file storage, and automatic cleanup.

### Modified Capabilities

_(none)_

## Impact

- **New source modules**: `src/stacks/` module for stack logic and session file I/O.
- **CLI additions**: Four new subcommands added to the existing clap-based CLI (`push`, `pop`, `undo`, `redo`).
- **Filesystem**: Creates and manages files under the platform runtime/temp directory (XDG runtime dir on Linux when set, otherwise `std::env::temp_dir()`, including Windows). No new persistent state or database dependencies.
- **Dependencies**: `serde` and `serde_json` (already present) for JSON serialization. No new crate dependencies expected.
