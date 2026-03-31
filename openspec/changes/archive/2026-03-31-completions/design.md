## Context

dx has path resolution, bookmarks, session stacks, and shell hooks implemented. The next piece is tab completion — surfacing directory candidates interactively. The original cd-extras has distinct completion modes (Ancestors, Frecents, Recents, Stack, Paths) each tied to a specific navigation action. This design carries that model forward with explicit mode-based completion subcommands rather than a single inferred mode.

Current CLI surface: `Resolve`, `Init`, `Mark`, `Unmark`, `Bookmarks`, `Push`, `Pop`, `Undo`, `Redo`. No completion subcommands exist yet. Shell hooks (`dx init`) generate cd wrappers and session recording but no completion bindings.

## Goals / Non-Goals

**Goals:**

- Explicit completion modes matching cd-extras: paths, ancestors, frecents, recents, stack
- `dx complete <mode> [query]` returns candidates to stdout (one absolute path per line, or JSON)
- Shell completion bindings in `dx init` output that dispatch by command context to the correct mode
- `FrecencyProvider` trait with `ZoxideProvider` adapter — no new crate dependencies
- Graceful degradation when zoxide is absent (frecents returns empty, not an error)
- `up`, `back`, and `forward` navigation commands accept either a numeric step or full/partial path selector and move to the closest matching candidate

**Non-Goals:**

- Interactive TUI menu (`dx menu`) — separate future change
- Tab keybinding override — separate future change (Phase 2/3 per dx-cli design D6)
- Native frecency database — deferred per zoxide-first decision
- Introducing mode or direction metadata in completion payloads when the caller already determines mode context

## Decisions

### D1: Mode-based subcommand tree via clap

**Choice:** Model completion modes as clap subcommands under `dx complete`:

```
dx complete paths <query>           # resolution-style candidates
dx complete ancestors [query]       # parent directories from cwd to root
dx complete frecents [query]        # frecency-ranked via provider
dx complete recents [query]         # recent directories from session stack
dx complete stack --direction back|forward [query]  # session undo/redo entries
```

All modes accept an optional `--json` flag for structured output.

**Why:** Clap subcommands give us type-safe argument parsing, help text, and error messages for free. Each mode has distinct required/optional arguments (e.g., `stack` needs `--direction`), which maps cleanly to per-subcommand arg definitions. No runtime string dispatch needed.

**Alternatives considered:**
- Single `dx complete --mode <name> [query]`: works but less ergonomic; mode as a flag feels awkward when it's the primary discriminator.
- Positional mode: `dx complete paths query` — this is what we're doing, just with clap subcommands backing it.

### D2: Per-mode candidate sources

**Choice:** Each mode has a dedicated candidate source. No mode mixes sources from another mode.

| Mode | Source | Filter |
|------|--------|--------|
| `paths` | Resolver's abbreviated/fallback/bookmark chain | Query as prefix/abbreviation filter |
| `ancestors` | Walk from cwd to `/` | Optional query filters by full path or basename |
| `frecents` | `FrecencyProvider::query()` | Provider handles filtering internally |
| `recents` | Session stack undo entries (most recent first) | Optional query filters by path substring |
| `stack` | Session stack undo (back) or redo (forward) entries | Optional query filters by full path or basename |

**Why:** Mixing sources (e.g., returning frecents alongside ancestors) creates confusion about what the user is selecting. The shell hook already knows the navigation intent, so each mode should return exactly the candidates relevant to that intent.

**Note on `paths` mode:** This mode reuses the existing `Resolver` in list/multi-match mode rather than reimplementing path matching. The resolver already handles direct paths, step-up, abbreviations, fallback roots, and bookmarks. For completion, we want all candidates at the abbreviation/fallback/bookmark stages rather than failing on ambiguity — so the mode calls the resolver with a "collect candidates" strategy rather than the normal "first unique match" strategy.

### D3: FrecencyProvider trait

**Choice:** Define a trait for frecency candidate retrieval:

```rust
pub trait FrecencyProvider {
    fn query(&self, filter: &str) -> Vec<PathBuf>;
    fn is_available(&self) -> bool;
}
```

Initial implementation: `ZoxideProvider` that shells out to `zoxide query --list [filter]` via `std::process::Command`. Results are returned as-is (zoxide handles scoring/ranking).

