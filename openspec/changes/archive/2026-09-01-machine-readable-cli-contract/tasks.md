## 1. Resolve Exit Semantics

- [x] 1.1 Rename `CliError::ResolveReportedAsJson` to `ResolveReportedOnStdout` so the variant covers `--list` as well as `--json`.
- [x] 1.2 Return the silent error from the `(Json, Ambiguous)` and `(List, Ambiguous)` arms of `emit_error` so ambiguity exits non-zero with an empty stderr.
- [x] 1.3 Replace the `emit_error` doc comment, which asserted the opposite rule, with the exit-code rule and the stdout/stderr discriminator.

## 2. JSON Output Consistency

- [x] 2.1 Terminate `dx complete --json` with a newline so it matches `dx stack --list --json` byte for byte.
- [x] 2.2 Emit a single JSON object from `dx bookmarks add` and `remove` when `--json` is given, reporting `exists` on removal.
- [x] 2.3 Remove the redundant per-subcommand `--json` declarations on `bookmarks list` and `prune`, leaving the parent's global flag.

## 3. Verification

- [x] 3.1 Add a table-driven test covering all nine cells of output mode by resolution outcome, asserting exit code and stream emptiness.
- [x] 3.2 Update the two tests that pinned exit 0 for ambiguity in `--list` and `--json` modes.
- [x] 3.3 Add tests for JSON newline termination, empty-result `[]`, and stack/complete byte parity.
- [x] 3.4 Add tests for the bookmark `add`/`remove` JSON object and the unchanged plain output.
- [x] 3.5 Regenerate hook goldens and confirm the only change is the inherited `--json` description text.
