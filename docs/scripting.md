# Scripting

`dx` is built to be called by scripts as well as by shell hooks. Every command
puts its answer on stdout and its diagnostics on stderr, and never both.

This page is the reference for that surface. For everyday interactive use, start
with the [Navigation Guide](./navigation.md).

## Exit Codes

| Code | Meaning |
|---|---|
| `0` | The command produced its result. For `dx resolve`, that means **exactly one** directory resolved. |
| `1` | The command failed. A diagnostic is on stderr, unless the failure is already on stdout in a machine-readable mode. |
| `2` | Usage error — unknown subcommand, missing argument, bad flag value. Emitted by the argument parser before `dx` runs. |

`--list` and `--json` choose how an outcome is presented. They never change
whether the command succeeded. An ambiguous query has not resolved to one
directory, so it exits non-zero in every mode.

To tell ambiguity from a hard failure without parsing anything:

- **Ambiguity** writes to stdout and leaves stderr empty.
- **A hard failure** writes to stderr and leaves stdout empty.

## Output Streams

stdout is the machine channel and stderr is for humans. A command never writes
to both.

Plain output is one absolute path per line, newline-terminated. An empty result
produces no bytes at all, not a blank line. JSON output is one document
terminated by exactly one newline; an empty completion result is `[]`.

## dx resolve

Turns one query into one directory.

```bash
dx resolve pr/dx           # /home/me/projects/dx
```

### --list

Prints every candidate, one per line, when a query is ambiguous. This is the
flag for inspecting *why* a query did not resolve:

```bash
dx resolve pro/al --list
```

A miss has nothing to list, so it still reports on stderr.

### --json

Prints one object. All four keys are always present:

```json
{"status":"ok","reason":null,"path":"/home/me/projects/dx","candidates":null}
{"status":"error","reason":"ambiguous","path":null,"candidates":["/a/x","/b/x"]}
{"status":"error","reason":"not_found","path":null,"candidates":null}
```

| Field | Type |
|---|---|
| `status` | `"ok"` or `"error"` |
| `reason` | `null`, `"ambiguous"`, or `"not_found"` |
| `path` | absolute path string, or `null` |
| `candidates` | array of path strings, or `null` |

Only ambiguity and not-found have a JSON representation. Any other failure — an
unreadable directory, an unsupported drive-relative query — produces empty
stdout and a stderr diagnostic even under `--json`.

The full matrix:

| Query | Mode | Exit | stdout | stderr |
|---|---|---|---|---|
| resolves | any | `0` | the path | empty |
| ambiguous | plain | `1` | empty | diagnostic |
| ambiguous | `--list` | `1` | candidates | empty |
| ambiguous | `--json` | `1` | JSON | empty |
| not found | plain | `1` | empty | diagnostic |
| not found | `--list` | `1` | empty | diagnostic |
| not found | `--json` | `1` | JSON | empty |

## dx complete

Collects every candidate a query could mean, instead of failing on ambiguity.
Always exits `0`, including when there are no candidates.

### Modes

```bash
dx complete paths <query>
dx complete ancestors [query]
dx complete frecents [query]
dx complete recents [query] [--session ID]
dx complete stack --direction back|forward [query] [--session ID]
dx complete filesystem path|directory|file [query]
```

### --json

An array of objects, `rank` counting from 1:

```json
[{"path":"/home/me/projects","label":"me/projects","rank":1}]
```

`label` is a shortened display form — the last one or two path components. It is
for showing to a person; use `path` for anything else.

### --limit, and the --list alias

`--limit N` caps the number of candidates.

`--list` is a historical alias for `--limit` on completion subcommands, and it
**takes a value**:

```bash
dx complete paths --list 20 project    # same as --limit 20
```

This is not the same flag as `dx resolve --list` or `dx stack --list`, which are
both booleans. Prefer `--limit` in new scripts. The alias is kept for
compatibility and is not offered by shell completion.

## dx navigate

