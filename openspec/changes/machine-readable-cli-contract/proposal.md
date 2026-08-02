## Why

The machine-readable surface is inconsistent in ways that only show up when you try to write it down.

- `dx resolve --list` and `--json` exit **0** on an ambiguous query while the same query in default mode exits non-zero, and `--json` on a *not-found* query already exits 1. So the flags silently change success semantics, and inconsistently. The natural script — `if dx resolve "$q" --json > out; then use out; fi` — takes the success branch on a failure whose payload literally says `"status":"error"`.
- `dx complete --json` emits no trailing newline while `dx stack --list --json` does, despite sharing a formatter. The requirement was silent on termination, which is how the two call sites diverged.
- `dx bookmarks add` and `remove` accept `--json` and silently ignore it, always printing a bare path.

None of this is documented, so none of it is discoverable until it bites.

## What Changes

- Establish one rule: **`dx resolve` exits 0 if and only if the query resolved to exactly one directory.** `--list` and `--json` become presentation flags that no longer affect success. Ambiguity under either flag now exits non-zero with the candidates still on stdout.
- Make the stream split contract rather than incidental, since it is the only way to tell ambiguity from a hard failure without parsing: ambiguity writes stdout and leaves stderr empty; a hard failure writes stderr and leaves stdout empty.
- State that only ambiguity and not-found have a JSON representation. Every other resolver failure produces empty stdout and a stderr diagnostic even under `--json`.
- Terminate `dx complete --json` with exactly one newline, making it byte-identical to `dx stack --list --json` for the same candidates.
- Emit JSON from `dx bookmarks add` and `remove` when `--json` is given: a single object with the same `name`/`path`/`exists` keys the plural operations use. `remove` reports whether the bookmark it dropped was still live.
- Remove the redundant per-subcommand `--json` declarations on `dx bookmarks list` and `prune`; the parent's global flag already covers both positions.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `path-resolution`: Ambiguity in a machine-readable mode exits non-zero; the stdout/stderr split becomes part of the contract.
- `completions`: JSON output is newline-terminated.
- `bookmarks`: `add` and `remove` honor `--json`.

## Impact

- Affects `src/cli/resolve.rs`, `src/cli/error.rs`, `src/cli/complete.rs`, `src/cli/bookmarks.rs`, `src/cli/mod.rs`.
- **Breaking**: `dx resolve --list <ambiguous>` and `dx resolve --json <ambiguous>` now exit 1 instead of 0. Output shape is unchanged.
- **Breaking**: `dx complete <mode> --json` now ends with a newline. Byte-exact comparisons differ; JSON parsers are unaffected.
- **Breaking**: `dx bookmarks add --json` and `remove --json` now emit JSON instead of a bare path.
- **Breaking (Rust API)**: `CliError::ResolveReportedAsJson` renamed to `ResolveReportedOnStdout`, since it now covers `--list` as well.
- No generated shell hook is affected: every hook calls plain `dx resolve` and branches on the exit code, and none passes `--list` or `--json` to it. The only `--json` in any template is a `dx complete` call in the PowerShell native menu, which parses with `ConvertFrom-Json` and tolerates the added newline.
- Hook goldens change only in that `--json` under `bookmarks list`/`prune` now carries the global flag's description text.
- Does not change any JSON field name or shape, the `dx menu` action protocol, completion routes, or resolution precedence.
