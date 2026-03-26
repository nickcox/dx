## Context

dx has a working path resolver (`src/resolve/`) with a strict precedence chain (direct > step-up > abbreviated > fallback roots > failure) and per-session undo/redo stacks (`src/stacks/`). Both features are wired through `clap` subcommands in `src/cli/`. Configuration lives in `src/config/mod.rs` and supports TOML + environment variable layering.

This change adds persistent named bookmarks — a simple name-to-path mapping that lets users save directories under memorable aliases and jump to them later. Bookmarks are shared across all shells and terminal sessions.

## Goals / Non-Goals

**Goals:**

- Persistent named directory aliases that survive reboots and work across all shells
- Human-readable, hand-editable store file (TOML)
- Integration with the existing resolve precedence chain so `dx resolve <bookmark-name>` works
- Same output contract as existing commands (one path to stdout on success, stderr + non-zero on failure)
- Zero new external crate dependencies (reuse existing `serde`, `toml` crates)

**Non-Goals:**

- Bookmark syncing across machines (no cloud/git integration)
- Bookmark metadata (tags, descriptions, timestamps) — just name → path
- Fuzzy matching bookmark names (exact match only; fuzzy belongs in completions/menu)
- Automatic bookmark suggestions or inference

## Decisions

### D1: Store format — TOML flat file

**Choice:** Bookmarks stored as a flat TOML file with `name = "path"` entries under a `[bookmarks]` table.

```toml
[bookmarks]
proj = "/Users/nick/code/project"
docs = "/Users/nick/Documents"
```

**Why:** Bookmarks are a small, simple key-value dataset. TOML is already a project dependency (used by `src/config/`), is human-readable, and trivially hand-editable. SQLite would add complexity for no practical benefit at this scale. Even heavy users are unlikely to exceed a few hundred bookmarks.

**Alternatives considered:**
- SQLite: Overkill for simple key-value data; adds `rusqlite` dependency.
- JSON: Less human-friendly to hand-edit than TOML.
- Plain text (`name=path` lines): No parsing library support, fragile quoting.

### D2: Store location — XDG_DATA_HOME with fallback

**Choice:** Bookmark file at `$XDG_DATA_HOME/dx/bookmarks.toml`. If `XDG_DATA_HOME` is not set, fall back to `~/.local/share/dx/bookmarks.toml`. Allow override via `DX_BOOKMARKS_FILE` environment variable.

**Why:** Follows XDG Base Directory Specification for persistent user data, consistent with how the existing config uses `dirs::config_dir()`. The env var override enables testing and non-standard setups.

**Resolution order:**
1. `DX_BOOKMARKS_FILE` env var (if set)
2. `$XDG_DATA_HOME/dx/bookmarks.toml`
3. `~/.local/share/dx/bookmarks.toml` (or platform equivalent via `dirs::data_dir()`)

**Directory auto-creation:** The parent directory is created on the first `dx mark` call (not on read). If the file does not exist, reads return an empty bookmark set rather than failing.

### D3: Bookmark name validation

**Choice:** Bookmark names must be non-empty, contain only alphanumeric characters plus `-` and `_`, and must not conflict with path-like patterns (no `/`, `.`, `~`).

**Why:** Names are used as CLI arguments and as resolver query tokens. Restricting the character set prevents ambiguity with path syntax and shell metacharacters. The regex `^[a-zA-Z0-9_-]+$` is simple to validate and explain.

**Alternatives considered:**
- Allow any string: Too permissive; names containing `/` would be indistinguishable from paths in the resolver.
- Require lowercase only: Unnecessarily restrictive.

### D4: Resolve integration — New precedence stage after fallback roots

**Choice:** Bookmark lookup is inserted as a new stage between fallback roots and failure in the resolver pipeline. The updated precedence chain:

1. Direct paths (absolute, relative, `~`, `..`)
2. Step-up aliases (multi-dot patterns)
3. Abbreviated segment matching
4. Fallback root matching
5. **Bookmark lookup** ← new
6. Failure

