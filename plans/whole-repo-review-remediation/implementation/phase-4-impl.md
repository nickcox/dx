---
type: planning
entity: implementation-plan
plan: "whole-repo-review-remediation"
phase: 4
status: completed
created: "2026-04-16"
updated: "2026-04-19"
---

# Implementation Plan: Phase 4 - Finalize Hygiene and PowerShell Decision

> Implements [Phase 4](../phases/phase-4.md) of [whole-repo-review-remediation](../plan.md)

## Approach

Treat Phase 4 as bounded closeout work, not a new feature phase: complete only the accepted hygiene items listed in the gated phase, run an evidence-driven and time-boxed `ProxyCommand` evaluation against the current explicit PowerShell wrapper baseline, and close remaining Phase 4 smoke/documentation artifacts. Keep default posture as **reject unless proven better**.

## Affected Modules

| Module | Change Type | Description |
|--------|-------------|-------------|
| `src/resolve` | modify (bounded) | Complete accepted API hygiene only: `Resolver.config` visibility tightening and `ResolveMode` parameter cleanup/justification. |
| `src/test_support` | modify (docs/comments) | Clarify shared global `env_lock()` contract for env-mutating tests across modules. |
| `src/hooks` | modify (tests, bounded) | Replace remaining bare `unwrap()` marker assertions with diagnostic `expect(...)`; add menu-enabled balanced-delimiter checks. |
| `src/hooks/pwsh.rs` | evaluate (conditional modify) | Compare explicit wrapper baseline vs bounded `ProxyCommand` prototype; adopt only if gate criteria are met. |
| `tests` | modify (conditional, bounded) | Keep shell/menu contracts stable while adding only evidence-oriented coverage needed for accepted hygiene or adopted PowerShell path. |
| `plans/whole-repo-review-remediation/verification` | modify | Capture final shell smoke rows and explicit ProxyCommand adopt/reject evidence. |

## Required Context

| File | Why |
|------|-----|
| `plans/whole-repo-review-remediation/phases/phase-4.md` | Bounded scope and acceptance gates for hygiene + PowerShell decision. |
| `plans/whole-repo-review-remediation/plan.md` | Overall Definition of Done and phase-close alignment required by Phase 4 finalization work. |
| `plans/whole-repo-review-remediation/todo.md` | Active execution backlog and final closeout state updates required when Phase 4 completes. |
| `plans/whole-repo-review-remediation/phases/phase-3.md` | Confirms dependency baseline: Phase 3 is completed/accepted before Phase 4 starts. |
| `plans/whole-repo-review-remediation/implementation/phase-3-impl.md` | Carries forward accepted verification posture and known non-blocking follow-ups. |
| `plans/whole-repo-review-remediation/reviews/impl-review-phase-3-collated-2026-04-17.md` | Reviewer follow-up context to classify into in-scope vs deferred for Phase 4. |
| `src/resolve/mod.rs` | `Resolver.config` current visibility anchor (`src/resolve/mod.rs:50-52`). |
| `src/resolve/pipeline.rs` | Unused `_mode: ResolveMode` parameter anchor (`src/resolve/pipeline.rs:8-12`). |
| `src/resolve/output.rs` | `ResolveMode` usage in execution/output path if parameter is removed (`src/resolve/output.rs:14-61`). |
| `src/cli/resolve.rs` | CLI call site constructing `ResolveMode` (`src/cli/resolve.rs:3-12`). |
| `tests/resolve_cli.rs` | Named behavior guardrails for default/list/json resolve contracts (`tests/resolve_cli.rs:27-108`) if `ResolveMode` plumbing is touched. |
| `tests/resolve_precedence.rs` | Secondary precedence guardrails when validating resolver behavior remains unchanged (`tests/resolve_precedence.rs:78-120`). |
| `src/test_support.rs` | Shared env lock helper missing explicit contract docs (`src/test_support.rs:1-9`). |
| `src/hooks/mod.rs` | Existing hook tests: ordering-position `unwrap()` sites (`src/hooks/mod.rs:122-125`) and non-menu-only delimiter coverage (`:369-386`; menu marker only at `:401-413`). |
| `src/hooks/pwsh.rs` | Baseline explicit wrapper behavior for ProxyCommand comparison (`src/hooks/pwsh.rs:134-186`, `:241-292`). |
| `tests/shell_hook_guard.rs` | Current generated-hook runtime helper still shells via `bash -lc` (`tests/shell_hook_guard.rs:35-41`) for hermeticity triage. |
| `tests/init_cli.rs` | PowerShell init surface assertions to preserve while evaluating wrapper changes (`tests/init_cli.rs:58-71`). |
| `tests/menu_cli.rs` | Existing menu fallback/PSReadLine contract tests that must continue to pass (`tests/menu_cli.rs:169-193`, `:473-544`). |
| `src/common/mod.rs` | Known pre-existing clippy lint anchor (`src/common/mod.rs:104-105`) to classify as in-scope fix vs explicit deferral. |
| `Cargo.toml` | Confirms no new runtime dependencies should be added in this phase (`Cargo.toml:8-16`). |
| `tech-docs/shell-hook-guarding.md` | Baseline fallback and menu behavior contract to preserve (`tech-docs/shell-hook-guarding.md:99-115`). |
| `plans/whole-repo-review-remediation/verification/shell-smoke-matrix.md` | Final phase smoke rows including ProxyCommand conditional row (`...:17-21`). |

