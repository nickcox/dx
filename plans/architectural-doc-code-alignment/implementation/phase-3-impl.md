---
type: planning
entity: implementation-plan
plan: "architectural-doc-code-alignment"
phase: 3
status: completed
created: "2026-04-08"
updated: "2026-04-08"
---

# Implementation Plan: Phase 3 - Verify Across Shells and Finalize Documentation

> Implements [Phase 3](../phases/phase-3.md) of [architectural-doc-code-alignment](../plan.md)

## Approach

Phase 3 is a verification/finalization pass, not a new feature phase: validate that Phase 1 contract decisions and Phase 2 hook/menu hardening are actually reflected in runtime behavior and docs across Bash, Zsh, Fish, and PowerShell. The JSON action protocol remains fixed for this phase (`noop`/`replace` with `replaceStart`/`replaceEnd`/`value`), and completion criteria focus on trustworthy evidence capture (automated tests + shell smoke matrix) and final doc/plan state cleanup.

## Affected Modules

| Module | Change Type | Description |
|--------|-------------|-------------|
| `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` | modify (execution artifact) | Record per-shell scenario status/evidence from reproducible smoke runs. |
| `plans/architectural-doc-code-alignment/phases/phase-2.md` | modify (prerequisite artifact) | Ensure Phase 2 status reflects completed baseline before Phase 3 closure claims. |
| `plans/architectural-doc-code-alignment/todo.md` | modify (closure artifact) | Reconcile Phase 2 stale items and record Phase 3 verification completion/deferments. |
| `docs/shell-hook-guarding.md` | modify (conditional) | Apply only verification-driven wording fixes if observed behavior differs from current text. |
| `docs/cd-extras-cli-prd.md` | modify (conditional) | Apply only verification-driven corrections to current-contract claims. |
| `docs/configuration.md` | modify (conditional) | Update only if smoke verification reveals user-visible config/flag behavior drift. |
| `plans/architectural-doc-code-alignment/plan.md` and phase artifacts | modify (execution artifact) | Mark completion/deferments and capture residual follow-up items after verification closes. |

## Required Context

| File | Why |
|------|-----|
| `plans/architectural-doc-code-alignment/plan.md` | Global DoD and verification bar (automated tests + four-shell smoke coverage). |
| `plans/architectural-doc-code-alignment/phases/phase-3.md` | Gated scope and acceptance criteria for this phase. |
| `plans/architectural-doc-code-alignment/phases/phase-2.md` | Prerequisite status source; must not remain stale/contradictory when Phase 3 starts/closes. |
| `plans/architectural-doc-code-alignment/todo.md` | Cross-phase execution truth for pending/completed items; required for prerequisite and closure hygiene. |
| `plans/architectural-doc-code-alignment/implementation/phase-2-impl.md` | Continuity for what behavior was intentionally changed in Phase 2. |
| `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` | Authoritative C4/C5 target contract to verify against real shell behavior. |
| `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` | Required checklist and evidence sink for Bash/Zsh/Fish/PowerShell smoke runs. |
| `docs/shell-hook-guarding.md` | Expected shell runtime contract wording to confirm (or correct) from smoke evidence. |
| `docs/cd-extras-cli-prd.md` | Current command/architecture contract wording that must remain reality-based. |
| `docs/configuration.md` | Env flag semantics (`DX_MENU`, `DX_MENU_DEBUG`, menu limits) cross-check target. |
| `src/cli/menu.rs` | Rust-owned menu action behavior for selected/cancelled/no-TTY paths. |
| `src/menu/action.rs` | JSON protocol shape that must remain unchanged this phase. |
| `src/hooks/{bash,zsh,fish,pwsh}.rs` | Shell fallback and replace-application behavior to validate in smoke runs. |
| `tests/menu_cli.rs` | Cross-shell generated-hook/menu contract tests anchoring changed behavior. |
| `tests/init_cli.rs` | Init generation sanity for all supported shells. |

## Implementation Steps

### Step 1: Freeze verification baseline and enforce Phase 2 prerequisite gate

- **What**: Run a concrete preflight gate before any Step 2+ work: (a) confirm scope boundaries (verification/finalization only), (b) confirm JSON protocol remains frozen (`action`, `replaceStart`, `replaceEnd`, `value`), and (c) confirm Phase 2 baseline artifacts are coherent (`phases/phase-2.md`, `todo.md`, and `implementation/phase-2-impl.md` do not contradict a complete/stable Phase 2 baseline).
- **Where**: `plans/.../phase-3.md`, `plans/.../phases/phase-2.md`, `plans/.../todo.md`, `plans/.../implementation/phase-2-impl.md`, `plans/.../contracts/phase-1-conflict-inventory.md`, `src/menu/action.rs`.
- **Why**: Prevent scope creep into protocol redesign or unrelated feature work.
- **Considerations**: If stale prerequisite statuses are found (for example Phase 2 still marked active/pending in planning artifacts), pause shell-smoke/finalization, reconcile artifact status first using existing Phase 2 implementation evidence, then continue; if reconciliation cannot be justified from evidence, record an explicit blocker and do not claim Phase 3 closure.