**Why:** Filesystem-based resolution should always take priority — if a real directory matches the query, use it. Bookmarks serve as a named shortcut when no filesystem match exists. Placing bookmarks after all filesystem strategies ensures they don't shadow real directories.

**Bookmark resolution semantics:**
- Exact name match only (no prefix/fuzzy matching)
- The bookmarked path must still exist on disk; if it doesn't, the bookmark match is skipped (treated as stale) and resolution continues to failure
- A single bookmark name maps to exactly one path, so ambiguity is impossible at this stage

**Integration approach:** The `Resolver` struct gains an optional bookmark store reference. The `resolve()` method adds a bookmark lookup call between the existing fallback-roots check and the final `NotFound` error. The bookmark module exposes a `lookup(name) -> Option<PathBuf>` function that the resolver calls.

### D5: Module structure

**Choice:** New `src/bookmarks/` module with two files:

- `src/bookmarks/mod.rs`: `BookmarkStore` struct with `load()`, `save()`, `set()`, `remove()`, `get()`, `list()` methods plus bookmark name validation.
- `src/bookmarks/storage.rs`: File path resolution (`data_dir()` logic), TOML parsing/serialization, atomic write (write-to-temp-then-rename, same pattern as session stacks).

CLI wiring in `src/cli/bookmarks.rs` with three subcommands: `mark`, `unmark`, `bookmarks`.

**Why:** Mirrors the `src/stacks/` module layout. Keeps store logic separate from CLI concerns.

### D6: Atomic writes — write-temp-then-rename

**Choice:** Same atomic write strategy used by session stacks. Write to a `.tmp` sibling file, then rename over the target. This prevents corruption from interrupted writes.

**Why:** Bookmarks are persistent user data — losing them to a truncated write would be worse than losing ephemeral session state. The pattern is proven in `src/stacks/storage.rs` and trivial to reuse.

### D7: Concurrent access — last-writer-wins

**Choice:** No file locking. Each command reads the full file, modifies in memory, and writes atomically. Concurrent `dx mark` calls from different terminals use last-writer-wins semantics.

**Why:** Bookmark mutations are rare (a few times per day at most), so write conflicts are extremely unlikely. Adding file locking would increase complexity for a near-zero probability scenario. The atomic write ensures the file is never corrupted; at worst, one of two simultaneous marks is lost. This matches the pragmatic approach taken by tools like zoxide and fasd.

**Alternatives considered:**
- File locking (`flock`/`fcntl`): Cross-platform complexity (especially Windows) for negligible benefit.
- SQLite: Solves concurrent writes elegantly but is overkill (see D1).

## Risks / Trade-offs

**[Risk] Stale bookmarks pointing to deleted directories** → Mitigation: Bookmark resolution validates that the target path exists before returning it. `dx bookmarks` list output could optionally flag stale entries (deferred to a future enhancement, not in this change). Users can manually `dx unmark` stale entries.

**[Risk] Name collisions with filesystem paths** → Mitigation: Bookmarks are the lowest-priority filesystem strategy (D4). A directory named `proj` in the current directory will always win over a bookmark named `proj`. This is correct — bookmarks are fallbacks.

**[Risk] TOML file grows unwieldy** → Mitigation: Extremely unlikely; even 500 bookmarks in a flat TOML table parse in microseconds. No pruning needed.

**[Trade-off] No fuzzy/prefix matching for bookmark names** → Keeps the resolver simple and deterministic. Fuzzy matching belongs in the completion/menu layer (`dx complete` / `dx menu`), not in `dx resolve`.

**[Trade-off] Last-writer-wins instead of locking** → Accepts the theoretical possibility of a lost concurrent write in exchange for zero complexity. Acceptable given how rarely bookmarks are mutated.

## Open Questions

- Should `dx mark` without a `<path>` argument default to `$PWD`, or require the path explicitly? The proposal says it defaults to current directory; confirming this is the right UX.
- Should `dx bookmarks` show a warning for stale bookmarks (paths that no longer exist)? Useful but adds I/O; could be opt-in via `--check` flag. Deferred for now.
