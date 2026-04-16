---
type: planning
entity: implementation-plan
plan: "whole-repo-review-remediation"
phase: 3
status: completed
created: "2026-04-16"
updated: "2026-04-17"
---

# Implementation Plan: Phase 3 - Retire Legacy Hook Prototypes

> Implements [Phase 3](../phases/phase-3.md) of [whole-repo-review-remediation](../plan.md)

## Approach

First migrate shell-hook guard assertions off `scripts/hooks/*` and onto generated-hook outputs using the authoritative runtime surfaces:

- `dx init bash --command-not-found`
- `dx init zsh --command-not-found`

Coverage equivalence is defined against generated-hook + current docs/contracts (not prototype quirks). Explicitly acknowledge known heuristic divergence between legacy bash/zsh prototypes and generated hooks, then preserve coverage for intended behavior only. Deletion proceeds only after replacement runtime + generated-hook contract coverage is green.

## Affected Modules

| Module | Change Type | Description |
|--------|-------------|-------------|
| `tests` | modify | Rework shell-hook guard tests to validate generated hooks instead of sourcing legacy prototypes. |
| `src/hooks` | modify (tests only) | Add/adjust generated-hook contract tests to cover guard/fallback semantics needed for prototype deletion confidence. |
| `scripts/hooks` | delete | Remove `dx.bash` and `dx.zsh` prototypes after replacement coverage gate passes. |
| `plans/whole-repo-review-remediation/verification` | modify | Record generated-hook authority transition evidence in shell smoke matrix. |

## Required Context

| File | Why |
|------|-----|
| `tests/shell_hook_guard.rs` | Current tests hard-source `scripts/hooks/dx.bash` and must be migrated (`tests/shell_hook_guard.rs:20-23`, `:46-49`, `:88-90`). |
| `src/hooks/mod.rs` | Existing generated-script contract tests and marker assertions to extend for parity (`src/hooks/mod.rs:295-366`, `:401-413`). |
| `src/cli/mod.rs` | Confirms `dx init` exposes distinct `--command-not-found` and `--menu` flags (`src/cli/mod.rs:28-34`). |
| `src/cli/init.rs` | Entrypoint for generated hook output used by replacement runtime tests (`src/cli/init.rs:3-14`). |
| `tests/init_cli.rs` | Existing proof that `--command-not-found` toggles handler emission (`tests/init_cli.rs:86-108`). |
| `src/hooks/bash.rs` | Generated Bash guard/wrapper behavior source of truth (`src/hooks/bash.rs:13-25`, `:315-347`). |
| `src/hooks/zsh.rs` | Generated Zsh command-not-found/menu fallback behavior for Zsh deletion rationale (`src/hooks/zsh.rs:13-25`, `:309-341`). |
| `tech-docs/shell-hook-guarding.md` | Declares generated hooks authoritative and legacy scripts non-authoritative (`tech-docs/shell-hook-guarding.md:46-56`). |
| `plans/whole-repo-review-remediation/verification/shell-smoke-matrix.md` | Defines required Phase 3 Bash/Zsh smoke evidence rows (`...:14-15`). |

## Implementation Steps

### Step 1: Define and implement replacement coverage equivalence gate

- **What**: Translate each behavior currently asserted via legacy sourced hook tests into generated-hook runtime assertions built from `dx init <shell> --command-not-found` output before any file deletion.
- **Where**: `tests/shell_hook_guard.rs` and/or `tests/menu_cli.rs` + `src/hooks/mod.rs` generated-script contract tests.
- **Why**: Phase gate requires equivalence, not just moving tests.
- **Considerations**:
  - Runtime tests should evaluate generated scripts and mock `dx` deterministically (prefer temporary PATH shim executable; function-export approach acceptable if robustly scoped).
  - Equivalence baseline is generated-hook authoritative behavior. Do **not** preserve legacy-only heuristic quirks (e.g., prototype dot-containing vs generated dot-prefix path-like matching) unless separately justified.
  - Enumerate legacy test disposition explicitly:
    1. `bash_hook_guard_prevents_recursive_resolve_calls` → migrate to generated-hook runtime test.
    2. `bash_hook_resolves_path_like_command_once` → migrate to generated-hook runtime test.
    3. `bash_cd_wrapper_invokes_dx_once_and_changes_directory` → migrate to generated-hook runtime test.

### Step 2: Cover Bash and Zsh deletion rationale explicitly

