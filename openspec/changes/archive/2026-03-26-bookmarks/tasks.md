## 1. Module Scaffold & Data Model

- [x] 1.1 Create `src/bookmarks/mod.rs` with `BookmarkStore` struct holding a `BTreeMap<String, PathBuf>` for sorted bookmark entries
- [x] 1.2 Create `src/bookmarks/storage.rs` with bookmark file path resolution (`DX_BOOKMARKS_FILE` > `XDG_DATA_HOME` > `dirs::data_dir()`)
- [x] 1.3 Add `pub mod bookmarks;` to `src/lib.rs`
- [x] 1.4 Define TOML schema struct (`BookmarkFile` with `bookmarks: BTreeMap<String, String>`) and derive `Serialize`/`Deserialize`

## 2. Storage I/O

- [x] 2.1 Implement `read_store()` — load and parse TOML file; return empty `BookmarkStore` if file missing
- [x] 2.2 Implement `write_store()` — serialize to TOML and write atomically (temp file + rename)
- [x] 2.3 Implement auto-create parent directory on first write
- [x] 2.4 Add unit test: missing file returns empty store
- [x] 2.5 Add unit test: round-trip write then read preserves bookmarks
- [x] 2.6 Add unit test: corrupt file returns error (not silent empty)

## 3. Bookmark Name Validation

- [x] 3.1 Implement `validate_name()` — check non-empty and matches `^[a-zA-Z0-9_-]+$`
- [x] 3.2 Add unit test: valid names (`my-project`, `docs_v2`, `A1`) accepted
- [x] 3.3 Add unit test: invalid names (`../hack`, `foo/bar`, `~home`, `has space`, empty) rejected

## 4. BookmarkStore Operations

- [x] 4.1 Implement `set(name, path)` — validate name, canonicalize path, verify path exists, insert into map
- [x] 4.2 Implement `remove(name)` — remove entry, return error if not found
- [x] 4.3 Implement `get(name)` — exact-match lookup returning `Option<PathBuf>`
- [x] 4.4 Implement `list()` — return sorted iterator/vec of `(name, path)` pairs
- [x] 4.5 Add unit test: set bookmark for current directory (path omitted defaults to cwd)
- [x] 4.6 Add unit test: set bookmark with explicit path
- [x] 4.7 Add unit test: overwrite existing bookmark replaces path
- [x] 4.8 Add unit test: set with nonexistent path fails
- [x] 4.9 Add unit test: remove existing bookmark succeeds
- [x] 4.10 Add unit test: remove nonexistent bookmark fails
- [x] 4.11 Add unit test: get returns matching path
- [x] 4.12 Add unit test: get returns None for stale path (target deleted after bookmark created)
- [x] 4.13 Add unit test: list returns alphabetically sorted entries

## 5. CLI Wiring

- [x] 5.1 Create `src/cli/bookmarks.rs` with clap subcommands: `mark <name> [path]`, `unmark <name>`, `bookmarks [--json]`
- [x] 5.2 Register bookmark subcommands in `src/cli/mod.rs`
- [x] 5.3 Implement `mark` handler — load store, validate, set, save, output contract (exit 0 on success, stderr + non-zero on failure)
- [x] 5.4 Implement `unmark` handler — load store, remove, save, output contract
- [x] 5.5 Implement `bookmarks` handler — load store, list in `name = path` format or `--json` object
- [x] 5.6 Add unit test: empty bookmark list produces no stdout output and exits 0

## 6. Resolve Integration

- [x] 6.1 Add `lookup(name) -> Option<PathBuf>` function to bookmarks module that checks name match and verifies path exists on disk
- [x] 6.2 Add optional `BookmarkStore` field or bookmark lookup function to `Resolver` struct
- [x] 6.3 Insert bookmark lookup call in `Resolver::resolve()` between fallback-roots check and `NotFound` error
- [x] 6.4 Add unit test: bookmark resolves when no filesystem match exists
- [x] 6.5 Add unit test: fallback root takes precedence over bookmark with same name
- [x] 6.6 Add unit test: stale bookmark (deleted target) returns no match, resolution fails

## 7. Integration Tests

- [x] 7.1 Create `tests/bookmarks_cli.rs` with CLI round-trip tests
- [x] 7.2 Add test: `dx mark proj` then `dx bookmarks` shows the entry
- [x] 7.3 Add test: `dx mark proj` then `dx unmark proj` then `dx bookmarks` shows empty
- [x] 7.4 Add test: `dx mark proj` then `dx resolve proj` returns bookmarked path
- [x] 7.5 Add test: `dx unmark nonexistent` fails with stderr and non-zero exit
- [x] 7.6 Add test: `dx mark bad/name` fails with name validation error
- [x] 7.7 Add test: `dx bookmarks --json` returns valid JSON object
- [x] 7.8 Add test: `DX_BOOKMARKS_FILE` env var is respected

## 8. Cross-Platform Verification

- [x] 8.1 Ensure macOS path canonicalization (resolve `/var` vs `/private/var`) in test assertions
- [x] 8.2 Verify all new code compiles and tests pass with Rust 1.77
