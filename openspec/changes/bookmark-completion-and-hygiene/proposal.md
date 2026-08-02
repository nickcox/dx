## Why

Bookmarks are write-only from a user's perspective. Storage, validation, canonicalization and atomic writes all work, but nothing surfaces a bookmark back to the user:

- Bookmark names are never completable. Completion injects a bookmark only on an exact name match and emits the bare target path, so with a bookmark `work`, `dx complete paths wo` returns nothing and the name itself never appears anywhere.
- Stale bookmarks fail silently. `BookmarkStore::get` filters targets that no longer exist, but the listing does not, so `dx bookmarks` prints a dead entry while `cd <name>` fails with a generic "unable to resolve query" and no indication why.
- `add` and `remove` print nothing on success, so path canonicalization — which resolves symlinks — stays invisible until it surprises someone.

## What Changes

- Match bookmark names by prefix when collecting completion candidates, so `cd wo` offers the `work` bookmark's target. This flows through the shared candidate collector, so `dx complete paths`, the generated `cd` completion, and the interactive menu all gain it without a new command, completion route, or menu mode.
- Keep resolution exact-match-only, so a partial name can never resolve to a directory the user did not name.
- Exclude stale bookmarks from completion candidates, matching what exact lookup already does.
- Mark stale entries in `dx bookmarks` output and add `dx bookmarks prune` to remove them, reporting each one rather than deleting silently.
- Change `dx bookmarks --json` from a name-to-path object to an array of `{name, path, exists}`, which is consistent with `dx complete --json` and can carry staleness.
- Print the canonical absolute path from `dx bookmarks add` and the removed path from `dx bookmarks remove`.
- Replace the resolver's `fn(&str) -> Option<PathBuf>` bookmark hook with a `BookmarkSource` trait, mirroring `FrecencyProvider`, since a bare fn pointer cannot express prefix enumeration. The production implementation reads the store at most once per invocation instead of re-parsing the TOML on every lookup.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `bookmarks`: Add prefix completion, a prune operation, staleness in the listing, success output for add/remove, and read-once store access that tolerates a corrupt store.
- `completions`: Paths mode matches bookmark names by prefix rather than only exactly.

## Impact

- Affects `src/bookmarks/mod.rs`, `src/resolve/{mod,bookmarks,completion,pipeline}.rs`, and `src/cli/bookmarks.rs`.
- Breaking: `dx bookmarks --json` output shape changes from `{"proj":"/path"}` to `[{"name":"proj","path":"/path","exists":true}]`.
- Breaking: `dx bookmarks add` and `remove` now write a path to stdout where they previously wrote nothing.
- Breaking: `Resolver::with_bookmark_lookup` is replaced by `Resolver::with_bookmarks` and `Resolver::without_bookmarks`, and `Resolver` no longer implements `Clone`.
- `dx complete paths <query>` returns additional rows when a bookmark name starts with the query.
- Hook goldens change only by clap-generated entries for the new `prune` subcommand. Completion routes, menu-eligible commands, and the menu JSON action contract are untouched.
- Does not change `dx resolve` precedence, bookmark storage format, name validation, or atomic write behavior.
