## 1. Module Scaffolding

- [x] 1.1 Create `src/frecency/mod.rs` with `FrecencyProvider` trait (`query(&self, filter: &str) -> Vec<PathBuf>`, `is_available(&self) -> bool`)
- [x] 1.2 Create `src/complete/mod.rs` with `Candidate` struct (`path: PathBuf`, `label: String`, `rank: usize`), `CompletionMode` enum, and shared output formatting functions (plain and JSON)
- [x] 1.3 Create stub files `src/complete/ancestors.rs`, `src/complete/paths.rs`, `src/complete/recents.rs`, `src/complete/stack.rs`
- [x] 1.4 Create `src/complete/filter.rs` with shared `filter_candidates()` function signature
- [x] 1.5 Create `src/cli/complete.rs` with clap subcommand skeleton for `dx complete`
- [x] 1.6 Wire `frecency` and `complete` modules into `src/lib.rs`
- [x] 1.7 Add `Complete` variant to `Commands` enum in `src/cli/mod.rs` and register in dispatch

## 2. FrecencyProvider and ZoxideProvider

- [x] 2.1 Implement `ZoxideProvider` struct with cached `is_available()` (checks for `zoxide` binary on PATH via `which`/`std::process::Command`)
- [x] 2.2 Implement `ZoxideProvider::query()` — execute `zoxide query --list [filter]`, parse one-path-per-line stdout output
- [x] 2.3 Implement graceful degradation: `query()` returns empty vec when `is_available()` is false, no stderr output
- [x] 2.4 Add unit tests for `ZoxideProvider`: unavailable returns empty, trait contract

## 3. Query Filtering

