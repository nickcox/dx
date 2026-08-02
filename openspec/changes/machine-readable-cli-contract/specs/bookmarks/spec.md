## MODIFIED Requirements

### Requirement: Output Contract
All bookmark CLI commands MUST follow the dx output contract: success outputs result to stdout and exits with code 0; failure outputs diagnostic to stderr and exits with a non-zero code.

A successful `dx bookmarks add` MUST print the canonical absolute path that was stored, so that path canonicalization — which resolves symlinks — is visible. A successful `dx bookmarks remove` MUST print the absolute path of the bookmark that was removed.

With `--json`, `add` and `remove` MUST instead output a single JSON object containing `name`, `path`, and `exists`, matching the shape of one element of the list output. Single-bookmark operations emit one object; operations over many bookmarks emit an array. For `remove`, `exists` MUST report whether the removed bookmark's target was still a directory.

The `--json` flag MUST be accepted both before and after the subcommand.

#### Scenario: Add success output
- **WHEN** `dx bookmarks add proj` succeeds
- **THEN** the system MUST print the stored canonical absolute path to stdout and exit with code 0

#### Scenario: Add reports the target behind a symlink
- **WHEN** `dx bookmarks add proj <symlink>` succeeds and the given path is a symlink to a directory
- **THEN** the printed path MUST be the canonical target rather than the symlink

#### Scenario: Remove success output
- **WHEN** `dx bookmarks remove proj` succeeds
- **THEN** the system MUST print the removed bookmark's absolute path to stdout and exit with code 0

#### Scenario: Remove failure output
- **WHEN** `dx bookmarks remove nonexistent` fails because the bookmark does not exist
- **THEN** the system MUST output a diagnostic message to stderr and exit with a non-zero code

#### Scenario: Add with --json flag
- **WHEN** `dx bookmarks add proj --json` succeeds
- **THEN** the system MUST output a single JSON object containing `name`, `path`, and `exists` keys rather than a bare path

#### Scenario: Remove with --json flag reports staleness
- **WHEN** `dx bookmarks remove proj --json` succeeds and the bookmark's target directory had already been deleted
- **THEN** the output object's `exists` value MUST be false