Resolves a selector against a candidate list and prints one absolute path. The
`up`, `back` and `forward` commands installed by `dx init` are thin wrappers
around it, so a script can call it directly without loading the hooks.

```bash
dx navigate up [selector]
dx navigate back [selector] [--session ID]
dx navigate forward [selector] [--session ID]
```

The selector grammar is shared with every command that takes one:

- Omitted — the first candidate.
- A positive integer — the Nth candidate, counting from 1.
- Anything else — the closest match, ranked exact path, then exact basename,
  then path prefix, then basename prefix, then substring. Existing candidate
  order breaks ties.

Prints one path and exits `0`, or writes a diagnostic and exits `1` when there
are no candidates, the index is out of range, or nothing matches.

## dx stack

Inspects and edits the current session's directory history.

```bash
dx stack --list [--direction undo|redo|both] [--json]
dx stack --clear [--direction undo|redo|both]
dx stack push <path>
dx stack undo [--preview]
dx stack redo [--preview]
```

`--list --json` emits the same array shape as `dx complete`, byte for byte.
`--preview` prints where the move would land without performing it.

Two things worth knowing:

- **`--direction` takes different words here than on `dx complete stack`.**
  `dx stack` uses `undo`, `redo` and `both`; `dx complete stack` uses `back` and
  `forward`. `undo` corresponds to `back`.
- **`dx stack --list` reports the raw stack, while `dx complete stack` collapses
  repeat visits**, keeping each directory at its most recent position. That is
  why `back 3` means three *places* back rather than three entries.

## dx bookmarks

```bash
dx bookmarks [list] [--json]
dx bookmarks add <name> [path] [--json]
dx bookmarks remove <name> [--json]
dx bookmarks prune [--json]
```

Operations over many bookmarks emit an array; single-bookmark operations emit
one object with the same keys:

```json
[{"name":"work","path":"/home/me/code/acme","exists":true}]
{"name":"work","path":"/home/me/code/acme","exists":true}
```

`exists` reports whether the target is still a directory. On `remove` it tells
you whether you dropped a live bookmark or one that had already gone stale.

`--json` is accepted on either side of the subcommand. Without it, `add` and
`remove` print the bare absolute path, which is what a shell wants to capture.

## Sessions

Back, forward and recent directories are scoped to a session id, taken from
`--session` or the `DX_SESSION` environment variable. Generated hooks set it to
the shell's process id.

The two families differ when no session is set, deliberately:

- `dx stack` fails with a diagnostic, because you asked about a specific
  session's history.
- `dx complete recents` and `dx complete stack` return nothing, because a
  completion that errors would break the shell binding it feeds.

## Stability

**Stable.** The plain and `--json` output of `dx resolve`, `dx complete`,
`dx navigate`, `dx stack` and `dx bookmarks`, along with the exit-code table and
the stdout/stderr split above.

**Internal — do not depend on.** The `dx menu` action protocol: `dx menu` always
writes a JSON object describing how the shell should edit its command line, and
its shape is coupled to the generated hooks that parse it. It changes whenever
those hooks do. Likewise the contents of `dx init` output, and hook-managed
variables such as `DX_RESOLVE_GUARD`.

## Worked Examples

Resolve with a fallback, using only the exit code:

```bash
target=$(dx resolve "$1" 2>/dev/null) || target=$HOME
cd "$target" || exit 1
```

Take the first completion candidate:

```bash
dx complete paths "$1" --json --limit 1 | jq -r '.[0].path // empty'
```

Tell ambiguity from a hard failure without `jq`:

```bash
if out=$(dx resolve "$1" --list 2>/dev/null); then
  cd "$out"
elif [ -n "$out" ]; then
  printf 'ambiguous, candidates were:\n%s\n' "$out" >&2
else
  printf 'no such directory: %s\n' "$1" >&2
fi
```

## Related Guides

- [Navigation Guide](./navigation.md)
- [Configuration Reference](./configuration.md)
- [Interactive Menu](./menu.md)
- [Troubleshooting](./troubleshooting.md)