### Step 2: Execute automated verification focused on changed menu/hook behavior

- **What**: Run the primary automated verification command, then run broader regression (`cargo test`) to satisfy phase acceptance.
- **Where**: Test targets in `tests/menu_cli.rs` and whole test suite.
- **Why**: `menu_cli` directly exercises cross-shell init script generation and menu action/fallback contract assertions for the behavior changed by this plan.
- **Considerations**: `tests/menu_cli.rs` and `tests/init_cli.rs` are verification baselines in this phase; keep them unchanged unless a blocker-fix behavior change requires test updates. Avoid filter false-passes by ensuring targeted commands execute intended tests (use exact target form when slicing individual tests).

### Step 3: Run and complete the four-shell smoke matrix

- **What**: Execute each matrix scenario row for Bash, Zsh, Fish, and PowerShell (init usage, menu disabled, successful replace, cancel with typed query, noop/error fallback, no-TTY/degraded where feasible) using reproducible trigger/observation rules, and record concise evidence in `plans/.../verification/shell-smoke-matrix.md`.
- **Where**: Real shell sessions + `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md`.
- **Why**: Automated tests prove generated contract structure; shell smokes prove end-to-end behavior in actual interactive environments.
- **Considerations**: Verify both non-menu init and `--menu` init paths for every shell. PowerShell must use single-script-block evaluation for both (`Invoke-Expression ((& dx init pwsh | Out-String))` and `Invoke-Expression ((& dx init pwsh --menu | Out-String))`). If a degraded path is not feasible in an environment, record `Not Feasible` with reason instead of implying pass.

### Step 4: Finalize docs and plan state from observed evidence

- **What**: Apply only verification-driven doc corrections and update plan/phase status artifacts to completed or intentionally deferred with rationale.
- **Where**: `docs/shell-hook-guarding.md`, `docs/cd-extras-cli-prd.md`, `docs/configuration.md` (conditional), `plans/.../plan.md`, and related phase/todo artifacts.
- **Why**: Close the plan with evidence-backed documentation rather than inferred behavior.
- **Considerations**: Any residual mismatch must be explicitly captured with owner/next step; do not leave ambiguous “pending” statements once phase closure is claimed.

## Testing Plan

Primary verify command (meaningfully exercises changed cross-shell menu/hook behavior):

```bash
cargo test --test menu_cli
```

| Test Type | What to Test | Expected Outcome |
|-----------|-------------|-----------------|
| Targeted integration (primary) | `tests/menu_cli.rs` contract checks for noop/replace JSON behavior and shell hook generation fallback/replace handling across Bash/Zsh/Fish/PowerShell. | All `menu_cli` tests pass; failures pinpoint contract drift in changed behavior surface. |
| Full automated regression | Full test suite after targeted pass. | `cargo test` passes with no disabled/weakened existing tests. |
| Shell smoke matrix | Execute every row in `plans/.../verification/shell-smoke-matrix.md` with concise evidence notes. | Each row is `Pass`, `Fail`, or `Not Feasible` with reproducible command/observation note; no silent `Pending` rows at phase close. |
| PowerShell-specific smoke | Validate single-script-block init and PSReadLine-driven menu path. | `Invoke-Expression ((& dx init pwsh --menu | Out-String))` works; fallback/degraded notes recorded when environment limits apply. |

### Concrete Shell-Smoke Execution Steps

1. Verify **non-menu init** in a clean session for each shell:
   - Bash: `eval "$(dx init bash)"`
   - Zsh: `eval "$(dx init zsh)"`
   - Fish: `dx init fish | source`
   - PowerShell: `Invoke-Expression ((& dx init pwsh | Out-String))`
2. Verify **menu-enabled init** in a separate clean session for each shell:
   - Bash: `eval "$(dx init bash --menu)"`
   - Zsh: `eval "$(dx init zsh --menu)"`
   - Fish: `dx init fish --menu | source`
   - PowerShell: `Invoke-Expression ((& dx init pwsh --menu | Out-String))`
3. Execute required interactive scenarios with this evidence format: `shell=<name>; scenario=<name>; trigger=<command+keys>; observed=<buffer/result>; status=<Pass|Fail|Not Feasible>`.
   - **Successful replace**: Trigger menu on a partial buffer (e.g., `cd <partial>` then completion trigger), select a concrete candidate; expected observation: buffer/path is replaced to selected candidate (replace action applied).
   - **Cancel-with-query-change**: Trigger menu, type additional query text, then cancel (`Esc`/cancel key); expected observation: buffer retains typed refinement (query change preserved) and no fallback completion overwrites it.
   - **Noop/error fallback**: Trigger menu where cancel/noop/error path occurs (cancel immediately or force menu failure condition); expected observation: shell falls back to native completion path (no broken prompt, no malformed replacement).
   - **No-TTY/degraded path**: Use a local non-interactive approximation (example: `dx menu --buffer "cd foo" --cursor 6 </dev/null`), then trigger completion flow; expected observation: graceful noop/fallback behavior without terminal-state corruption.
