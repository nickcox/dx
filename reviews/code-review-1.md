# Code Review — dx Rust Codebase
**Reviewer:** 1 (independent)  
**Date:** 2026-04-08  
**Scope:** All Rust source under `src/` (~8 600 LOC)  
**Verdict:** **mixed** — The architecture is clean and the module boundaries are sensible. Tests are thorough and the safety rules around environment mutation are followed. However, the codebase carries a significant duplication tax: the same ~10-line helpers are copy-pasted into 13 test modules each, two independent storage backends share an identical atomic-write pattern, and several logic paths are unnecessarily re-implemented at both the resolution and completion layers. None of the issues are correctness bugs, but they create real maintenance risk and some findings have correctness-adjacent consequences.

---

## High Severity

### H1 — Segment-traversal algorithm duplicated between `abbreviation` and `roots`

**Files:** `src/resolve/abbreviation.rs:20-58`, `src/resolve/roots.rs:62-92`

**Evidence:**  
`resolve_abbreviation` in `abbreviation.rs` (inner loop, ~30 lines) and `resolve_segment_path` in `roots.rs` (inner loop, ~25 lines) are structurally identical: both walk a list of `current` base directories, `read_dir` each one, filter by `matches_prefix`, and accumulate into `next`. The only semantic difference is the outer iteration: `abbreviation.rs` iterates over roots and collects into `matches`, while `roots.rs` starts from a single root.

```
// abbreviation.rs lines 25-56 — segment walk
let mut current = vec![root.clone()];
for segment in &segments {
    let mut next = Vec::new();
    for base in &current {
        let Ok(entries) = fs::read_dir(base) else { continue; };
        for entry in entries.flatten() { … if matches_prefix(…) { next.push(path); } }
    }
    current = next; …
}

// roots.rs lines 63-91 — identical inner body
let mut current = vec![root.clone()];
for segment in segments {
    let mut next = Vec::new();
    for base in &current { … same body … }
    current = next; …
}
```

**Why it matters:** Any change to the traversal logic (e.g. adding symlink handling, depth limits, or a different `matches_prefix` call site) must be made in two places. This has already produced a subtle discrepancy: `abbreviation.rs` collects from *all* roots before returning, while `roots.rs` processes one root at a time via a `for root in roots` loop in `resolve_fallbacks`, so root-dedup logic is distributed differently. A single change to one side will silently diverge.

**Recommended refactor:**  
Extract a free function in a shared location (e.g. `resolve::traversal` or a new `resolve::walk`):

```rust
/// Walk `roots`, descending through each segment as a prefix-matching
/// wildcard. Returns all matching leaf directories.
pub fn walk_segments(
    roots: &[PathBuf],
    segments: &[&str],
    case_sensitive: bool,
) -> Vec<PathBuf> {
    let mut current: Vec<PathBuf> = roots.to_vec();
    for segment in segments {
        let mut next = Vec::new();
        for base in &current {
            let Ok(entries) = fs::read_dir(base) else { continue };
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() { continue }
                if let Some(name) = entry.file_name().to_str() {
                    if matches_prefix(name, segment, case_sensitive) {
                        next.push(path);
                    }
                }
            }
        }
        current = next;
        if current.is_empty() { break }
    }
    current
}
```

Both `resolve_abbreviation` and `resolve_segment_path` become one-liners delegating to `walk_segments`.

---

### H2 — `apply_completion_limit` and `apply_limit_with_has_more` are the same function in two places

**Files:** `src/resolve/mod.rs:442-452`, `src/menu/mod.rs:93-106`

**Evidence:**
```rust
// resolve/mod.rs
fn apply_completion_limit(mut paths: Vec<PathBuf>, limit: Option<usize>) -> CompletionCandidates {
    let mut has_more = false;
    if let Some(max) = limit && paths.len() > max {
        paths.truncate(max); has_more = true;
    }
    CompletionCandidates { paths, has_more }
}

// menu/mod.rs
fn apply_limit_with_has_more(mut paths: Vec<PathBuf>, limit: Option<usize>) -> CompletionCandidates {
    let mut has_more = false;
    if let Some(max) = limit && paths.len() > max {
        paths.truncate(max); has_more = true;
    }
    CompletionCandidates { paths, has_more }
}
```

They are byte-for-byte identical except for function name.

