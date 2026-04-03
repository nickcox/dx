# AGENTS.md — Shared Project Knowledge

## Rust version

This project uses Rost 2024 Edition so crates should be compatible with that. The local complier is `rustc 1.94.1`.

## Gotcha

* **Evaluate `dx init pwsh` output as a single script block**: In PowerShell, piping `dx init pwsh` directly to `Invoke-Expression` can execute line-by-line and break multi-line constructs in the generated hook script. Convert output to one string first, e.g. `Invoke-Expression ((& dx init pwsh | Out-String))` (or join lines with `` `n ``) before evaluation.
* **Use one global env lock across test modules**: Tests that mutate process env vars (for example `DX_BOOKMARKS_FILE`, `XDG_DATA_HOME`) can flake under parallel execution if each module has its own `OnceLock<Mutex<()>>`. Module-local locks do not synchronize with each other. Use a single shared env lock helper (for example in `src/test_support.rs`) and have all env-mutating tests acquire it.
* **Canonicalize paths in macOS CLI tests**: On macOS, equivalent temp paths may appear as `/var/...` vs `/private/var/...`, causing brittle string comparisons in integration tests. Normalize both expected and actual paths with `std::fs::canonicalize` before asserting equality. This avoids false failures in CLI and shell-hook path assertions.
* **openspec strict markdown requirements**: The `openspec` CLI requires exact markdown formatting for parsing. In specs, scenarios MUST use exactly 4 hashtags (`#### Scenario:`); using 3 or bullets fails silently. Every requirement needs at least one scenario. `MODIFIED` requirements must include the entire original requirement block. In `tasks.md`, tasks MUST use exact `- [ ]` checkboxes to be tracked during the `apply` phase.
* **Use apply instructions for real task progress**: For OpenSpec implementation progress, treat `openspec instructions apply --change <name> --json` as the source of truth, not `openspec status`. `status.isComplete` only reflects artifact creation and can be true while apply tasks are still 0/N. Drive work from `state`, `progress`, `tasks`, and `contextFiles`, and mark completion by updating `tasks.md` checkboxes with exact `- [ ]` / `- [x]` syntax.
* **Sync delta specs before archiving new capabilities**: Before archiving a change with delta specs, always assess sync status and present a combined delta summary, then prompt for sync vs skip. If a capability main spec is missing (for example `openspec/specs/<capability>/spec.md` absent), syncing first creates it and prevents losing unsynced capability definitions inside `changes/archive`. Safe default: sync before archive.

## Decision

* **Frecency strategy: zoxide-first, native SQLite deferred**: dx defers building its own frecency store in favor of using zoxide as an external frecency provider. Define a `FrecencyProvider` trait with a `ZoxideProvider` impl that shells out to `zoxide query`. dx owns display, filtering, and selection — zoxide is just a candidate source. Build native SQLite store only if zoxide proves insufficient (scoring mismatch, latency, missing integration). This reversed D2 from the original design doc, which preferred own-store-first. Rationale: frecency is a solved problem, unique dx value is in path resolution, abbreviation expansion, session stacks, and interactive menu.
* **Navigation selector resolution lives in Rust, not shell scripts**: The `dx navigate up|back|forward [selector]` subcommand centralizes selector-to-path resolution in Rust rather than distributing it across per-shell wrapper scripts. Shell wrappers are thin: they call `dx navigate <mode> [selector]`, get back one absolute path, then `builtin cd` to it and `dx push`. Selector semantics: no arg = first candidate, integer = Nth candidate (1-based), non-integer = closest path match. Closest-match tie-break is deterministic: exact path → exact basename → path prefix → basename prefix → substring, with mode-native ordering preserved for ties. This keeps shell hooks trivial and testable from a single Rust test suite.

## Architecture

* **dx starts as single Rust binary crate**: The `dx` CLI is designed as one Cargo binary crate at repo root (`Cargo.toml`, `src/main.rs`) with internal modules for `cli`, `resolve`, and `config`, rather than a multi-crate workspace. This keeps early implementation simple while preserving a clean seam to extract `resolve` into a library crate later if the project expands.
* **dx resolve execution contract**: `dx resolve` translates input to absolute paths for shell hooks. It enforces a strict output contract: success prints exactly one absolute path to stdout; failure prints to stderr with a non-zero exit code. Precedence is strictly ordered (direct paths > step-up aliases > abbreviated segments > fallback roots). Ambiguous matches explicitly fail rather than guessing, unless `--list` or `--json` is specified.
* **dx feature breakdown and implementation sequencing**: Completions use explicit modes, not intent inference. CLI contract: `dx complete paths|ancestors|frecents|recents|stack`. Shell hooks route by command name to matching mode (`cd`→paths, `up`→ancestors, `z/cdf`→frecents, `cdr`→recents, `cd-/cd+`→stack). Plain output is one absolute path per line for ALL modes (including ancestors/stack — no numeric indexes). JSON output includes `path`, `label`, and `rank` only — no `mode` or `direction` fields since caller context already knows these. Navigation wrappers (`up`, `back`, `forward`) accept either: no arg (first candidate), integer (Nth candidate), or non-integer string (closest path match via exact→basename→prefix→substring scoring). Closest-match logic lives in Rust (`dx navigate`), not shell scripts.

## Pattern

* **Archive flow requires explicit change selection**: In `/opsx-continue` and `/opsx-archive`, if no change name is provided, require explicit user selection from active changes; do not infer from context or auto-pick. Use `openspec list --json`, present the most recently modified options with schema/status recency, and let the user choose. Auto-selection is only acceptable for `/opsx-apply` when unambiguous.

<!-- This section is maintained by the coding agent via lore (https://github.com/BYK/opencode-lore) -->
## Long-term Knowledge

### Gotcha

<!-- lore:019d4769-cbfb-7293-9bdc-70a43b93850a -->
* **Use ADDED vs MODIFIED correctly in delta specs**: In OpenSpec delta specs for existing capabilities, use \`## ADDED Requirements\` when introducing new requirements and reserve \`## MODIFIED Requirements\` only for changing existing requirement text. If you use MODIFIED, include the full updated original requirement block (not partial snippets), or archive/sync can lose behavior detail. A capability marked "modified" in proposal can still have an ADDED-only delta if existing requirements are untouched.

### Pattern

<!-- lore:019d3bdf-649a-77f2-92d0-5320c69b3d47 -->
* **Archive flow requires explicit change selection**: OpenSpec flows must require explicit change selection whenever multiple active changes exist. For \`/opsx-apply\`, prompt the user with numbered options and proceed only after a clear choice; auto-select is acceptable only when exactly one change is unambiguous. This keeps apply/archive behavior predictable and avoids acting on the wrong change.

<!-- lore:019d3bdf-6499-73f6-8282-abe598b775a8 -->
* **Canonicalize paths in macOS CLI tests**: On macOS, equivalent temp paths may appear as  vs , causing brittle string comparisons in integration tests. Normalize both expected and actual paths with  before asserting equality. This avoids false failures in CLI and shell-hook path assertions.

<!-- lore:019d3bdf-649a-77f2-92d0-531fd4f02b60 -->
* **dx feature breakdown and implementation sequencing**: Completions use explicit modes, not intent inference. CLI contract: . Shell hooks route by command name to matching mode (→paths, →ancestors, →frecents, →recents, →stack). Plain output is one absolute path per line for ALL modes (including ancestors/stack — no numeric indexes). JSON output includes , , and  only — no  or  fields since caller context already knows these. Navigation wrappers (, , ) accept either: no arg (first candidate), integer (Nth candidate), or non-integer string (closest path match via exact→basename→prefix→substring scoring). Closest-match logic lives in Rust (), not shell scripts.

<!-- lore:019d3bdf-649a-77f2-92d0-531e60627ae6 -->
* **dx resolve execution contract**: translates input to absolute paths for shell hooks. It enforces a strict output contract: success prints exactly one absolute path to stdout; failure prints to stderr with a non-zero exit code. Precedence is strictly ordered (direct paths > step-up aliases > abbreviated segments > fallback roots). Ambiguous matches explicitly fail rather than guessing, unless  or  is specified.

<!-- lore:019d3bdf-649a-77f2-92d0-531d5e0b2864 -->
* **dx starts as single Rust binary crate**: The  CLI is designed as one Cargo binary crate at repo root (, ) with internal modules for , , and , rather than a multi-crate workspace. This keeps early implementation simple while preserving a clean seam to extract  into a library crate later if the project expands.

<!-- lore:019d3bdf-6498-768d-917a-7f19655288af -->
* **Evaluate  output as a single script block**: In PowerShell, piping  directly to  can execute line-by-line and break multi-line constructs in the generated hook script. Convert output to one string first, e.g.  (or join lines with ) before evaluation.

<!-- lore:019d3bdf-6499-73f6-8282-abe70b24f65a -->
* **Frecency strategy: zoxide-first, native SQLite deferred**: dx defers building its own frecency store in favor of using zoxide as an external frecency provider. Define a  trait with a  impl that shells out to . dx owns display, filtering, and selection — zoxide is just a candidate source. Build native SQLite store only if zoxide proves insufficient (scoring mismatch, latency, missing integration). This reversed D2 from the original design doc, which preferred own-store-first. Rationale: frecency is a solved problem, unique dx value is in path resolution, abbreviation expansion, session stacks, and interactive menu.

<!-- lore:019d477f-2757-7f97-9122-2c67e9d248d2 -->
* **Menu integration is opt-in with noop fallback**: Menu completion uses a split I/O contract: hooks invoke \`dx menu\` with stdin (and UI interaction) on \`/dev/tty\` while keeping stdout reserved for machine-readable JSON action parsing. In completion contexts where stdout is captured, this TTY wiring is required for interactivity. The menu must stay open until explicit select/cancel or unrecoverable error, and terminal state (raw mode, alt screen, mouse capture) must be restored on every exit path; \`noop\` is valid for cancel/non-interactive/no-candidate flows, not a successful interactive start with candidates.

<!-- lore:019d3bdf-649a-77f2-92d0-531cdcd4cdcf -->
* **Navigation selector resolution lives in Rust, not shell scripts**: The  subcommand centralizes selector-to-path resolution in Rust rather than distributing it across per-shell wrapper scripts. Shell wrappers are thin: they call , get back one absolute path, then  to it and . Selector semantics: no arg = first candidate, integer = Nth candidate (1-based), non-integer = closest path match. Closest-match tie-break is deterministic: exact path → exact basename → path prefix → basename prefix → substring, with mode-native ordering preserved for ties. This keeps shell hooks trivial and testable from a single Rust test suite.

<!-- lore:019d3bdf-6499-73f6-8282-abe40f0a6481 -->
* Ignore this item

<!-- lore:019d3bdf-6499-73f6-8282-abe6fdae1213 -->
* **Use apply instructions for real task progress**: For OpenSpec implementation progress, treat  as the source of truth, not .  only reflects artifact creation and can be true while apply tasks are still 0/N. Drive work from , , , and , and mark completion by updating  checkboxes with exact  /  syntax.

<!-- lore:019d3bdf-6499-73f6-8282-abe390e72aab -->
* **Use one global env lock across test modules**: Tests that mutate process env vars (for example , ) can flake under parallel execution if each module has its own . Module-local locks do not synchronize with each other. Use a single shared env lock helper (for example in ) and have all env-mutating tests acquire it.
<!-- End lore-managed section -->
