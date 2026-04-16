---
type: planning
entity: implementation-plan-review-collated
plan: "whole-repo-review-remediation"
status: complete
created: "2026-04-16"
---

# Collated Implementation-Plan Review Summary (2026-04-16)

This artifact collates reviewer verdicts for implementation phases 1-4, identifies unanimous vs majority findings, and records how those findings were resolved in revised implementation-planning artifacts.

## Reviewer Verdict Rollup

| Phase | Reviewer 1 | Reviewer 2 | Reviewer 3 | Collated Outcome |
|------:|------------|------------|------------|------------------|
| 1 | Conditionally Accepted | Approved | Needs Revision | Revised before execution (major actionability/testing gaps closed) |
| 2 | Conditional Pass | Pass with Comments | Needs Revision | Revised before execution (grammar/scope/verify gaps closed) |
| 3 | Conditional Pass | Approved with Modifications | Needs Revision | Revised before execution (runtime mode + verify + deletion gate clarity closed) |
| 4 | Conditional Pass | Approved | Needs Revision | Revised before execution (verify + ProxyCommand actionability/context gaps closed) |

---

## Unanimous / Majority Findings and Resolutions

## Phase 1

### Majority findings

1. **Failure-injection strategy was not concrete/actionable enough** (R1 major, R3 major).
2. **Verify command too narrow / false-confidence risk** (R1 minor, R3 major).
3. **Step 2 ambiguity (likely verification-only no-op) needed clarification** (R1 minor).
4. **`cleanup_stale` interaction should be acknowledged for session path** (R1 minor).

### Resolution in revised plan

Updated `implementation/phase-1-impl.md`:

- Chose one deterministic strategy: **narrow test-only replace-failure seam** in `src/common/mod.rs` + caller-level tests in `src/bookmarks/storage.rs` and `src/stacks/storage.rs`.
- Aligned Step 3, Open Decisions, Required Context, Testing Plan, and Reality Check around that single mechanism/placement.
- Clarified Step 2 as **verification-first; expected no-op unless mismatch discovered**.
- Replaced verify section with one executable chained command that:
  - guards against zero-test false-pass via `--list | rg -q ...`,
  - runs targeted bookmark + session durability tests,
  - runs required full `cargo test`.
- Added explicit `cleanup_stale` note in Step 3 considerations + Reality Check.

## Phase 2

### Majority findings

1. **Completion API change shape was underspecified** (R1 medium, R3 major; R2 low comment aligned).
2. **Flagged `cd` behavior matrix was incomplete** (`cd -`, `cd -P ` / `cd -- ` pre-path state, shell-specific semantics) (R1 low, R3 major).
3. **PowerShell handling of POSIX flags needed explicit fallback/non-intervention** (R1 medium, R3 major).
4. **Verify command referenced non-existent/insufficiently scoped targets** (R1 high, R3 major).
5. **Required Context/Reality Check needed hook-layer/fallback anchors** (R3 minor, R1 medium anchor refinement).

### Resolution in revised plan

Updated `implementation/phase-2-impl.md`:

- Made API shape explicit: added shared resolver seam variant with injected cwd, plus required redirection of all internal cwd usages in `src/resolve/completion.rs`.
- Added explicit support/fallback decisions for:
  - `cd -L <path>`, `cd -P <path>`, `cd -- <path>` (support for POSIX flows),
  - `cd -P ` / `cd -- ` pre-path state (intervention with empty query region),
  - `cd -` (fallback),
  - quoted flagged paths (support),
  - unknown/grouped flags (fallback),
  - PowerShell `--psreadline-mode` with POSIX forms (fallback/non-intervention).
- Expanded Required Context and Reality Check to include `tests/menu_cli.rs`, `src/hooks/pwsh.rs`, `src/hooks/mod.rs`, and precise `src/menu/mod.rs` + `src/resolve/completion.rs` anchors.
- Replaced verify section with one executable chained command that checks presence of targeted parser + menu CLI tests, executes them, then runs full `cargo test`.

Updated `verification/shell-smoke-matrix.md`:

- Split formerly combined flagged-form row into:
  - Bash/Zsh supported POSIX flagged forms,
  - PowerShell non-intervention/fallback expectation for POSIX forms.

## Phase 3