## Implementation Steps

### Step 1: Apply bounded hygiene items only

- **What**: Implement exactly the five accepted minor hygiene items from the gated phase, or record an explicit per-item deferral rationale:
  1. tighten `Resolver.config` visibility when no external/public need remains,
  2. remove or explicitly justify the unused `ResolveMode` parameter in resolver pipeline APIs,
  3. document the shared `env_lock()` contract in `src/test_support.rs`,
  4. replace bare `.unwrap()` calls in hook marker-position tests with `expect(...)` diagnostics,
  5. extend balanced-delimiter tests to menu-enabled generated scripts (not only non-menu scripts).
- **Where**: `src/resolve/mod.rs`, `src/resolve/pipeline.rs`, `src/test_support.rs`, `src/hooks/mod.rs`.
- **Why**: Complete accepted debt without reopening broader architecture.
- **Considerations**:
  - If `ResolveMode` parameter is removed from `Resolver::resolve`, keep CLI output contract unchanged through `Resolver::execute`/`run_resolve` call flow.
  - Treat `tests/resolve_cli.rs` as explicit guardrails for default/list/json behavior (`outputs_single_absolute_path_on_success`, `list_mode_returns_candidates_for_ambiguity`, `json_mode_returns_structured_output`) whenever `ResolveMode` plumbing is adjusted.
  - Remove `_mode` only from `Resolver::resolve(...)` path if chosen; preserve actively-used mode dispatch in `Resolver::execute(...)` output formatting path.
  - Keep changes tightly local; do not broaden into resolver redesign or shell architecture refactor.
  - The pre-existing clippy finding (`src/common/mod.rs:104-105`) is **not automatically in scope**; decide explicitly in Reality Check (fix now vs defer) and keep decision bounded.

### Step 2: Run time-boxed ProxyCommand evaluation against explicit-wrapper baseline

- **What**: Compare current explicit wrapper (`src/hooks/pwsh.rs`) with a bounded prototype path using `[System.Management.Automation.ProxyCommand]::Create` only for `cd`/`Set-Location` wrapper fidelity questions.
- **Where**: Record evidence in a date-neutral artifact path: `plans/whole-repo-review-remediation/verification/proxycommand-eval-phase-4.md`; modify `src/hooks/pwsh.rs` and related tests only if gate passes.
- **Why**: User requested evidence-based decision; plan requires explicit adopt/reject outcome.
- **Considerations**:
  - **Time-box**: max 120 minutes implementation+evaluation effort or first hard blocker, whichever comes first.
  - **Scenario set (minimum)**:
    1. baseline unflagged path (`cd project`)
    2. previous-dir form (`cd -`)
    3. quoted path (`cd 'path with space'`)
    4. unsupported/unknown flags fallback behavior
    5. menu flow contract under `--menu` (`ConvertFrom-Json` + `TabCompleteNext` fallback) remains unchanged.
  - **Adopt criteria**: clear correctness win on scenario set and equal-or-better readability/maintainability with no fallback regressions.
  - **Reject criteria**: no measurable correctness win, readability degradation, or any fallback/contract regression.

