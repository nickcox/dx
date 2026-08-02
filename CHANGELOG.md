# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
While the major version is `0`, a minor bump may carry breaking changes; those
are always listed under **Breaking**.

## [Unreleased]

### Added

- `dx bookmarks prune` removes bookmarks whose target directory no longer
  exists, reporting each one rather than deleting silently.
- Bookmark names are completed by prefix, so `cd wo` offers the target of a
  bookmark named `work`. Candidates are listed after any filesystem matches, and
  stale bookmarks are excluded.
- `dx bookmarks` marks entries whose target is missing with `(missing)`.
- `dx bookmarks add` and `remove` print the absolute path they acted on, making
  the symlink resolution done at save time visible.
- The path matching rules are documented properly: the one-segment-per-level
  model, a worked directory tree with a table of eleven queries and what each
  resolves to, the grammar of a single segment, and why case sensitivity
  defaults on. This makes explicit that a query with no separator only matches
  directories directly inside a search root — `dx` alone will not find
  `projects/dx`, which nothing previously said.
- How to remove the shell integration, for every supported shell, including
  which stored state survives on purpose and where it lives.
- Search roots and `--command-not-found` now appear in the README highlights and
  have their own quickstart steps. Abbreviated paths only reach below the
  current directory until search roots are configured, which was easy to miss.
- The session file lifecycle is documented: where the files live, that `dx`
  removes ones untouched for seven days, how often it sweeps, and the 5000-entry
  cap on each direction of a session's history.
- Guidance on loading `dx` alongside other tools that wrap `cd`, such as
  zoxide's `--cmd cd` integration: initialise `dx` last, or its `cd` is replaced
  and abbreviated paths silently stop resolving. Includes a troubleshooting
  entry for that symptom.
- A [scripting guide](docs/scripting.md) documenting every JSON shape, the
  exit-code contract, the stdout/stderr split, and `dx navigate`, which was
  previously undocumented.
- `nix build .#dx` works as an alias for `nix build .#cdex`, matching the
  `nix run .#dx` that the docs already showed.

### Changed

- `dx complete <mode> --json` now ends with a newline, making it byte-identical
  to `dx stack --list --json` for the same candidates.

### Fixed

- The interactive menu no longer becomes unresponsive when scrolling quickly
  through a long list. It was asking the terminal to report every pointer
  movement — events it then discarded, each still costing a full redraw of every
  candidate. It now requests only wheel events, rebuilds candidate labels only
  when the query changes, and folds a queued run of scrolls into one move. For a
  flick of ~93 events over 4000 candidates this cuts render output from 52 KB to
  under 1 KB.
- Menu teardown releases the mouse before restoring the cursor and clearing the
  menu rows, and emits the whole sequence in one write. Releasing it last meant
  a cleanup that failed part way through could leave the terminal reporting
  mouse movement to the shell indefinitely, where the other steps only leave
  cosmetic marks the next prompt paints over.

### Breaking

- `dx resolve` now exits `0` if and only if the query resolved to exactly one
  directory. `--list` and `--json` are presentation flags and no longer change
  success: an ambiguous query under either flag exits `1` instead of `0`, with
  the candidates still on stdout and nothing on stderr. Scripts using
  `dx resolve q --list && …` change meaning.
- `dx bookmarks --json` emits an array of `{name, path, exists}` objects instead
  of an object mapping names to paths.
- `dx bookmarks add --json` and `remove --json` emit a single JSON object
  instead of ignoring the flag and printing a bare path.

## [0.12.0] - 2026-08-01

### Added

- Menu settings can be set in `config.toml` as well as the environment, with the
  environment taking precedence.
- Mouse wheel scrolling in the interactive menu.
- MIT license.

### Changed

- Menu label widths are measured in terminal cells rather than characters, so
  double-width characters such as CJK ideographs no longer overflow their
  column.
- One truthiness rule now applies to every boolean setting.
- Session history is capped at 5000 entries per direction.
- Repeat visits are collapsed in stack and recents completion, so `back 3` means
  three places back rather than three entries.
- Oversized navigation selectors are reported instead of silently clamped.
- PowerShell re-anchors the prompt before editing the buffer, fixing stale
  redraw positions.
