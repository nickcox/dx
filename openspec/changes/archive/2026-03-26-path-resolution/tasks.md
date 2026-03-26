## 1. Rust Project Scaffolding

- [x] 1.1 Initialize Cargo binary crate (`cargo init --name dx`) with Rust edition 2021+
- [x] 1.2 Add `clap` (derive) dependency for CLI arg parsing
- [x] 1.3 Create module structure: `src/cli/`, `src/resolve/`, `src/config/`
- [x] 1.4 Wire up `clap` subcommand skeleton with `dx resolve <query>` (stub that prints the query back)
- [x] 1.5 Verify `cargo build` and `cargo test` pass on the empty scaffold

## 2. Configuration and Search Roots

- [x] 2.1 Define config types for search roots and resolution options
- [x] 2.2 Implement config loading (e.g., from `~/.config/dx/config.toml` or environment variables)
- [x] 2.3 Write tests for config loading and defaults

## 3. Basic Path Resolution (Traversal)

- [x] 3.1 Define core types (`ResolveQuery`, `ResolveResult`, `ResolveError`)
- [x] 3.2 Implement resolution for absolute paths (`/x`) with existence validation
- [x] 3.3 Implement resolution for relative paths (`./x`, `x`) against current working directory
- [x] 3.4 Implement home directory expansion (`~`, `~/x`)
- [x] 3.5 Implement parent directory traversal (`..`, `../x`)
- [x] 3.6 Write tests for all basic traversal scenarios (including nonexistent paths)

## 4. Step-up Aliases

- [x] 4.1 Implement multi-dot expansion (`...` -> 2 levels up, `....` -> 3 levels up, N dots -> N-1 levels)
- [x] 4.2 Write tests for step-up aliases including edge cases (root directory, excessive depth)

## 5. Abbreviated Segment Matching

- [x] 5.1 Implement query parsing for multi-segment abbreviations (e.g., `f/b/b`)
- [x] 5.2 Implement segment-aware prefix matching against configured search roots
- [x] 5.3 Implement candidate collection and deterministic ranking
- [x] 5.4 Write tests for unambiguous matches, ambiguous matches, and no-match cases

## 6. Fallback Roots

- [x] 6.1 Implement fallback root scanning (CD_PATH-style) for exact and abbreviated matches
- [x] 6.2 Write tests for fallback root matching and precedence relative to direct paths

## 7. Resolution Precedence Chain

- [x] 7.1 Implement the precedence orchestrator (direct -> step-up -> abbreviated -> fallback -> failure)
- [x] 7.2 Write tests verifying precedence order (e.g., direct path wins over fallback root match)

## 8. CLI Output and Error Contracts

- [x] 8.1 Implement success output (single absolute path to stdout, exit code 0)
- [x] 8.2 Implement failure output (nothing to stdout, diagnostic to stderr, non-zero exit code)
- [x] 8.3 Implement ambiguity failure output (diagnostic listing candidates to stderr)
- [x] 8.4 Implement `--list` flag (all candidates to stdout, one per line)
- [x] 8.5 Implement `--json` flag (structured JSON object with status, path/candidates, reason)
- [x] 8.6 Write CLI integration tests verifying output contracts and exit codes for all modes

## 9. Shell Integration and Performance

- [x] 9.1 Add integration benchmarks ensuring resolution latency is under 50ms for typical configurations
- [x] 9.2 Create prototype shell hook scripts (Bash/Zsh) that consume `dx resolve` output
- [x] 9.3 Verify shell hooks do not cause recursion loops when `dx resolve` is called from `cd` wrapper or `command_not_found` handler
- [x] 9.4 Document shell hook guarding strategy for command-not-found forwarding