### Step 3: Resolve adopt/reject decision and lock outcome

- **What**: Record decision with concrete evidence:
  - **Adopt only if** ProxyCommand improves correctness on real flagged/path edge cases and keeps readability/maintainability acceptable with no fallback regressions.
  - **Otherwise reject** and retain explicit wrapper baseline.
- **Where**: Decision must be captured in `plans/whole-repo-review-remediation/verification/proxycommand-eval-phase-4.md` and reflected in `shell-smoke-matrix.md`; optional code/test changes only if adopted.
- **Why**: Avoid speculative PowerShell complexity.
- **Considerations**:
  - Phase 2 Rust parser fix remains authoritative for menu parsing regardless of PowerShell decision.
  - If adoption occurs, require at least one named exact-target PowerShell regression test to be added and run (planned: `cargo test --test menu_cli proxycommand_adopted_preserves_psreadline_fallback -- --exact`).

### Step 4: Final smoke and artifact closeout

- **What**: Fill final matrix rows for Bash/Zsh/Fish/PowerShell generated-init + fallback contract checks, and set conditional ProxyCommand row to `Pass`, `Not Feasible`, or `Not Applicable` (if rejected).
- **Where**: `plans/whole-repo-review-remediation/verification/shell-smoke-matrix.md` rows `17-21`.
- **Why**: Plan DoD requires explicit shell verification closure.
- **Considerations**:
  - Use `Not Feasible` only with explicit environment rationale.
  - Row 21 is conditional: set `Not Applicable` if ProxyCommand is rejected.
  - Final closeout includes synchronized updates to `implementation/phase-4-impl.md`, `phases/phase-4.md`, `plan.md`, and `todo.md`.

## Testing Plan

| Test Type | What to Test | Expected Outcome |
|-----------|-------------|-----------------|
| Unit/contract | Resolver API hygiene and hook-template hygiene updates (visibility/signature cleanup, `expect(...)` diagnostics, menu-enabled delimiter checks) | Targeted tests pass and unchanged resolver/shell contracts remain green |
| Integration | Generated hook/runtime contracts for Bash/Zsh/Fish/PowerShell menu fallback (`dx init <shell> --menu`) | Hook scripts preserve documented `dx menu` fallback behavior and no invalid JSON/wrapper regressions appear |
| Conditional integration (PowerShell) | If ProxyCommand is adopted, validate PSReadLine Tab handler path and `ConvertFrom-Json`/`TabCompleteNext` fallback parity | Adopted path matches or improves baseline behavior with no regressions |
| Shell smoke | Final Phase 4 matrix scenarios | Rows 17-21 completed with concrete evidence and rationale |

**Verify command (baseline):** `cargo test --test resolve_cli outputs_single_absolute_path_on_success -- --exact && cargo test --test resolve_cli list_mode_returns_candidates_for_ambiguity -- --exact && cargo test --test resolve_cli json_mode_returns_structured_output -- --exact && cargo test --lib hooks::tests::menu_enabled_scripts_keep_cross_shell_menu_invocation_marker -- --exact && cargo test --lib hooks::tests::generated_scripts_do_not_leak_internal_placeholder_tokens -- --exact && cargo test --test menu_cli hook_scripts_contain_fallback_on_noop -- --exact && cargo test --test menu_cli init_pwsh_with_menu_flag_includes_psreadline_handler -- --exact && cargo test`