- [x] 3.1 Implement `filter_candidates()` in `src/complete/filter.rs` with 5-tier path-aware matching: exact absolute path → exact basename → absolute-path prefix → basename prefix → substring
- [x] 3.2 Implement case-insensitive matching across all tiers
- [x] 3.3 Preserve input ordering within each match tier (mode's native order)
- [x] 3.4 Add unit tests: exact basename ranks first, prefix match, substring match, case-insensitive, no match returns empty

## 4. Ancestors Mode

- [x] 4.1 Implement `ancestors::complete()` in `src/complete/ancestors.rs`: walk from cwd to filesystem root, collect parent directories nearest-first
- [x] 4.2 Apply optional query filtering via `filter_candidates()`
- [x] 4.3 Return empty vec when cwd is root
- [x] 4.4 Add unit tests: full ancestor list, filtered list, root returns empty

## 5. Paths Mode

- [x] 5.1 Add a candidate-collection method to `Resolver` (or a parallel function) that collects all matches at abbreviation/fallback/bookmark stages instead of failing on ambiguity
- [x] 5.2 Implement `paths::complete()` in `src/complete/paths.rs`: delegate to resolver's candidate-collection, deduplicate by absolute path, order by resolution precedence
- [x] 5.3 Add unit tests: multiple abbreviation candidates, bookmark match included, no match returns empty

## 6. Recents and Stack Modes

- [x] 6.1 Implement `recents::complete()` in `src/complete/recents.rs`: read session stack undo entries, reverse to most-recent-first, apply optional query filtering
- [x] 6.2 Handle missing session gracefully: return empty vec (no error) when session ID is absent or session file does not exist
- [x] 6.3 Implement `stack::complete()` in `src/complete/stack.rs`: read undo (back) or redo (forward) entries, reverse to stack-top-first, apply optional query filtering
- [x] 6.4 Add unit tests for recents: history order, empty session, filtered, missing session
- [x] 6.5 Add unit tests for stack: back entries, forward entries, empty, filtered

## 7. Frecents Mode

- [x] 7.1 Implement `frecents::complete()` in `src/complete/mod.rs` or a dedicated helper: delegate to `FrecencyProvider::query()`, return paths as-is (provider handles ranking/filtering)
- [x] 7.2 Return empty vec when provider is unavailable
- [x] 7.3 Add unit tests using a mock `FrecencyProvider`: results returned, unavailable returns empty

## 8. Output Formatting and CLI Handlers

- [x] 8.1 Implement plain output formatter: one absolute path per line to stdout, empty output for no results
- [x] 8.2 Implement JSON output formatter: serialize `Vec<Candidate>` as JSON array with `path`, `label`, `rank` fields; `[]` for no results
- [x] 8.3 Implement label generation: derive human-readable label from path (e.g., last two path components)
- [x] 8.4 Ensure all completion CLI handlers exit with code 0 regardless of result count
- [x] 8.5 Wire `dx complete paths <query>` CLI handler in `src/cli/complete.rs`
- [x] 8.6 Wire `dx complete ancestors [query]` CLI handler
- [x] 8.7 Wire `dx complete frecents [query]` CLI handler
- [x] 8.8 Wire `dx complete recents [query] [--session]` CLI handler
- [x] 8.9 Wire `dx complete stack --direction back|forward [query] [--session]` CLI handler
- [x] 8.10 Add `--json` flag support to all complete subcommands
- [x] 8.11 Add unit tests for plain and JSON output formatting

## 9. Navigation Selector Resolution

- [x] 9.1 Implement selector parsing: empty/absent → first candidate, integer string → Nth (1-based), non-integer → closest path match
- [x] 9.2 Implement closest-match scoring with 5-tier rules: exact absolute path → exact basename → absolute-path prefix → basename prefix → substring; ties preserve mode's native order
- [x] 9.3 Add `Navigate` CLI subcommand with `up|back|forward [selector]` and optional `--session` flag
- [x] 9.4 Wire `dx navigate up [selector]` to ancestors mode + selector resolution, print single path to stdout
- [x] 9.5 Wire `dx navigate back [selector]` to stack back mode + selector resolution
- [x] 9.6 Wire `dx navigate forward [selector]` to stack forward mode + selector resolution
- [x] 9.7 Implement error output: diagnostic to stderr + exit non-zero for no match, out of range, or empty candidate list
- [x] 9.8 Register `Navigate` variant in `Commands` enum and dispatch in `src/cli/mod.rs`
- [x] 9.9 Add unit tests: empty selector, numeric selector, path selector, out of range, no match

## 10. Shell Hook Updates — Navigation Wrappers

- [x] 10.1 Add Bash wrapper functions to `src/hooks/bash.rs`: `up`, `back`, `forward`, `cdf`, `z`, `cdr`, `cd-`, `cd+` — each delegates to `dx navigate` or `dx complete` + native cd + `dx push`
- [x] 10.2 Add Zsh wrapper functions to `src/hooks/zsh.rs` with same wrapper set
- [x] 10.3 Add Fish wrapper functions to `src/hooks/fish.rs` with same wrapper set
- [x] 10.4 Add PowerShell wrapper functions to `src/hooks/pwsh.rs` with same wrapper set
- [x] 10.5 Ensure `cd-`/`back` and `cd+`/`forward` are aliases for the same behavior in each shell

## 11. Shell Hook Updates — Completion Bindings

- [x] 11.1 Add Bash completion functions (`_dx_complete_paths`, `_dx_complete_ancestors`, etc.) and `complete -F` bindings for `dx` and all wrapper functions in `src/hooks/bash.rs`
- [x] 11.2 Add Zsh `_dx_complete_*` functions and `compdef` bindings for `dx` and all wrappers in `src/hooks/zsh.rs`
- [x] 11.3 Add Fish `complete -c` entries for `dx` subcommands and all wrapper functions in `src/hooks/fish.rs`
- [x] 11.4 Add PowerShell `Register-ArgumentCompleter` handlers for `cd`, `Set-Location`, and all wrapper functions in `src/hooks/pwsh.rs`
- [x] 11.5 Implement `dx` subcommand completion dispatch: `dx resolve` → paths mode, `dx complete` → mode names, `dx push` → filesystem default

## 12. Integration Tests

- [x] 12.1 Create `tests/complete_cli.rs` with tests for `dx complete ancestors` (full list, filtered, root)
- [x] 12.2 Add tests for `dx complete paths` (abbreviation candidates, no match)
- [x] 12.3 Add tests for `dx complete recents` and `dx complete stack` (session-based, empty session, missing session)
- [x] 12.4 Add tests for `dx complete frecents` (graceful degradation when zoxide absent)
- [x] 12.5 Add tests for `--json` output format across modes
- [x] 12.6 Add tests for error cases: missing mode, invalid mode, stack without direction
- [x] 12.7 Create `tests/navigate_cli.rs` with tests for `dx navigate up|back|forward` with empty, numeric, and path selectors
- [x] 12.8 Update `tests/init_cli.rs` to verify completion bindings and navigation wrapper functions appear in init output for all shells

## 13. Build and Verification

- [x] 13.1 Run `cargo build` and fix any compilation errors
- [x] 13.2 Run `cargo test` and fix any test failures
- [x] 13.3 Verify all 5 completion modes produce correct plain and JSON output
- [x] 13.4 Verify shell hook output includes completion bindings and navigation wrappers for all 4 shells