**Why:** The trait boundary isolates dx from zoxide's CLI interface and makes testing trivial (mock provider). When/if a native SQLite store is built, it implements the same trait.

**Graceful degradation:** `ZoxideProvider::is_available()` checks `which zoxide` once (cached). If absent, `query()` returns an empty vec — the completion mode simply has no frecency candidates. No error message, no panic.

**Alternatives considered:**
- Link zoxide as a library crate: zoxide doesn't expose a stable library API; shelling out is the supported interface.
- Parse zoxide's database directly: fragile, undocumented format, and violates zoxide's intended usage.

### D4: Output format contract

**Choice:** All modes output one absolute path per line to stdout by default. This includes `ancestors` and `stack` modes; plain mode does not emit numeric indexes. With `--json`, output a JSON array of candidate objects that includes only fields needed by menu/completion consumers.

Default (plain text):
```
/home/user/projects/dx
/home/user/projects/dotfiles
/home/user/projects/dx-menu
```

JSON mode:
```json
[
  {"path": "/home/user/projects/dx", "label": "projects/dx", "rank": 1},
  {"path": "/home/user/projects/dotfiles", "label": "projects/dotfiles", "rank": 2},
  {"path": "/home/user/projects/dx-menu", "label": "projects/dx-menu", "rank": 3}
]
```

Empty results produce no output (plain) or `[]` (JSON). Exit code is always 0 for completions — empty results are valid, not errors.

**Why:** Path-per-line works in every shell completion system and keeps plain mode meaningful for ancestor/stack workflows (the inserted token is directly actionable). JSON mode exposes menu metadata (`label`, `rank`) without redundant `mode`/`direction` fields; the shell/script already knows mode and direction from the command it invoked. Exit code 0 for empty results avoids false completion errors in shells.

### D5: Shell completion dispatch in `dx init`

**Choice:** `dx init` output includes completion registration blocks that route by the subcommand currently being typed. The generated code inspects the command buffer to determine which `dx complete <mode>` to call.

For direct `dx` completion, route by `$words[2]` (zsh) / `${COMP_WORDS[1]}` (bash):

| Subcommand context | Completion mode |
|---|---|
| `dx resolve ...` | `dx complete paths` |
| `dx undo` / `dx redo` | No completion (no path argument) |
| `dx push` | Filesystem default |
| `dx complete ...` | Complete mode names: `paths`, `ancestors`, `frecents`, `recents`, `stack` |

For shell wrapper functions (generated by `dx init`):

| Wrapper | Completion mode |
|---|---|
| `cd` | `dx complete paths` |
| `up` | `dx complete ancestors` |
| `cdf` / `z` | `dx complete frecents` |
| `cdr` | `dx complete recents` |
| `cd-` | `dx complete stack --direction back` |
| `cd+` | `dx complete stack --direction forward` |

**Why:** The shell hook already knows the navigation intent from the wrapper name. No inference needed — just a static mapping from wrapper to mode.

**Implementation per shell:**
- **Zsh:** One `_dx` completion function with `$service`/`$words[2]` branching, plus `compdef _dx_complete_<mode> <wrapper>` for each wrapper.
- **Bash:** `complete -F _dx_complete dx` with `COMP_WORDS` dispatch, plus per-wrapper `complete -F` bindings.
- **Fish:** `complete -c dx` with subcommand-aware conditions, plus `complete -c cd` / `complete -c up` / etc. for wrappers.
- **PowerShell:** `Register-ArgumentCompleter -CommandName cd,Set-Location` routing to `dx complete paths --json`, plus separate registrations for wrapper aliases.

### D6: Navigation wrapper generation and selector semantics

**Choice:** `dx init` generates shell navigation wrappers for `up`, `back`, `forward`, `cdf`/`z`, `cdr`, `cd-`, `cd+` alongside the existing `cd` wrapper. Each wrapper has its own completion binding and accepts an optional selector argument.

Selector behavior for `up`, `back`, and `forward`:

- No argument: use the first candidate from the corresponding mode (`ancestors` for `up`, `stack --direction back` for `back`, `stack --direction forward` for `forward`).
- Integer argument (`N`): select the Nth candidate (1-based).
- Non-integer argument: treat as full/partial path selector and move to the closest matching candidate.