- **What**: Ensure Bash replacement tests exercise behavior previously checked via prototype sourcing; for Zsh, rely on existing generated-hook contract tests in `src/hooks/mod.rs` where sufficient, and add only low-cost supplemental assertions if a concrete gap remains.
- **Where**: `src/hooks/mod.rs` marker tests + shell smoke matrix evidence rows for Bash and Zsh.
- **Why**: User-required deletion rationale must include both shells.
- **Considerations**: If Zsh runtime smoke cannot run in environment, provide `Not Feasible` with concrete rationale and rely on generated-hook contract assertions.

### Step 3: Delete legacy prototypes once coverage gate is green

- **What**: Remove deletion targets after replacement coverage passes:
  - `scripts/hooks/dx.bash`
  - `scripts/hooks/dx.zsh`
  - legacy test implementations in `tests/shell_hook_guard.rs` that source `scripts/hooks/*` via absolute path
  and eliminate any remaining test/doc references that imply authority.
- **Where**: `scripts/hooks/dx.bash`, `scripts/hooks/dx.zsh`, impacted tests under `tests/`.
- **Why**: Align repository artifacts with documented source of truth.
- **Considerations**: Deletion is blocked until replacement coverage and phase verify command pass.

### Step 4: Update smoke matrix for authority transition

- **What**: Fill Phase 3 matrix rows with pass/evidence for Bash generated-hook replacement and Zsh deletion safety rationale.
- **Where**: `plans/whole-repo-review-remediation/verification/shell-smoke-matrix.md` rows `:14-15`.
- **Why**: Plan DoD requires shell-facing evidence for transition.
- **Considerations**: Keep evidence pointers concrete (test names/commands and shell transcript notes where available).

## Testing Plan

| Test Type | What to Test | Expected Outcome |
|-----------|-------------|-----------------|
| Integration/runtime | Generated `dx init bash --command-not-found` and `dx init zsh --command-not-found` guard behavior previously covered by legacy prototype source tests | Same guard outcomes validated without `scripts/hooks/*` |
| Contract (Zsh + Bash) | Generated script markers for recursion guard, path-like heuristic, fallback, and wrapper contract | Assertions pass for both Bash and Zsh generated output |
| Integration (init CLI) | `dx init zsh --command-not-found` emits handler wiring in CLI surface | Handler inclusion assertion passes and matches generated-hook authority model |
| Smoke evidence | Phase 3 matrix scenarios | Bash and Zsh rows updated with `Pass`/`Not Feasible` plus rationale |

**Verify command:** `cargo test --test shell_hook_guard -- --list | rg -q 'bash_generated_hook_command_not_found_guard_prevents_recursive_resolve_calls' && cargo test --test shell_hook_guard -- --list | rg -q 'bash_generated_hook_command_not_found_resolves_path_like_command_once' && cargo test --test shell_hook_guard -- --list | rg -q 'bash_generated_hook_cd_wrapper_invokes_dx_once_and_changes_directory' && cargo test --test shell_hook_guard -- --list | rg -q 'zsh_generated_hook_command_not_found_guard_prevents_recursive_resolve_calls' && cargo test --test shell_hook_guard -- --list | rg -q 'zsh_generated_hook_command_not_found_resolves_path_like_command_once' && cargo test --test init_cli -- --list | rg -q 'init_zsh_with_command_not_found_flag_includes_handler' && cargo test --lib -- --list | rg -q 'hooks::tests::all_shells_freeze_command_not_found_guard_contract_markers' && cargo test --test shell_hook_guard bash_generated_hook_command_not_found_guard_prevents_recursive_resolve_calls -- --exact && cargo test --test shell_hook_guard bash_generated_hook_command_not_found_resolves_path_like_command_once -- --exact && cargo test --test shell_hook_guard bash_generated_hook_cd_wrapper_invokes_dx_once_and_changes_directory -- --exact && cargo test --test shell_hook_guard zsh_generated_hook_command_not_found_guard_prevents_recursive_resolve_calls -- --exact && cargo test --test shell_hook_guard zsh_generated_hook_command_not_found_resolves_path_like_command_once -- --exact && cargo test --test init_cli init_zsh_with_command_not_found_flag_includes_handler -- --exact && cargo test --lib hooks::tests::all_shells_freeze_command_not_found_guard_contract_markers -- --exact && cargo test`

### Test Integrity Constraints

