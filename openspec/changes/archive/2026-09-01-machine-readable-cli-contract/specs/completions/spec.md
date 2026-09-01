## MODIFIED Requirements

### Requirement: JSON Output Format
When `--json` is specified, all completion modes SHALL output a JSON array of candidate objects to stdout. Each object SHALL contain a `path` field (absolute path string), a `label` field (human-readable display string), and a `rank` field (1-based integer reflecting position in the candidate list). Empty results SHALL produce `[]`. The command SHALL always exit with code 0.

The JSON document SHALL be terminated by exactly one newline. Every command that emits this array — including `dx stack --list --json` — SHALL produce byte-identical output for the same candidate list.

No `mode` or `direction` fields SHALL be included in the JSON output; the caller already knows its invocation context.

#### Scenario: JSON output with candidates
- **WHEN** `dx complete ancestors --json` is invoked from `/a/b/c`
- **THEN** stdout SHALL contain a JSON array where each element has `path`, `label`, and `rank` fields, and `rank` values are sequential starting from 1

#### Scenario: JSON label for ancestors
- **WHEN** `dx complete ancestors --json` is invoked from `/home/user/code`
- **THEN** the `label` for `/home/user` SHALL be a human-readable representation (e.g., `user` or `home/user`)

#### Scenario: Empty JSON results
- **WHEN** a completion mode produces no candidates and `--json` is specified
- **THEN** stdout SHALL contain `[]` and the exit code SHALL be 0

#### Scenario: JSON output is newline-terminated
- **WHEN** any completion mode is invoked with `--json`
- **THEN** stdout SHALL end with exactly one newline following the closing bracket

#### Scenario: Stack listing matches completion output
- **WHEN** `dx stack --list --json` and `dx complete stack --json` are invoked over the same session stack in the same direction
- **THEN** both SHALL write byte-identical output to stdout