**Why it matters:** `CompletionCandidates` is defined in `resolve`, so the helper logically belongs there. `menu` importing from `resolve` is already the case — there is no circular dependency concern. The duplication means future changes (e.g. off-by-one fix, metric tracking) must be applied twice.

**Recommended refactor:**  
Make `apply_completion_limit` `pub(crate)` in `resolve/mod.rs` and delete the copy from `menu/mod.rs`.

---

### H3 — `is_valid_name` / `is_valid_session_id` are byte-for-byte identical

**Files:** `src/bookmarks/mod.rs:105-111`, `src/stacks/storage.rs:179-185`

**Evidence:** Both functions implement the same rule: non-empty, ASCII alphanumeric + `-` + `_`. The bodies are identical including the pattern. The same rule is referenced in docs/shell scripts as the allowed character set for session IDs and bookmark names.

**Why it matters:** If the allowed charset changes (e.g. supporting `.` in session IDs), only one copy will be updated.  Any divergence would be a correctness bug (e.g. `dx stack` accepts a name that `dx bookmarks` rejects or vice-versa for a shared concept).

**Recommended refactor:**  
Move to `src/` root or to a small `crate::util` module:

```rust
/// Returns true if `s` is a valid dx identifier: non-empty ASCII alphanumeric, `-`, `_`.
pub fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.as_bytes().iter().all(|b| b.is_ascii_alphanumeric() || *b == b'-' || *b == b'_')
}
```

---

### H4 — Atomic write-with-rename pattern duplicated in both storage modules

**Files:** `src/stacks/storage.rs:92-108`, `src/bookmarks/storage.rs:119-136`

**Evidence:** The 17-line "rename-or-retry-then-cleanup-temp" block is copy-pasted identically into both `write_session` and `write_store`. Only the error variant names differ (`ReplaceSession` vs `ReplaceStore`).

**Why it matters:** This is a concurrency correctness pattern. Any bug fix (e.g. the retry logic has a TOCTOU window between `target.exists()` and `remove_file`) must be applied in two places. The pattern is already under test, but the tests only cover one copy.

**Recommended refactor:**  
The two error types use different variant names but carry the same fields. Options:

1. **Simplest:** Extract an `fs_util::atomic_write(temp: &Path, target: &Path) -> io::Result<()>` free function that each storage module calls, mapping the `io::Error` to its own variant at the call site.

2. **Cleaner:** If the error type signature can be unified, a generic `atomic_rename<E>(temp, target, mk_err: impl Fn(...) -> E) -> Result<(), E>`.

---

## Medium Severity

### M1 — Three `match mode { … }` arms in `resolve` produce the same `Err(ResolveError::Ambiguous{…})`

**File:** `src/resolve/mod.rs:196-209`

**Evidence:**
```rust
match mode {
    ResolveMode::List => Err(ResolveError::Ambiguous { count: candidates.len(), candidates }),
    ResolveMode::Json => Err(ResolveError::Ambiguous { count: candidates.len(), candidates }),
    ResolveMode::Default => Err(ResolveError::Ambiguous { count: candidates.len(), candidates }),
}
```
All three arms are identical. The `mode` is then re-inspected in `emit_error` to decide formatting. The comment-free `prepare_candidates(&mut candidates, None)` call above signals intent to always pass back all candidates regardless of mode.

**Why it matters:** Misleads readers into believing the arms differ. The separate branching in `emit_error` is the correct place for mode-specific behaviour; this `match` adds noise and obscures intent.

**Recommended refactor:**
```rust
return Err(ResolveError::Ambiguous { count: candidates.len(), candidates });
```
Remove the `match mode` entirely.

---

### M2 — `collect_completion_candidates` is a thin wrapper on a wrapper; the API surface is confusing

**File:** `src/resolve/mod.rs:212-229`

**Evidence:**
```rust
pub fn collect_completion_candidates(&self, raw_query: &str) -> Vec<PathBuf> {
    self.collect_completion_candidates_with_meta(raw_query).paths   // wrapper
}
pub fn collect_completion_candidates_with_meta(&self, raw_query: &str) -> CompletionCandidates {
    self.collect_completion_candidates_impl(raw_query, None)        // wrapper
}
pub fn collect_completion_candidates_with_limit(&self, raw_query: &str, limit: Option<usize>) -> CompletionCandidates {
    self.collect_completion_candidates_impl(raw_query, limit)       // wrapper
}
```

