## 1. Bookmark Source Seam

- [x] 1.1 Add a `BookmarkSource` trait in `src/resolve/bookmarks.rs` with exact and prefix lookups, plus a `NoBookmarks` implementation.
- [x] 1.2 Add `StoredBookmarks` in `src/bookmarks/mod.rs`, reading the store lazily and at most once, treating a corrupt store as empty.
- [x] 1.3 Add `BookmarkStore::prefix_matches`, filtering by name prefix and dropping stale targets, honoring case sensitivity.
- [x] 1.4 Replace the resolver's fn-pointer field with `Box<dyn BookmarkSource>` and provide `with_bookmarks` / `without_bookmarks` constructors.
- [x] 1.5 Migrate every `with_bookmark_lookup` call site, including the benchmark and latency test, and remove the free `bookmarks::lookup`.

## 2. Prefix Completion

- [x] 2.1 Match bookmark names by prefix for plain queries in the shared candidate collector, keeping exact-match behavior for explicit filesystem-prefix queries.
- [x] 2.2 Keep bookmark candidates ordered after filesystem candidates.
- [x] 2.3 Leave resolution exact-match-only and gated on an empty candidate set.

## 3. Staleness, Prune and Success Output

- [x] 3.1 Add `BookmarkEntry`, `BookmarkStore::entries`, and `BookmarkStore::prune_stale`, replacing the now-superseded `list`.
- [x] 3.2 Mark stale entries in the human listing with a ` (missing)` suffix.
- [x] 3.3 Change `--json` output to an array of `{name, path, exists}` objects, preserving the existing non-UTF-8 rejection.
- [x] 3.4 Add a `dx bookmarks prune` subcommand that reports each removed bookmark and writes only when something changed.
- [x] 3.5 Print the canonical path from `add` and the removed path from `remove`.

## 4. Verification

- [x] 4.1 Add unit tests for prefix matching, case sensitivity, empty prefix, staleness exclusion, entries, prune, corrupt-store tolerance, and read-once behavior.
- [x] 4.2 Add completion tests for prefix offers, stale exclusion, filesystem-candidate ordering, filesystem-prefix queries, and root-anchored queries.
- [x] 4.3 Add a resolution test proving a bookmark prefix does not resolve while an exact name does.
- [x] 4.4 Add integration tests for add/remove echo, the symlink case, the stale marker, prune, and the no-op prune.
- [x] 4.5 Update the `--json` integration test for the array shape and regenerate hook goldens.

## 5. Documentation

- [x] 5.1 Replace the "bookmark names are not completed yet" statement in shell setup.
- [x] 5.2 Document prune, the stale marker, the JSON shape, success output, and the resolution/completion split in the navigation guide.
- [x] 5.3 Add a bookmarks step to the quickstart and note prefix completion in the README highlights.