4. Execute menu-disabled scenario with `DX_MENU=0` in each shell and record fallback-native completion behavior.
5. Update matrix status/evidence immediately after each run; no scenario remains implicitly passed.

### Test Integrity Constraints

- Do not disable, delete, or weaken existing tests to obtain green results.
- Treat `tests/menu_cli.rs` and `tests/init_cli.rs` as read-only verification baselines in this phase; update only if a justified blocker-fix behavior change requires it.
- Treat `cargo test` filter usage carefully: a passing filtered command can run zero tests; when slicing a single test, use exact target form (for example `cargo test --test menu_cli hook_scripts_contain_fallback_on_noop -- --exact`).
- Keep the menu JSON protocol frozen in this phase (`action`, `replaceStart`, `replaceEnd`, `value`); verification/finalization must not introduce field additions, renames, or semantic redesign.
- Maintain cross-shell verification parity: Bash/Zsh/Fish/PowerShell must all be represented in both automated assertions and smoke evidence.
- Matrix evidence must reflect actual execution outcomes; unknown/unrun paths stay `Not Run`/`Not Feasible` (never implied pass).

## Rollback Strategy

If Phase 3 verification reveals regressions introduced by this plan, rollback in this order: (1) revert any late-stage behavior changes made while fixing blockers, (2) rerun `cargo test --test menu_cli` and then full `cargo test` to confirm baseline restoration, (3) reset matrix/doc status lines to match restored behavior, and (4) reopen unresolved items as explicit follow-up tasks instead of forcing closure.

## Open Decisions

| Decision | Options | Chosen | Rationale |
|----------|---------|--------|-----------|
| Handling environment-limited degraded-path smoke scenarios | A) Block phase completion until all degraded cases are executable locally, B) Allow `Not Feasible` with explicit reason/evidence and keep phase closure transparent | B | Preserves verification integrity while acknowledging shell/terminal capability differences, especially for PowerShell/PSReadLine and no-TTY paths. |

## Reality Check

### Code Anchors Used

| File | Symbol/Area | Why it matters |
|------|-------------|----------------|
| `src/menu/action.rs:3-19` | `MenuAction` enum (`noop`/`replace`) | Confirms protocol is already explicit and should remain unchanged in Phase 3. |
| `src/cli/menu.rs:220-281` | `run_menu` select/cancel/no-TTY branches | Anchors expected `replace` vs `noop` runtime outcomes used by smoke scenarios. |
| `src/hooks/bash.rs:304-407` | `__dx_try_menu` + `_dx_menu_wrapper` | Bash fallback and replace extraction contract under verification. |
| `src/hooks/zsh.rs:288-381` | `__dx_menu_widget` | Zsh fallback and replace application path to validate for C4 parity. |
| `src/hooks/fish.rs:220-302` | `__dx_menu_complete` | Fish fallback behavior and JSON extraction path to validate under smoke. |
| `src/hooks/pwsh.rs:291-349` | PSReadLine Tab handler + `ConvertFrom-Json` | PowerShell structured parsing and fallback expectations, including PSReadLine dependence. |
| `tests/menu_cli.rs:305-344` | `hook_scripts_contain_fallback_on_noop` | Existing automated assertion for cross-shell noop/error fallback structure. |
| `tests/menu_cli.rs:420-449` | `hook_scripts_apply_replace_action_contract` | Existing automated assertion for replace protocol consumption across shells. |
| `plans/.../verification/shell-smoke-matrix.md:7-32` | Matrix rows (finalized Pass/Not Feasible) | Confirms Phase 3 execution/evidence completion for all shells/scenarios with explicit feasibility notes. |

### Mismatches / Notes

- Automated coverage for generated hook contracts (`tests/menu_cli.rs`) was retained and passed; shell-smoke rows were finalized with explicit `Pass`/`Not Feasible` outcomes.
- Phase 2 artifact hygiene gate was reconciled before closure (`phase-2.md` and `phase-2-impl.md` updated to completed).
- PowerShell degraded/no-TTY behavior remained environment-sensitive due to PSReadLine and host terminal capabilities; explicit `Not Feasible` notes were recorded where interactive repro was unavailable.
- Final plan artifacts and matrix entries now reflect verified outcomes only.

## Verification Outcomes

- Automated tests recorded in this phase:
  - `cargo test --test menu_cli` => 20 passed (1 suite)
  - `cargo test` => 263 passed (13 suites)
  - `cargo test --test init_cli` => 7 passed
  - `cargo test key_event_mapping_` => 4 passed
  - `cargo test --test menu_cli hook_scripts_contain_fallback_on_noop -- --exact` => 1 passed
  - `cargo test --test menu_cli hook_scripts_apply_replace_action_contract -- --exact` => 1 passed