- Existing cross-shell generated-script contract tests in `src/hooks/mod.rs` must remain active; do not replace robust assertions with weaker string checks.
- No test may source `scripts/hooks/*` after migration.
- Bash behavior parity must be demonstrated before deleting prototype files; deletion cannot precede coverage replacement.
- Zsh deletion rationale must be evidenced by generated-hook coverage and/or smoke evidence, not by assumption.

## Rollback Strategy

If generated-hook replacement coverage fails or reveals a gap, restore prototype files only with a documented blocker and keep deletion deferred; do not leave repository in mixed state where prototypes are deleted but equivalent generated-hook coverage is absent.

## Open Decisions

| Decision | Options | Chosen | Rationale |
|----------|---------|--------|-----------|
| Legacy deletion sequencing | Delete first vs coverage-first | Coverage-first | Required by phase prerequisite and minimizes safety-net regression risk. |
| Bash equivalence proof style | Runtime sourced generated script vs pure static markers | Runtime + marker mix | Runtime tests prove behavior; markers anchor cross-shell structural contracts. |
| Zsh deletion rationale evidence | Require full runtime smoke always vs allow generated-test + documented feasibility notes | Generated-test + smoke when feasible | Keeps phase executable in constrained environments while still evidence-driven. |
| Equivalence baseline | Preserve prototype quirks vs generated-hook authoritative behavior | Generated-hook authoritative behavior | Aligns with `tech-docs/shell-hook-guarding.md` source-of-truth and avoids encoding known prototype/generator divergence. |

## Reality Check

### Code Anchors Used

| File | Symbol/Area | Why it matters |
|------|-------------|----------------|
| `tests/shell_hook_guard.rs:20-23` | hardcoded `scripts/hooks/dx.bash` source path | Confirms current safety net depends on legacy prototype. |
| `scripts/hooks/dx.bash:27-53` | legacy `command_not_found_handle` behavior | Defines currently asserted guard/wrapper behaviors to migrate off prototype sourcing. |
| `scripts/hooks/dx.zsh:3-53` | legacy Zsh prototype | Confirms second prototype file exists and must be addressed in deletion rationale. |
| `src/hooks/mod.rs:334-366` | command-not-found guard contract markers | Generated-hook contract baseline for Bash/Zsh/Fish/Pwsh. |
| `src/cli/mod.rs:28-34` + `src/cli/init.rs:3-14` | `dx init` flag model and script generation path | Grounds requirement to use `--command-not-found` in replacement runtime tests. |
| `tests/init_cli.rs:86-108` | CLI test anchors for handler inclusion/exclusion | Confirms authoritative runtime generation mode for guard behavior. |
| `tech-docs/shell-hook-guarding.md:46-56` | source-of-truth statement | Documentation baseline that this phase aligns to. |

### Mismatches / Notes

- Known divergence exists between legacy bash/zsh prototypes and generated hooks in path-like heuristic details; coverage equivalence is framed against generated-hook authoritative behavior and documented contracts, not wholesale prototype mimicry.

## Execution Outcome

- **Status:** Completed.
- **Main implementation landed in:**
  - `tests/shell_hook_guard.rs`
  - `tests/init_cli.rs`
  - `src/hooks/mod.rs` (existing generated-hook contract test retained and re-verified)
  - `plans/whole-repo-review-remediation/verification/shell-smoke-matrix.md`
- **Prototype retirement outcome:**
  - `scripts/hooks/dx.bash` deleted
  - `scripts/hooks/dx.zsh` deleted
  - `scripts/hooks/` now contains no authoritative hook prototype files
- **Verification evidence captured:**
  - `command -v zsh` resolved to `/etc/profiles/per-user/nick/bin/zsh`
  - exact runtime tests passed for 5 generated-hook scenarios in `tests/shell_hook_guard.rs` (3 Bash + 2 Zsh)
  - exact CLI test passed: `init_zsh_with_command_not_found_flag_includes_handler`
  - exact unit test passed with corrected invocation: `cargo test --lib hooks::tests::all_shells_freeze_command_not_found_guard_contract_markers -- --exact`
  - full `cargo test` passed (`289 passed`)
- **Final acceptance record:** `../reviews/impl-review-phase-3-collated-2026-04-17.md`
- **Carry-forward verification context (non-Phase-3 blocker):** `cargo clippy --all-targets -- -D warnings` currently fails on pre-existing `clippy::let_and_return` in `src/common/mod.rs:104-105` outside the Phase 3 diff; track resolution in later-phase hygiene closeout.
- **Approach deviation:** None required; execution remained aligned with the approved implementation plan.
