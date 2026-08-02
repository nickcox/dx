# Navigation Guide

`dx` augments normal shell navigation rather than replacing it. Generated shell
hooks call the native `cd` or `Set-Location` command after `dx` resolves a path,
and record successful transitions in a per-shell session stack.

## Path Resolution

Continue using `cd` as usual:

```text
cd /absolute/path
cd ./relative/path
cd ../sibling
cd ~/code
```

Direct paths take precedence. If the path does not resolve literally, `dx`
tries its shortening rules and configured search roots.

### Segment prefixes

A plain fragment matches the start of a directory segment:

```text
cd pr/dx
```

For example, this can resolve to `projects/dx` when the match is unambiguous.

### Word delimiters

The characters `.`, `_`, and `-` identify literal boundaries inside a segment:

```text
cd cd-e
cd .sdk
```

These can match names containing the same delimiters or word boundaries.

### In-segment gaps

A doubled period inside a segment represents a gap:

```text
cd P..Shell
cd S..32
```

These can match names such as `PowerShell` and `System32`. Matching is
case-sensitive by default; with `DX_CASE_SENSITIVE=false`, lowercase queries
such as `p..shell` work too. A token made only of periods keeps its
ancestor-navigation meaning, so `...` is not treated as an in-segment query.

### Ambiguous results

Normal resolution fails instead of guessing when multiple paths match. To
inspect candidates directly, use:

```bash
dx resolve <query> --list
```

## Search Roots

Search roots let abbreviations resolve from stable locations even when they are
not beneath the current directory.

```toml
search_roots = ["/home/me/code", "/home/me/work"]
```

Configured roots are considered in order, and the current working directory is
also included unless it duplicates a configured root. Matches across all roots
still must be unambiguous. See
[Configuration Reference](./configuration.md).

## Ancestor Navigation

`up` lists ancestors nearest-first and changes to the selected one:

```text
up             # nearest ancestor
up 3           # third ancestor
up project     # closest matching ancestor
```

PowerShell installs `..` as an alias for `up`. In POSIX shells, use `up` as the
command; `..` remains a path passed to `cd`.

## Back and Forward

Every successful navigation updates the current shell session's history.

```text
back
forward
```

Equivalent aliases are:

```text
cd-
cd+
```

Selectors work with stack navigation too:

```text
back 2
back project
forward 1
```

Stack transitions consume history up to the selected destination. They do not
add a new navigation entry while traversing.

## Selector Rules

Commands that accept selectors use the same rules:

- No selector chooses the first candidate.
- A positive integer chooses the Nth candidate, starting at 1.
- Text chooses the closest path match.

Text matching prefers exact paths and basenames, then prefixes, then substring
matches. Existing candidate order breaks ties.

## Recent Directories

`cdr` jumps to directories recorded in the current shell session:

```text
cdr
cdr project
```

The current directory is omitted from candidate results.

## Frecent Directories

`z` and `cdf` query zoxide and jump to its first matching result:

```text
z project
cdf project
```

`dx` does not maintain a separate frecency database. Install zoxide and use it
normally to populate its history. If zoxide is unavailable, frecent completion
returns no candidates.

## Bookmarks

Save the current directory under a name:

```bash
dx bookmarks add work
```

Save a specific directory:

```bash
dx bookmarks add docs /path/to/documentation
```

Both `add` and `remove` print the absolute path they acted on. Paths are
canonicalized when saved, so the echoed path is where the bookmark really
points — which matters when the directory you named was a symlink.

List or remove bookmarks:

```bash
dx bookmarks
dx bookmarks --json
dx bookmarks remove work
```

`--json` emits an array of objects:

```json
[{"name": "work", "path": "/home/me/code/acme", "exists": true}]
```

### Resolution and completion

The two behave differently, on purpose:

- **Resolution** requires an exact bookmark name. `cd work` reaches the `work`
  bookmark only when higher-priority direct and abbreviation matches do not
  resolve first. `cd wo` does not, so a partial name can never quietly land you
  somewhere you did not ask for.
- **Completion** matches by name prefix. Typing `cd wo` and pressing the
  completion key offers the `work` bookmark's target, listed after any
  filesystem candidates.

Bookmarks whose target no longer exists are excluded from completion and never
resolve.

### Stale bookmarks

A bookmark whose directory has been deleted or unmounted is marked in the
listing:

```text
work = /home/me/code/acme
old  = /mnt/archive/project (missing)
```

Remove all of them with:

```bash
dx bookmarks prune
```

`prune` prints each bookmark it removed and does nothing when every target is
present. It is never automatic — a missing target is often just an unmounted
volume rather than a bookmark you want to lose.

## Inspect or Clear Session History

Generated hooks manage the stack automatically, but maintenance commands are
available:

```bash
dx stack --list
dx stack --list --direction undo
dx stack --list --direction redo --json
dx stack --clear
dx stack --clear --direction redo
```

## Related Guides

- [Quickstart](./quickstart.md)
- [Shell Setup](./shell-setup.md)
- [Interactive Menu](./menu.md)
- [Configuration Reference](./configuration.md)