**Verify command (conditional if ProxyCommand adopted):** `cargo test --test menu_cli proxycommand_adopted_preserves_psreadline_fallback -- --exact && cargo test`

### Test Integrity Constraints

- Do not weaken existing cross-shell contract tests in `src/hooks/mod.rs`; additive assertions only.
- If `ResolveMode` cleanup changes signatures, existing resolve behavior and call-site semantics must remain equivalent.
- If `ResolveMode` plumbing is touched, exact-target `tests/resolve_cli.rs` checks for default/list/json behavior must pass as behavioral guardrails.
- If ProxyCommand is adopted, PowerShell fallback (`TabCompleteNext`) and JSON parsing (`ConvertFrom-Json`) contract tests must remain or be strengthened.
- If ProxyCommand is adopted, include at least one named exact-target regression test for the adopted wrapper path (planned target: `proxycommand_adopted_preserves_psreadline_fallback` in `tests/menu_cli.rs`).
- If ProxyCommand is rejected, no behavior change should be introduced in `src/hooks/pwsh.rs` beyond bounded hygiene updates.
- Avoid filter false-passes by using explicit `--test <target> <test_name> -- --exact` form for targeted integration checks.

## Rollback Strategy

Rollback Phase 4 as bounded slices: revert hygiene changes independently from PowerShell evaluation outcome. If ProxyCommand adoption regresses shell behavior, revert to explicit wrapper baseline in one change and keep decision recorded as rejected/deferred.

## Open Decisions

| Decision | Options | Chosen | Rationale |
|----------|---------|--------|-----------|
| `Resolver.config` visibility | Keep `pub` vs tighten (`pub(crate)`/private) | **Tighten to `pub(crate)`** | Implemented in `src/resolve/mod.rs`; no external/public need remained. |
| Unused `ResolveMode` parameter | Keep dormant vs remove/plumb real mode behavior | **Removed from `Resolver::resolve`; enum retained where used** | Dead resolver-pipeline parameter was removed in `src/resolve/pipeline.rs`; `ResolveMode` remains actively used in `src/resolve/output.rs` mode-aware output/error handling. |
| ProxyCommand strategy | Adopt vs reject | **Rejected after evaluation** | `verification/proxycommand-eval-phase-4.md` showed no net correctness win and unsupported-flag ambiguity risk; explicit wrapper baseline preserved. |
| ProxyCommand baseline reference | Abstract ideal vs concrete current wrapper | Concrete current wrapper | Phase requires comparison against existing `src/hooks/pwsh.rs` behavior. |
| Pre-existing `clippy::let_and_return` (`src/common/mod.rs:104-105`) | Fix in Phase 4 vs explicitly defer | **Deferred** | Not part of the bounded accepted hygiene list; avoid silent scope expansion. |

## Reality Check

### Code Anchors Used

