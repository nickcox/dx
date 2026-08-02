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

### One segment, one level

This is the rule everything else builds on: **each `/`-separated piece of your
query matches exactly one level of directory.** `pr/dx` means "a directory whose
name starts with `pr`, containing one whose name starts with `dx`".

The consequence is worth stating outright, because it is easy to trip over: a
query with no separator only matches directories sitting *directly* inside a
search root. `dx` on its own will not find `projects/dx`, however unique that
name is — you have to say `pr/dx` and account for the level in between.

### A worked example

Given this tree, with `~/code` as a search root:

```text
~/code
├── projects
│   ├── dx
│   └── dx-extras
├── presentations
├── PowerShell
├── System32
├── cd-extras
├── my_module
└── .config
```

| Query | Resolves to | Why |
|---|---|---|
| `pro` | `projects` | prefix of a directory in the root |
| `pre` | `presentations` | prefix; `pro` and `pre` pick different ones |
| `pro/dx-e` | `projects/dx-extras` | one segment per level |
| `p/dx-e` | `projects/dx-extras` | segments can be as short as you like |
| `cd-e` | `cd-extras` | `-` is a boundary: starts `cd`, then `e` after a `-` |
| `my_mod` | `my_module` | `_` works the same way |
| `.con` | `.config` | so does a leading `.` |
| `P..Shell` | `PowerShell` | `..` is a gap: starts `P`, then `Shell` somewhere later |
| `S..32` | `System32` | same, mid-name |
| `dx` | *nothing* | one segment never descends past the root's children |
| `pro/dx` | *ambiguous* | matches both `dx` and `dx-extras` |

### What a segment can contain

A segment with none of `.`, `_` or `-` in it is a plain **prefix**: `pro`
matches `projects`.

Adding any of those characters switches the segment into a more precise mode:

- **`.`, `_` and `-` are boundaries.** They must appear literally in the name,
  and text after one is matched anywhere following it rather than immediately.
  So `cd-e` requires a name starting `cd`, containing `-`, with an `e` after it.
- **`..` is a gap.** Whatever follows may appear anywhere later in the name, with
  no boundary character required — which is what lets `P..Shell` reach into the
  middle of `PowerShell`.
- The first piece is still anchored to the start of the name unless a gap comes
  before it, and a segment made only of operators matches nothing. That is why
  `...` keeps its ancestor meaning instead of being read as a name query.

### Case sensitivity

Matching is case-sensitive by default, so `P..Shell` finds `PowerShell` while
`p..shell` does not. That keeps queries precise in trees holding many
similarly-named directories, at the cost of having to reproduce capitalisation.

Set `DX_CASE_SENSITIVE=false` if you would rather type everything lowercase:

```bash
export DX_CASE_SENSITIVE=false
```

### Ambiguous results

Normal resolution fails instead of guessing when multiple paths match. To
inspect candidates directly, use:

```bash
dx resolve <query> --list
```

`--list` changes how the outcome is presented, not whether it succeeded: an
ambiguous query still exits non-zero, with the candidates on stdout and nothing
on stderr. See [Scripting](./scripting.md#exit-codes).

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

If you run zoxide's own `cd` integration, initialise `dx` after it — see
[Loading Alongside Other `cd` Wrappers](./shell-setup.md#loading-alongside-other-cd-wrappers).

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

[Scripting](./scripting.md#dx-stack) documents the `--json` shape and the two
ways `dx stack` differs from `dx complete stack`.

## Related Guides

- [Quickstart](./quickstart.md)
- [Shell Setup](./shell-setup.md)
- [Interactive Menu](./menu.md)
- [Configuration Reference](./configuration.md)
- [Scripting](./scripting.md)