The "plain" method is used only in `complete/paths.rs` (which then adds its own `HashSet` dedup on top — see M3). The `_with_meta` method is called only from within the class. The `_with_limit` method is called from `menu/mod.rs`. Three public names for one private function, with a redundant dedup layer on one consumer.

**Why it matters:** API surface bloat makes the correct call non-obvious to future contributors. The `paths` field extraction from `_with_meta` in the "plain" variant silently drops `has_more`.

**Recommended refactor:**  
Consolidate to two public methods matching the two semantically distinct use cases:
- `collect_completion_candidates_with_limit(raw_query, limit: Option<usize>) -> CompletionCandidates` (the real impl)
- `collect_completion_candidates(raw_query) -> Vec<PathBuf>` (convenience, calls with `None`)

Remove `_with_meta` entirely (it is `_with_limit(q, None)` which is the same as the convenience form). Also remove the redundant `HashSet`-dedup in `complete/paths.rs::complete` — `collect_completion_candidates_impl` already calls `push_unique` internally.

---

### M3 — `complete/paths.rs::complete` re-deduplicates a collection that is already deduplicated

**File:** `src/complete/paths.rs:6-17`

**Evidence:**
```rust
pub fn complete(resolver: &Resolver, query: &str) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    for path in resolver.collect_completion_candidates(query) {
        let key = path.display().to_string();
        if seen.insert(key) { output.push(path); }
    }
    output
}
```
`collect_completion_candidates` already calls `push_unique` (the same `HashSet<String>` pattern) internally. The outer dedup is a no-op, but a reader must trace through two levels to verify that.

**Recommended refactor:**  
```rust
pub fn complete(resolver: &Resolver, query: &str) -> Vec<PathBuf> {
    resolver.collect_completion_candidates(query)
}
```
Or eliminate `complete/paths.rs::complete` entirely and call `resolver.collect_completion_candidates` directly at the single call site in `cli/complete.rs`.

---

### M4 — `resolve_session_id` (stacks) and `resolve_session` (complete) are near-identical

**Files:** `src/cli/stacks.rs:315-328`, `src/cli/complete.rs:168-180`

**Evidence:** Same logic — check CLI arg first, fall back to `DX_SESSION` env var. The only differences are: `stacks` returns `Result<String, i32>` and emits an error message for the missing-session case; `complete` returns `Option<String>` and stays silent. The shared `cli` module is the right home for a single function.

**Why it matters:** If the session-resolution logic changes (e.g. supporting a different env var, trimming differently), only one copy will be updated.

**Recommended refactor:**  
Move `resolve_session` to `cli/mod.rs` (or a new `cli/session.rs`) as the canonical form returning `Option<String>`. `cli/stacks.rs` can wrap it with the `Option::ok_or_else` error message pattern:

```rust
fn require_session(cli_session: Option<&str>) -> Result<String, i32> {
    cli::resolve_session(cli_session).ok_or_else(|| {
        eprintln!("dx stack: missing session id (use --session or DX_SESSION)");
        1
    })
}
```

---

### M5 — `ensure_absolute` takes `&PathBuf` instead of `&Path`

**File:** `src/stacks/mod.rs:88`

```rust
fn ensure_absolute(path: &PathBuf) -> Result<(), StackError> { … }
```