Closest-match tie-break rules (deterministic):

1. Exact absolute path match
2. Exact basename match
3. Absolute-path prefix match
4. Basename prefix match
5. Substring match

If multiple candidates remain at the same score, preserve the mode's native order (nearest ancestor first; stack top first).


This closest match logic should live in the Rust source code, not the per-shell scripts.
Example (`up` in zsh):
```zsh
up() {
  local selector="$1"
  local target
  target=$(dx navigate up "$selector")
  [[ -n "$target" ]] && builtin cd "$target" && dx push "$target" --session "$DX_SESSION"
}
compdef _dx_complete_up up
```

**Why:** Returning paths from completion only is not enough; users also need command execution semantics that accept the same values typed from completion (full paths, partial paths, or numeric steps). Matching command selectors to completion output keeps the UX consistent.

**Alternatives considered:**
- User-defined aliases only: less discoverable, forces manual setup per shell.
- Numeric-only navigation (`up 2`, `back 3`) with indexed completion output: concise but brittle and less meaningful in plain completion output.

### D7: Query filtering strategy

**Choice:** Optional query arguments filter candidates using path-aware matching. Filtering is case-insensitive.

- `ancestors`: match full path or basename via exact/prefix/substring rules
- `recents` / `stack`: match full path or basename via exact/prefix/substring rules
- `frecents`: query passed directly to zoxide (it handles its own filtering)
- `paths`: query passed to resolver as abbreviation input

**Why:** Path-aware matching lets users type meaningful partial paths (not just first-segment prefixes) and supports navigation selectors and completion with the same behavior.

### D8: Module layout

**Choice:**

```
src/complete/
  mod.rs          # CompletionMode enum, shared output formatting, mode dispatch
  paths.rs        # paths mode (delegates to Resolver)
  ancestors.rs    # ancestors mode (walks cwd to root)
  recents.rs      # recents mode (reads session stack)
  stack.rs        # stack mode (reads undo/redo from session)
src/frecency/
  mod.rs          # FrecencyProvider trait + ZoxideProvider
src/cli/
  complete.rs     # clap subcommand definitions, mode routing
```

**Why:** Mirrors the existing module pattern (`src/resolve/`, `src/stacks/`, `src/bookmarks/`). Each completion mode is isolated for testability. The frecency module is separate from complete because it's a general provider that could be used outside of completion (e.g., future `dx frecent` command).

## Risks / Trade-offs

**[Risk] Zoxide CLI output format changes** — Mitigation: `ZoxideProvider` parses one-path-per-line output from `zoxide query --list`, which has been stable. Pin to known behavior; if format changes, only one adapter needs updating.

**[Risk] Shell completion binding conflicts with user's existing setup** — Mitigation: Generated wrapper completions use dx-prefixed function names (`_dx_complete_up`, `_dx_complete_back`) to minimize namespace collisions. Users who define their own `up` function can simply not eval the dx-generated wrapper.

**[Risk] Latency on frecents completion (subprocess spawn)** — Mitigation: `zoxide query --list` is fast (~2-5ms typical). If it becomes a bottleneck, the `FrecencyProvider` trait allows swapping to a native implementation without changing completion logic.

**[Trade-off] Wrapper functions in `dx init` increase output size** — Each additional wrapper (up, cdf, z, cdr, cd-, cd+) adds ~10-15 lines per shell. Accepted because the convenience of zero-config navigation vocabulary outweighs slightly larger init output.

**[Trade-off] Closest-match selector may be surprising in edge cases** — Mitigation: deterministic scoring rules and stable tie-breakers. Document selection behavior and provide clear stderr diagnostics when no match is found.

## Open Questions

- **Wrapper naming**: Should `dx init` generate `z` as an alias for frecency jump, or only `cdf`? Both? Configurable? Need to avoid conflicting with zoxide's own `z` alias if both are installed.
- **Recents scope**: Should `recents` include only the current session's history, or aggregate across sessions? Current design is per-session (from session stack). Cross-session recents would need a persistent store.
- **Candidate limits**: Should modes cap the number of returned candidates? Shell completion UIs can struggle with very long lists. A sensible default (e.g., 50) with `--limit N` override may be needed.