### Majority findings

1. **Replacement runtime tests needed explicit `--command-not-found` init mode** (R3 major; also implied by R1/R2 concerns).
2. **Verify command pointed to non-existent target / false-pass risk** (R1 high, R3 major, R2 medium mismatch).
3. **Equivalence needed reframing to generated-hook authoritative behavior, not prototype quirks** (R1 high divergence note, R3 minor/major framing).
4. **Deletion/migration disposition of existing legacy-sourcing tests needed to be explicit** (R1 medium).
5. **Missing required context for init CLI surfaces** (`src/cli/mod.rs`, `src/cli/init.rs`, `tests/init_cli.rs`) (R3 minor).
6. **Runtime mocking strategy for `dx` in generated-hook tests needed explicit guidance** (R2 low).

### Resolution in revised plan

Updated `implementation/phase-3-impl.md`:

- Approach and Step 1 now explicitly require runtime coverage via:
  - `dx init bash --command-not-found`,
  - `dx init zsh --command-not-found`.
- Reframed equivalence baseline to **generated-hook authoritative behavior + documented contracts**, explicitly noting known prototype-vs-generated heuristic divergence.
- Enumerated disposition of the three legacy-sourcing Bash tests as migration targets.
- Added explicit deletion targets:
  - `scripts/hooks/dx.bash`,
  - `scripts/hooks/dx.zsh`,
  - legacy `tests/shell_hook_guard.rs` implementations that source `scripts/hooks/*`.
- Added Required Context rows for `src/cli/mod.rs`, `src/cli/init.rs`, `tests/init_cli.rs`.
- Added runtime mocking consideration (temporary PATH shim preferred; exported function fallback acceptable).
- Replaced verify section with one executable chained command covering:
  - migrated shell-hook runtime tests,
  - existing `src/hooks/mod.rs` generated-hook guard contract test,
  - required full `cargo test`.

## Phase 4

### Majority findings

1. **Verify command referenced non-existent test / false-pass risk** (R1 medium, R3 major).
2. **ProxyCommand evaluation was under-specified operationally** (R1 medium, R3 major).
3. **Required Context/Reality Check missed real anchors** (`unwrap()` site precision, additional `ResolveMode` call sites) (R3 major/minor; R1 low wording correction).
4. **Smoke matrix wording too vague (“sanity”) for evidence closure** (R1 low).

### Resolution in revised plan

Updated `implementation/phase-4-impl.md`:

- Replaced verify command with one executable chained command that runs existing menu-related hook contract tests and full `cargo test`.
- Added conditional secondary checks (only if ProxyCommand adopted) in text rather than broken base exact target.
- Made ProxyCommand evaluation concrete:
  - explicit **120-minute time-box** (or first hard blocker),
  - minimum scenario set,
  - adopt/reject criteria,
  - explicit evidence destination: `verification/proxycommand-eval-2026-04-16.md`.
- Added missing Required Context/Reality Check anchors:
  - `src/hooks/mod.rs:122-125` unwrap locations,
  - `src/resolve/output.rs`, `src/cli/resolve.rs`, `benches/resolve_latency.rs`, `tests/resolve_latency.rs` for `ResolveMode` impact.
- Clarified hygiene wording to hook ordering/positioning tests (not vague marker-test phrasing).

Updated `verification/shell-smoke-matrix.md`:

- Replaced vague Phase 4 “sanity” expected results with concrete observable pass criteria per shell.

---

## Reviewer Differences and Resolution Strategy

- Where one reviewer approved and others requested revision, resolution followed **majority/criticality weighting** and executable-plan quality bars:
  - deterministic test mechanism selection,
  - concrete API/grammar decisions,
  - shell-specific behavioral scoping,
  - zero-test false-pass prevention,
  - explicit evidence destinations and criteria.
- Low-severity disagreements were resolved in favor of keeping scope bounded while making execution constraints explicit (for example, optional zsh runtime expansion kept as optional, with existing generated-hook contract tests treated as baseline evidence).

---

## Artifacts Revised from This Collation

- `implementation/phase-1-impl.md`
- `implementation/phase-2-impl.md`
- `implementation/phase-3-impl.md`
- `implementation/phase-4-impl.md`
- `verification/shell-smoke-matrix.md`