**Why it matters:** `&PathBuf` is an anti-pattern in Rust — it prevents callers from passing `&Path` or `path.as_path()` and forces an unnecessary indirection. The `std::path::Path` docs (and Clippy's `clippy::ptr_arg` lint) recommend `&Path` as the idiomatic type for read-only path arguments. This function is called 6 times internally but the type leaks conceptually.

**Recommended refactor:**
```rust
fn ensure_absolute(path: &Path) -> Result<(), StackError> { … }
```
All six call sites use `&path` / `&next` / `&current` which already coerce correctly.

---

### M6 — `truncate_for_cell` in `tui.rs` reverses a char iterator twice

**File:** `src/menu/tui.rs:578-588`

**Evidence:**
```rust
let tail: String = input
    .chars()
    .rev()
    .take(tail_len)
    .collect::<Vec<_>>()   // reverse collect
    .into_iter()
    .rev()                  // reverse again
    .collect();
```
This reverses twice to get the last `tail_len` chars. The `collect::<Vec<_>>()` intermediate allocation is avoidable.

**Why it matters:** The intermediate allocation is unnecessary and the double-reverse is confusing. For a function called per-render-frame with potentially hundreds of cells, this is a minor but real allocation hotspot.

**Recommended refactor:**
```rust
let char_count = input.chars().count(); // already checked above
let start = char_count - tail_len;
let tail: String = input.chars().skip(start).collect();
format!("…{tail}")
```

---

### M7 — Shell hook `cd()` function is copy-pasted between bash and zsh (with one shell-syntax delta)

**Files:** `src/hooks/bash.rs:103-166`, `src/hooks/zsh.rs:103-166`

**Evidence:** The `cd()` function body, `__dx_push_pwd`, `__dx_cd_native`, `__dx_nav_wrapper`, `__dx_stack_wrapper`, `__dx_jump_mode`, `__dx_complete_first`, `__dx_is_path_like`, all navigation aliases (`up`, `back`, `forward`, `cd-`, `cd+`, `cdf`, `z`, `cdr`), and the base session-init block are ~150 lines copy-pasted between `bash.rs` and `zsh.rs`. The only differences are:

- Bash uses `command -v dx >/dev/null 2>&1` for availability checks; Zsh uses `(( $+commands[dx] ))`.
- Zsh uses `local -a __dx_flags` instead of `local __dx_flags=()`.
- Completion functions differ (`COMPREPLY` vs `compadd`, `compdef` vs `complete`).

**Why it matters:** Any fix to shell logic (e.g. the `cd -` handling, flag parsing loop, `__dx_stack_wrapper` retry logic) must be applied to both files. This has already produced the subtle `local __dx_flags=()` vs `local -a __dx_flags` difference — not a bug now, but a maintenance landmine.

**Recommended refactor:**  
Extract the shared logic into a Rust-level `fn common_posix_body(check_dx: &str) -> String` that emits the shell-agnostic function bodies, parameterised by the availability-check snippet. Each shell module then assembles: `common_posix_body(check) + shell_specific_completions()`.

---

## Low Severity

### L1 — `make_temp_dir` copy-pasted into 13 test modules; `env_lock` wrapper into 9

**Files:** Every module with `#[cfg(test)]` blocks.

`make_temp_dir` exists in 13 separate test modules with identical bodies (SystemTime nonce + `create_dir_all`). `env_lock() -> MutexGuard<'static,()>` wrapping `test_support::env_lock()` is re-declared in 9 test modules.

The canonical `env_lock` is already in `test_support`. `make_temp_dir` could also live there.

**Recommended refactor:**  
Add to `src/test_support.rs`:
```rust
pub fn make_temp_dir(label: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock").as_nanos();
    let path = std::env::temp_dir()
        .join(format!("dx-{label}-{nonce}-{}", std::process::id()));
    std::fs::create_dir_all(&path).expect("create temp dir");
    path
}
```
Remove all 13 local copies; replace local `env_lock()` wrappers with direct calls to `test_support::env_lock()`.

---

### L2 — `query_is_empty` check in `cli/menu.rs` is over-complicated

**File:** `src/cli/menu.rs:161`

```rust
let query_is_empty = parsed.query.is_none() || parsed.query.as_deref() == Some("");
```
`parsed.query` is set to `None` when `query_text.is_empty()` (buffer.rs:159), so `parsed.query == Some("")` can never occur — `Some("")` is never constructed. The second condition is dead code.

**Recommended refactor:**
```rust
let query_is_empty = parsed.query.is_none();
```

---

### L3 — `resolve/roots.rs::resolve_fallbacks` checks `!has_slash` twice redundantly

**File:** `src/resolve/roots.rs:7-37`

```rust
let has_slash = query.contains('/');
…
if !has_slash { let direct = root.join(query); if direct.is_dir() { … } }
if has_slash  { matches.extend(resolve_segment_path(…)); }
else          { matches.extend(resolve_single_segment(…)); }
```

The "direct exact join" branch (`if !has_slash { direct … }`) produces a result that `resolve_single_segment` would also produce (since it prefix-matches every child, and an exact name is a prefix match). When `query` exactly equals the directory name, a duplicate is emitted before the `sort; dedup` at the end. More importantly, the two `if !has_slash` / `if has_slash` / `else` blocks can be unified:

**Recommended refactor:**
```rust
for root in roots {
    if !root.is_dir() { continue; }
    if has_slash {
        matches.extend(walk_segments(&[root.clone()], &segments, case_sensitive));
    } else {
        // exact-match shortcut
        let direct = root.join(query);
        if direct.is_dir() { matches.push(direct); continue; }
        matches.extend(resolve_single_segment(root, query, case_sensitive));
    }
}
```
(After H1 is resolved the `walk_segments` call replaces both `resolve_segment_path` and `resolve_abbreviation`'s inner body.)

---

### L4 — `Resolver::bookmark_lookup` field uses a raw fn pointer instead of a boxed closure

**File:** `src/resolve/mod.rs:58`

```rust
bookmark_lookup: fn(&str) -> Option<PathBuf>,
```

A raw fn pointer is fine for the current usage (only `bookmarks::lookup` and `|_| None` are passed). However, any future caller that wants to capture state (e.g. a mock in more complex tests, or an injected store) would need to refactor the API. The idiomatic Rust alternative is `Box<dyn Fn(&str) -> Option<PathBuf>>` or a generic parameter `B: Fn(&str) -> Option<PathBuf>`.

**Why it matters:** Low risk now, but noted because `Resolver::with_bookmark_lookup` is a test-seam and the current type prevents passing closures that close over test state without a workaround.

---

### L5 — `format_plain` builds a `String` line-by-line instead of using `join`

**File:** `src/complete/mod.rs:107-118`

```rust
let mut output = String::new();
for path in paths {
    output.push_str(&path.display().to_string());
    output.push('\n');
}
```

More idiomatic:
```rust
paths.iter()
    .map(|p| p.display().to_string())
    .collect::<Vec<_>>()
    .join("\n")
    + "\n"
```
Or using a `write!` loop with `BufWriter` to stdout. Minor, but shows up on hot completion paths.

---

### L6 — `ResolveMode::Default` and `ResolveMode::List` produce identical output on success

**File:** `src/resolve/mod.rs:101-108`

```rust
ResolveMode::Default => { println!("{}", result.path.display()); 0 }
ResolveMode::List    => { println!("{}", result.path.display()); 0 }
```
These two arms are identical. If `List` mode is meant to produce a different format when the result is unambiguous, it is silently not doing so. If they are intentionally identical, the arms should be merged: `ResolveMode::Default | ResolveMode::List => …`.

---

## Duplication Hotspot Summary

| Duplicated item | Locations | Lines each |
|---|---|---|
| Segment-traversal inner loop | `abbreviation.rs`, `roots.rs` | ~30 |
| `apply_completion_limit` / `apply_limit_with_has_more` | `resolve/mod.rs`, `menu/mod.rs` | 10 |
| `is_valid_name` / `is_valid_session_id` | `bookmarks/mod.rs`, `stacks/storage.rs` | 6 |
| Atomic rename-with-retry block | `stacks/storage.rs`, `bookmarks/storage.rs` | 17 |
| `make_temp_dir` test helper | 13 test modules | ~8 each |
| `env_lock()` wrapper | 9 test modules | 3 each |
| `resolve_session` / `resolve_session_id` | `cli/complete.rs`, `cli/stacks.rs` | 12 |
| `cd()` + shell utility functions | `hooks/bash.rs`, `hooks/zsh.rs` | ~150 |

---

## Overall Verdict: **mixed**

**Strengths:**
- Module boundaries are coherent and the separation of `resolve`, `complete`, `menu`, `stacks`, and `bookmarks` is sensible.
- Error types use `thiserror` consistently and carry full context.
- The global env lock pattern in `test_support` is used correctly in most (not all) modules.
- `SessionStack` logic (`push/pop/undo/redo/sanitize`) is clean and well-tested.
- The TUI selection loop is well-structured and handles terminal state restoration via `Drop`.

**Weaknesses:**
- The codebase is in an early stage where several abstractions that belong together have been independently implemented in parallel: the segment traversal, the limit helper, the identifier validator, the atomic write, and the session resolver each appear twice. Any change to any of these requires a dual-site update.
- The test helper duplication (13× `make_temp_dir`) is mechanical noise that clutters diffs and will silently diverge if the nonce strategy changes.
- A few public API choices (three wrappers for one impl, `&PathBuf` parameter, raw fn pointer) are minor ergonomic debts but will bite as the codebase grows.

The issues are well-scoped and largely mechanical to fix — none require architectural rethinking.