| File | Symbol/Area | Why it matters |
|------|-------------|----------------|
| `src/resolve/mod.rs:50-52` | `Resolver { pub(crate) config: AppConfig, ... }` | Confirms accepted visibility tightening landed. |
| `src/resolve/pipeline.rs:8-82` | `resolve(&self, query: ResolveQuery<'_>)` | Confirms dead `_mode: ResolveMode` parameter was removed from resolver pipeline API. |
| `src/resolve/output.rs:14-59` | `Resolver::execute` + mode-aware output/error path | Confirms `ResolveMode` remains actively used where behavior differs by mode. |
| `src/cli/resolve.rs:3-12` | `run_resolve` mode mapping | Meaningful external call site for `ResolveMode` behavior. |
| `tests/resolve_cli.rs:27-108` | resolve default/list/json CLI tests | Behavioral guardrails if `ResolveMode` resolver plumbing is adjusted. |
| `src/test_support.rs:1-11` | `env_lock()` helper + contract doc comment | Confirms shared global lock contract is now documented for env-mutating tests across modules. |
| `src/hooks/mod.rs:122-136` | ordering-position marker assertions | Confirms `.find(...)` assertions use diagnostic `expect(...)` instead of bare `unwrap()`. |
| `src/hooks/mod.rs:369-412` | balanced-delimiter + menu-enabled delimiter tests | Confirms menu-enabled balanced-delimiter coverage exists for Bash/Zsh/Fish/Pwsh. |
| `src/hooks/pwsh.rs:241-292` | PSReadLine Tab handler + `ConvertFrom-Json` + `TabCompleteNext` | Defines baseline PowerShell wrapper behavior that evaluation must preserve. |
| `tests/shell_hook_guard.rs:35-43` | `run_shell` uses `bash -lc` / `zsh -fc` | Confirms reviewer hermeticity follow-up is real and nearby but not explicitly gated in accepted Phase 4 hygiene list. |
| `tech-docs/shell-hook-guarding.md:55-56` | legacy prototype wording | Confirms docs now state non-authoritative status; wording polish is optional follow-up, not a Phase 4 blocker. |
| `src/common/mod.rs:104-105` | `let result = operation(); result` | Confirms the currently failing pre-existing clippy lint location called out in plan history. |
| `plans/whole-repo-review-remediation/verification/shell-smoke-matrix.md:17-21` | final phase shell scenarios | Required closure artifact for phase completion. |

### Mismatches / Notes

- Menu-enabled delimiter parity is now implemented: `src/hooks/mod.rs` includes balanced-brace/quote tests for Bash/Zsh/Fish/Pwsh `build_menu_script(...)` outputs.
- `ProxyCommand` remains unused in repository by explicit Phase 4 decision; rejection evidence is captured in `../verification/proxycommand-eval-phase-4.md`.
- Reviewer follow-up on Bash helper hermeticity (`tests/shell_hook_guard.rs` using `bash -lc`) is **deferred by default** as out-of-scope unless required to keep newly changed Phase 4 tests stable.
- Reviewer follow-up on `tech-docs/shell-hook-guarding.md` wording is **deferred** from implementation scope; if adjusted later, it should be handled as documentation maintenance outside this bounded implementation plan.
- `cargo clippy --all-targets -- -D warnings` pre-existing `clippy::let_and_return` (`src/common/mod.rs:104-105`) remains explicitly deferred from this bounded phase.

## Execution Outcome

- **Status:** Completed.
- **Accepted hygiene items landed:**
  1. `Resolver.config` visibility tightened to `pub(crate)` in `src/resolve/mod.rs`.
  2. Dead `ResolveMode` parameter removed from `Resolver::resolve` in `src/resolve/pipeline.rs`.
  3. Shared `env_lock()` contract documented in `src/test_support.rs`.
  4. Hook marker test `.unwrap()` sites replaced with diagnostic `expect(...)` in `src/hooks/mod.rs`.
  5. Menu-enabled balanced-delimiter tests added for Bash/Zsh/Fish/Pwsh in `src/hooks/mod.rs`.
- **PowerShell decision outcome:** `ProxyCommand` evaluation completed and **rejected**; explicit wrapper baseline in `src/hooks/pwsh.rs` retained. Decision/evidence recorded in `../verification/proxycommand-eval-phase-4.md`.
- **Verification evidence captured:**
  - Baseline verify chain completed with full-suite pass (`cargo test`: `293 passed`).
  - Shell smoke matrix Phase 4 rows 17-20 marked `Pass`; conditional row 21 marked `Not Applicable` after reject decision (`../verification/shell-smoke-matrix.md`).
  - Targeted tests for resolve output guardrails and hook/menu marker/fallback contracts passed as listed in the verify command.
- **Deferred non-blocking notes retained:**
  - Bash helper hermeticity follow-up in `tests/shell_hook_guard.rs`.
  - Optional wording polish in `tech-docs/shell-hook-guarding.md`.
  - Pre-existing `clippy::let_and_return` in `src/common/mod.rs` (outside bounded Phase 4 scope).
- **Final review/collation record:** `../reviews/impl-review-phase-4-collated-2026-04-19.md`.