- Bookmark paths that TOML cannot represent are refused rather than stored
  lossily.
- Faster completion and menu rendering: candidates are no longer canonicalised
  on every keystroke, filesystem completion drops redundant canonicalisation,
  only visible menu rows are styled, `dx stack push` is faster, and the
  directory walk skips directories it could never have entered.

### Fixed

- Prompt prefix handling in the menu.
- Menu row truncation.

## [0.11.0] - 2026-07-21

### Added

- Native PowerShell menu using PSReadLine's own completion UI, via
  `dx init pwsh --native-menu`.
- Clap-based completion for the `dx` command itself, covering subcommands,
  options and enum values.

### Fixed

- Backslashes were being escaped incorrectly in PowerShell.

## [0.10.0] - 2026-07-20

### Added

- User-facing documentation.

### Fixed

- Relative path resolution.
- POSIX menu placement now uses the measured cursor position rather than a
  guess.
- Shell stack navigation handles operating-system protected directories that
  allow a builtin `cd` but refuse to start child processes there.

## [0.9.0] - 2026-07-19

### Added

- Portable Windows path resolution, with CI coverage.
- Secure menu command mappings.

### Changed

- Hardened stack persistence, config loading, and atomic writes.
- Improved PowerShell `Set-Location` wrapper and post-menu redraw.

### Breaking

- Windows drive-relative queries such as `C:work` are rejected rather than
  resolved against an implicit per-drive working directory.

## [0.8.0] - 2026-07-18

### Changed

- The PowerShell integration loads as an in-memory module, removing the need for
  a file on disk.
- PowerShell commands use idiomatic verb-noun names (`Set-DxLocation`,
  `Step-Up`, `Undo-Location`, `Redo-Location`, `Set-FrecentLocation`,
  `Set-RecentLocation`).

## [0.7.0] - 2026-05-17

### Fixed

- The jump origin is seeded consistently before `cd`, so `back` returns where
  you expect after a frecent or recent jump.
- Directories are completed with a trailing slash in path mode.

## [0.6.0] - 2026-05-17

### Changed

- Dependency updates and internal tidying. No user-visible changes.

## [0.5.0] - 2026-05-17

### Added

- `PageUp` and `PageDown` navigation in the menu.
- `LS_COLORS` support for unselected filesystem candidates.
- Alias support for mapped commands in PowerShell.

### Changed

- Menu labels are rendered relative to the query style.
- The status bar shows the full resolved path.
- New menu defaults for rows and item width.

### Fixed

- Root path completion for mapped commands.
- Trailing slashes are placed inside quotes rather than after them.

## [0.4.0] - 2026-05-10

### Added

- Configurable menu command mappings, so commands such as `ls`, `open` and `cat`
  can use menu-backed filesystem completion.
- A configurable PowerShell menu keybinding, with fallback to the previous
  binding.

### Fixed

- The prompt is no longer redrawn when the TUI was never displayed.

## [0.3.0] - 2026-05-02

### Added

- The `dx` binary and its shell hook distribution model.
- The interactive completion menu, with live buffer sync, live filtering,
  multi-column layout, and dynamic height.
- Path abbreviation matching.
- Delimiter-aware path shortening, so `cd-e` matches `cd-extras`.
- `dx stack` listing and clearing.

### Changed

- Menu filtering is clamped to extensions of the initial query.
- Cancelling the menu restores the original prompt.
- Completion fallback behaviour is consistent across shells.

### Breaking

- The pure-PowerShell `cd-extras` module is deprecated in favour of the
  binary-plus-hook distribution model.

## [0.2.0] - 2026-04-22

### Changed

- The menu is multi-column and borderless by default.

### Added

- Homebrew tap, updated automatically on release.

## [0.1.0] - 2026-04-22

Initial release.

### Added

- Path resolution with abbreviated segments and fallback search roots.
- Session stacks for back and forward navigation.
- Named directory bookmarks.
- Completions for paths, ancestors, frecents, recents and the session stack.
- Shell hooks for Bash, Zsh, Fish and PowerShell.
- The current directory is included as a search root by default.
- A Nix flake.
