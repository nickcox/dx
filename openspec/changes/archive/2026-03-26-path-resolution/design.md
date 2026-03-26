## Context

`path-resolution` defines the behavior of `dx resolve`, the core command that translates user input into an absolute directory path. This command is used in two high-frequency flows: (1) explicit navigation (`cd <query>` wrappers) and (2) command-not-found forwarding for auto-cd style usage.

Because this runs in interactive shells, behavior must be both deterministic and low-latency. Ambiguous resolution, inconsistent precedence, or shell-specific quirks would degrade trust quickly.

## Goals / Non-Goals

**Goals:**
- Define deterministic resolution precedence for all supported query forms.
- Guarantee shell-safe output contracts for hooks (single resolved path or structured failure).
- Keep median resolution latency low enough for interactive use.
- Make behavior consistent across Bash, Zsh, Fish, and PowerShell via shell-agnostic core logic.
- Support abbreviated multi-segment matching (e.g., `pr/po`), standard traversal (`..`, `~`), and configured search roots.

**Non-Goals:**
- Full fuzzy finder UX (handled separately by completion/menu capabilities).
- Persistent frecency scoring (handled by `frecency` capability; this design only defines resolve behavior).
- File (non-directory) path resolution.
- Re-implementing shell parser semantics in the core binary.

## Decisions

### D1: Resolution precedence order is explicit and stable

`dx resolve <query>` follows this precedence:
1. **Direct absolute/relative paths** (`/x`, `./x`, `../x`, `~`, `~/x`) with normalization.
2. **Step-up aliases** (e.g., `up`, repeated aliases if configured).
3. **Abbreviated directory segments** (`pr/po` style) against configured candidate roots.
4. **CD_PATH / fallback roots** exact or abbreviation-compatible matches.
5. **Failure** with explicit non-zero exit and machine-readable reason (with `--json`).

Why: predictable precedence prevents surprising jumps and makes shell integration testable.

Alternatives considered:
- Frecency-first matching: rejected for `resolve` because explicit path intent should beat heuristic ranking.
- Global fuzzy search by default: rejected due to ambiguity and latency cost.

### D2: Candidate discovery is root-scoped, not full-filesystem

Resolution scans only configured roots (e.g., project roots, CD_PATH entries, optional defaults), never the entire filesystem tree.

Why: bounds latency and avoids expensive accidental traversals.

Alternatives considered:
- Recursive global scans: rejected for performance and unpredictability.

### D3: Abbreviation matching is segment-aware

For `a/b/c` style queries, each query segment must match a candidate segment prefix in order. Matching is case-sensitive by default on case-sensitive filesystems and configurable otherwise.

Why: segment-aware matching mirrors current `cd-extras` mental model and reduces false positives compared with raw fuzzy substring search.

Alternatives considered:
- Character-level fuzzy scoring across full paths: rejected for lower explainability.

### D4: Ambiguity handling is explicit

If multiple candidates tie at best score:
- Default behavior: return ambiguity failure (do not guess).
- Optional `--list`/`--json` mode: return ranked candidates for shell/UI selection.

Why: correctness over convenience for command execution paths.

Alternatives considered:
- Auto-pick first lexicographic match: rejected as surprising.

### D5: Output contract is shell-friendly first, structured optional

- Default: print one absolute path to stdout on success; no extra text.
- Failure: no stdout path output; diagnostics to stderr; non-zero exit.
- `--json`: structured object containing status, resolved path or failure reason, and optional candidate list.

Why: keeps shell wrappers simple and robust while supporting advanced tooling.

### D6: Rust project structure uses a single binary crate with internal modules

The `dx` binary is a single Cargo binary crate rooted at the repository top level. Internal concerns are organized as modules under `src/`, not as separate workspace crates (premature for a single binary).

```
dx/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, CLI arg parsing
│   ├── cli/
│   │   ├── mod.rs            # CLI command dispatch
│   │   └── resolve.rs        # `dx resolve` command handler
│   ├── resolve/
│   │   ├── mod.rs            # Public resolve API
│   │   ├── traversal.rs      # Direct/relative/home/step-up resolution
│   │   ├── abbreviation.rs   # Segment-aware abbreviated matching
│   │   ├── roots.rs          # Fallback root / CD_PATH scanning
│   │   └── precedence.rs     # Orchestrates the precedence chain
│   └── config/
│       └── mod.rs            # Configuration loading (search roots, options)
├── tests/
│   ├── resolve_traversal.rs  # Integration tests for basic traversal
│   ├── resolve_abbrev.rs     # Integration tests for abbreviation matching
│   └── resolve_cli.rs        # CLI output contract tests
├── openspec/                 # Specifications (existing)
└── docs/                     # Project documentation (existing)
```

Why: a flat binary crate is the simplest starting point. Modules provide encapsulation without the overhead of workspace management. If the project grows to include a library crate (e.g., for shell-completion or a daemon), we can extract `src/resolve/` into a `dx-core` library crate later.

Alternatives considered:
- Cargo workspace with `dx-core` library + `dx` binary from the start: rejected as premature given the single-binary scope of this change.

### D7: CLI framework uses `clap` with derive macros

`clap` (derive) is the standard Rust CLI framework. It handles arg parsing, subcommands, `--json`/`--list` flags, help generation, and shell completion generation out of the box.

Why: well-maintained, widely adopted, supports all the output modes we need, and generates completions for Bash/Zsh/Fish/PowerShell.

Alternatives considered:
- Manual arg parsing: rejected for maintenance cost.
- `argh`: rejected for smaller ecosystem and less flexible completion support.

### D8: Command-not-found forwarding uses guarded resolve attempts

Shell hooks should only call `dx resolve` for command-not-found inputs that look path-like (e.g., contain `/`, `.`, `~`, or configured aliases), then fall back to native shell errors on resolve miss.

Why: prevents unnecessary subprocess calls and avoids latency for ordinary command typos.

## Risks / Trade-offs

- **[Risk] Ambiguity frustrates users when no auto-pick happens** → Mitigation: provide `--list` output and integrate with completion/menu flows for fast disambiguation.
- **[Risk] Large configured roots degrade performance** → Mitigation: cache directory index metadata with bounded TTL and invalidate on explicit refresh.
- **[Risk] Cross-platform path normalization differences** → Mitigation: centralize canonicalization rules and test on macOS/Linux/Windows paths.
- **[Trade-off] Deterministic precedence vs heuristic convenience** → We prioritize predictability for execution correctness; heuristic ranking is available in list/menu contexts.
- **[Trade-off] Guarded command-not-found forwarding may miss edge shorthand** → Acceptable to preserve shell responsiveness and avoid intercepting normal command workflows.

## Migration Plan

1. Specify and validate precedence and ambiguity rules in `specs/path-resolution/spec.md`.
2. Implement resolver with pure unit tests covering normalization, segment matching, and tie behavior.
3. Add shell-hook contract tests for success/failure output handling.
4. Add performance benchmarks on representative root sizes and enforce regression thresholds.
5. Roll into integrated `dx` hooks with command-not-found guards and manual shell verification.

## Open Questions

- Should case-sensitivity be auto-detected from filesystem semantics only, or user-configurable per shell?
- Should `resolve` expose a configurable max candidate count in `--list` mode?
- How should symlink canonicalization be handled (preserve user-facing symlink path vs resolve realpath)?
- Do we need a compatibility flag to mirror current PowerShell `cd-extras` precedence exactly during migration?
