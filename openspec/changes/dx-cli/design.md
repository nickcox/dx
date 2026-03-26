## Context

`cd-extras` is a mature PowerShell module for directory navigation — abbreviation expansion, frecency-based jumps, undo/redo stacks, and bookmarks. It works well in PowerShell but is limited to a single shell, suffers from blocking latency on complex operations, and uses fragile concurrency mechanisms (background runspaces + Mutexes over CSV files).

This design describes the architecture for `dx`, a compiled cross-shell replacement binary with thin per-shell hooks. The binary handles all logic and state; it never changes directories itself. Shell hooks intercept navigation commands, delegate to `dx`, perform the actual `cd`, and report back.

Target shells: Bash, Zsh, Fish, PowerShell.

## Goals / Non-Goals

**Goals:**

- Sub-millisecond path resolution and frecency lookups via compiled binary
- Shared frecency database and bookmarks across all shells and sessions
- Per-session undo/redo stacks keyed by PID
- Consistent interactive menu-based completion experience across Bash, Zsh, and Fish
- Native completion integration for PowerShell (leveraging PSReadLine's existing menu UI)
- Auto-cd support for abbreviated paths via `command_not_found_handler` forwarding
- Clean `dx init <shell>` onboarding — single eval line in shell profile
- Tab keybinding for `dx menu` that preserves default completion for non-dx contexts

**Non-Goals:**

- General-purpose fuzzy file finder (not a replacement for `fzf` or `fd`)
- Directory indexing or filesystem watching (frecency is visit-based, not scan-based)
- GUI or browser-based UI
- Windows cmd.exe support (PowerShell only on Windows)
- Replacing zoxide for users who prefer it (interop yes, replacement no)

## Decisions

### D1: Implementation language — Rust

**Choice:** Rust with `clap` for CLI, `rusqlite` for embedded SQLite, `crossterm`/`ratatui` for TUI menu.

**Why over Go:** Rust produces smaller, faster, dependency-free static binaries. `rusqlite` bundles SQLite with no runtime dependency. The `ratatui` ecosystem is mature for terminal UI. Cargo distribution is natural for CLI tools.

**Alternatives considered:**
- Go: Faster compilation, simpler concurrency model, but larger binaries and GC pauses are undesirable for a latency-sensitive shell tool.

### D2: Frecency store — Zoxide first, native SQLite later (revised)

**Choice:** Defer building a native frecency database. Use zoxide as the frecency candidate provider initially via a `FrecencyProvider` trait abstraction. Build a native SQLite backend (`~/.local/share/dx/frecency.db`) only if zoxide proves insufficient.

**Why zoxide-first:**
- Zoxide already solves frecency well and is widely installed.
- Building a custom SQLite store (schema, scoring algorithm, pruning, concurrent access) is significant work with diminishing returns while zoxide works.
- A trait boundary (`FrecencyProvider`) keeps the coupling clean — dx owns display, filtering, and selection; zoxide is just a candidate source.
- This lets dx ship useful features faster by focusing effort on what's unique (path resolution, session stacks, shell hooks, TUI menu).

**When to revisit:**
- Zoxide scoring/query semantics don't match dx UX expectations.
- Shelling out to `zoxide query` adds unacceptable latency.
- Features need tighter integration than zoxide's CLI output allows.
- Users request a standalone dx with no external dependencies.

**Migration path:** `dx import zoxide` command to seed the native store when/if it's built. This was always planned and remains viable.

**Original rationale (preserved for context):**
- Own SQLite with WAL mode would give full control over scoring, concurrent-safe reads/writes, and tighter integration with bookmarks and session stacks.
- Plain JSON/CSV was ruled out to escape file-locking complexity.

### D3: Session stacks — Temp JSON files keyed by PID

**Choice:** Per-session undo/redo stacks stored at `/tmp/dx-sessions/<PID>.json` (or `$XDG_RUNTIME_DIR/dx-sessions/` where available).

**Why:** Session stacks are inherently ephemeral and per-terminal. JSON files in `/tmp` auto-clean on reboot. No database overhead for transient state. PID-keying naturally isolates sessions.

**Alternatives considered:**
- SQLite table for sessions: overkill for ephemeral data, adds write contention to the frecency DB.
- In-memory daemon: adds process management complexity for little gain.

### D4: Completion architecture — Hybrid (native + `dx menu`)

**Choice:** Two-tier completion system:

1. **`dx complete`** — Returns ranked completion candidates as structured output (one per line or JSON). Used by shells that have good native completion UIs.
2. **`dx menu`** — Full interactive TUI selector (built with `ratatui`/`crossterm`). Used by shells where native completion UI is insufficient or inconsistent.

**Shell mapping:**
- **PowerShell**: Native completion UI via `Register-ArgumentCompleter` + `dx complete` as provider. PSReadLine already provides excellent menu-style completion (`MenuComplete`).
- **Bash/Zsh/Fish**: `dx menu` for consistent interactive selection. Bound to Tab in cd-context with fallback to native completion for everything else.

**Why:** PowerShell's PSReadLine `MenuComplete` is already excellent — reimplementing it would be worse. Bash/Zsh/Fish native completion varies widely in UX and capability, so `dx menu` provides a uniform experience.

**Alternatives considered:**
- Shell-native completion everywhere: inconsistent UX, especially Bash readline limitations.
- `dx menu` everywhere including PowerShell: would be worse than PSReadLine's native menu.
- External dependency on `fzf`: adds install requirement, less control over UX.

### D5: Buffer-aware completion contract

**Choice:** Define a shell-agnostic protocol between shell hooks and `dx menu`/`dx complete`:

**Input (shell hook → dx):**
```json
{
  "buffer": "cd pr/po",
  "cursor": 8,
  "cwd": "/Users/nick",
  "sessionId": "12345"
}
```

Passed as CLI args: `dx menu --buffer "cd pr/po" --cursor 8 --cwd /Users/nick --session 12345`

**Output (dx → shell hook):**
```json
{
  "action": "replace",
  "replaceStart": 3,
  "replaceEnd": 8,
  "value": "/Users/nick/projects/powerops"
}
```

The shell hook reads the output and patches the command line buffer: replace characters `[replaceStart, replaceEnd)` with `value`.

**Why:** This contract makes `dx menu` behave identically regardless of shell. The shell hook's only job is passing buffer state in and applying the replacement out. All ranking, filtering, and UI logic lives in `dx`.

**Note — `shell` is intentionally excluded from the input contract.** `dx menu` has no reason to vary its behaviour by shell — buffer parsing, candidate ranking, TUI rendering, and replacement range calculation are all shell-agnostic. The one concern that might seem to require it (path quoting/escaping in the output value) is the shell hook's responsibility: `dx menu` always returns a raw unquoted path, and the hook wraps it appropriately for its own shell before applying it to the buffer.

**Edge cases:**
- Empty buffer (`cd <Tab>`): show frecent/recent directories
- Partial token (`cd pr/po<Tab>`): filter by abbreviation match
- Multiple tokens (`cd -L pr/po<Tab>`): only complete the token at cursor position
- Cancelled selection: output `{"action": "noop"}`, shell hook does nothing

### D6: Tab keybinding strategy — Conditional context-aware binding

**Choice:** Bind Tab to a shell function that:
1. Inspects the current command buffer.
2. If the command is `cd` (or a dx alias like `cdf`, `cdr`, `mark`), invoke `dx menu` with buffer context.
3. Otherwise, call the shell's original/default completion handler.

**Implementation per shell:**
- **Zsh:** Custom ZLE widget bound to `^I` (Tab). Checks `$BUFFER` prefix. Falls back to `expand-or-complete`.
- **Fish:** Custom `fish_user_key_bindings` function. Checks `commandline -poc`. Falls back to `complete`.
- **Bash:** Readline binding via `bind -x`. Checks `$READLINE_LINE` prefix. Falls back to `complete -o default`.

**Phased rollout:**
1. **Phase 1:** Ship `dx complete` with native shell completion bindings (safe, no Tab override).
2. **Phase 2:** Add `dx menu` bound to an alternate key (`Ctrl+Space` or `Ctrl+F`).
3. **Phase 3:** Opt-in Tab takeover for cd-context only (`dx init <shell> --menu-tab` or `DX_MENU_TAB=1`).

**Constraint:** Tab binding must NEVER break completion for non-dx commands. The conditional check must be fast (string prefix match, no subprocess).

**Alternatives considered:**
- Always override Tab: too invasive, will alienate users.
- Never bind Tab: safe but less discoverable, loses the "just press Tab" UX.
- Bind only via explicit config: good default, but opt-in Tab for cd-context is the sweet spot.

### D7: Auto-cd via `command_not_found_handler`

**Choice:** Shell hooks register a `command_not_found_handler` (or equivalent) that forwards unresolved commands to `dx resolve` before reporting an error.

**Flow:**
1. User types `pr/cd` (no `cd` prefix) and presses Enter.
2. Shell cannot find a command named `pr/cd`.
3. Handler calls `dx resolve "pr/cd"`.
4. If resolve succeeds → `builtin cd <result>` + `dx add <result> --session $PID`.
5. If resolve fails → standard "command not found" error (preserve normal shell behavior).

**Guardrails:**
- Only attempt resolve if input looks path-like (contains `/` or `.`). Don't call `dx resolve` for every typo.
- Avoid recursion: if `dx` itself is not found, don't loop.
- Must not add measurable latency to legitimate "command not found" errors.

**Shell support:**
- Bash: `command_not_found_handle` function
- Zsh: `command_not_found_handler` function
- Fish: `fish_command_not_found` function
- PowerShell: `CommandNotFoundAction` event (or custom proxy function)

### D8: CLI output protocol

**Choice:** `dx` commands output plain text to stdout by default (one path per line). Structured JSON output available via `--json` flag for programmatic consumption.

**Why:** Shell hooks need fast, simple parsing. Plain text is trivially consumed by `read` / variable assignment. JSON mode enables richer integrations and debugging.

## Risks / Trade-offs

**[Risk] TUI rendering inconsistency across terminals** → Mitigation: Use `crossterm` for cross-platform terminal abstraction. Test against common terminal emulators (iTerm2, Windows Terminal, Alacritty, kitty, GNOME Terminal). Provide `DX_NO_TUI=1` fallback to plain list output.

**[Risk] Tab binding breaks user's existing completion setup** → Mitigation: Phased rollout (alternate key first, opt-in Tab later). Conditional binding only in cd-context. `DX_MENU_TAB=0` escape hatch. Thorough testing with popular completion frameworks (oh-my-zsh, bash-completion, Fisher).

**[Risk] Bash Readline integration is fragile** → Mitigation: Bash gets the simplest integration first (native completion via `dx complete`). `dx menu` on Bash is Phase 2 with explicit opt-in. Accept that Bash may have a slightly degraded experience vs Zsh/Fish.

**[Risk] SQLite locking under heavy concurrent writes** → Mitigation: WAL mode handles concurrent readers well. Writes are infrequent (one per `cd`). If contention occurs, retry with backoff. Session stacks intentionally kept out of SQLite.

**[Risk] Large frecency database over time** → Mitigation: Automatic pruning — expire entries not visited in N days, cap total entries. Configurable via `dx config`.

**[Trade-off] Own frecency store vs zoxide interop** → We accept the cost of maintaining our own store in exchange for full control over scoring, query semantics, and tighter integration. `dx import zoxide` provides migration path.

**[Trade-off] `dx menu` TUI vs simpler list output** → The TUI adds binary size and complexity (ratatui dependency) but delivers the PowerShell-parity menu experience that is a core goal.

## Feature Breakdown and Sequencing

The full `dx` vision is implemented as a sequence of focused, independently deliverable changes. Each change maps to one capability with its own proposal, design, specs, and tasks. This avoids a monolithic implementation and lets each piece ship and stabilize before the next builds on it.

### Completed Changes

| # | Change | Capability | Status |
|---|--------|-----------|--------|
| 1 | `path-resolution` | Abbreviated path expansion, traversal, step-up aliases, fallback roots, precedence chain | **Done** — archived 2026-03-26 |

### Planned Changes (in order)

| # | Change | Capability | Key deliverables | Dependencies |
|---|--------|-----------|-----------------|--------------|
| 2 | `session-stacks` | Per-session undo/redo | `dx push`/`dx pop`/`dx undo`/`dx redo`, PID-keyed temp JSON files, auto-cleanup | path-resolution (done) |
| 3 | `bookmarks` | Named persistent directory aliases | `dx mark`/`dx unmark`/`dx bookmarks`, TOML or SQLite store | Standalone |
| 4 | `shell-hooks` | Shell integration and onboarding | `dx init <shell>`, cd wrappers, command_not_found forwarding, auto-cd, frecency recording via `dx add` | path-resolution, session-stacks, bookmarks |
| 5 | `completions` | Tab completion candidates | `dx complete` structured output, native shell completion bindings, zoxide as frecency candidate source | shell-hooks |
| 6 | `dx-menu` | Interactive TUI selector | `ratatui`/`crossterm` TUI, buffer protocol (D5), conditional Tab binding (D6) | completions |
| 7 | `frecency-store` | Native frecency database | Own SQLite backend replacing zoxide dependency, `dx import zoxide`, pruning | Only if/when zoxide proves insufficient |

### Sequencing Rationale

- **session-stacks first**: Standalone, simple, immediately useful alongside path-resolution. No external dependencies.
- **bookmarks next**: Also standalone and simple. Gives users persistent named shortcuts before full shell integration.
- **shell-hooks ties it together**: Requires the core features (resolve, stacks, bookmarks) to exist so the hooks can orchestrate them. This is where `dx` becomes a daily-driver.
- **completions after hooks**: Completion candidates need shell integration in place. Zoxide provides frecency candidates until a native store exists (see D2 revision below).
- **dx-menu last among features**: Most complex piece (TUI rendering, buffer protocol, keybinding). Needs completions infrastructure first.
- **frecency-store deferred**: Zoxide already handles frecency well. Building a native store is only justified if zoxide integration proves limiting (scoring mismatch, latency, missing integration points). This may never be needed.

## Migration Plan

_Superseded by the feature breakdown above._ The original phased plan is preserved here for reference:

1. ~~**Phase 1 — Core engine**: Build `dx` binary with `resolve`, `add`, `undo`/`redo`, `frecent`/`recent`, `bookmarks`, SQLite store.~~
2. ~~**Phase 2 — PowerShell hook**: Replace `cd-extras.psm1` with thin wrapper calling `dx`.~~
3. ~~**Phase 3 — Cross-shell hooks**: `dx init bash/zsh/fish` templates.~~
4. ~~**Phase 4 — `dx menu`**: Interactive TUI selector.~~
5. ~~**Phase 5 — Release**: Homebrew formula, Cargo publish, AUR package.~~

## Open Questions

- **Frecency algorithm**: Port the existing cd-extras algorithm exactly, or adopt a well-known one (e.g., Mozilla/Firefox frecency, zoxide's algorithm)? Needs benchmarking.
- **XDG compliance**: Should we strictly follow XDG Base Directory spec (`$XDG_DATA_HOME`, `$XDG_RUNTIME_DIR`) on Linux/macOS, or use simpler fixed paths?
- **Fish auto-cd interaction**: Fish has implicit auto-cd behavior built in. How does this interact with our `command_not_found` hook — do we need to disable Fish's native auto-cd to avoid double-resolution?
- **Nushell**: Should Nushell be a target shell? It's gaining traction and has good completion APIs.
