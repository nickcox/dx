---
type: planning
entity: implementation-plan
plan: "architectural-doc-code-alignment"
phase: 2
status: completed
created: "2026-04-08"
updated: "2026-04-08"
---

# Implementation Plan: Phase 2 - Harden Cross-Shell Menu and Hook Boundaries

> Implements [Phase 2](../phases/phase-2.md) of [architectural-doc-code-alignment](../plan.md)

## Approach

Phase 2 is a code-hardening pass that implements the Phase 1 approved C4/C5 boundary contract in hook generators and menu payload handling while preserving the thin-wrapper architecture: shells perform local buffer/cd mutations and Rust owns selector resolution and menu action semantics. The core implementation focus is to converge noop/error fallback behavior across Bash/Zsh/Fish/PowerShell and make shell-side replace parsing safer without introducing new required dependencies.

Supporting docs are updated only where implementation details need clarification (not broad architecture rewrites), and Phase 3 remains responsible for recording full shell smoke results.

## Affected Modules

| Module | Change Type | Description |
|--------|-------------|-------------|
| `src/hooks/bash.rs` | modify | Harden replace payload extraction and preserve dependency-free fallback path. |
| `src/hooks/zsh.rs` | modify | Align noop/error fallback with approved native-completion-equivalent behavior and harden replace parsing. |
| `src/hooks/fish.rs` | modify | Harden JSON field extraction for replace actions while preserving `commandline -f complete` fallback. |
| `src/hooks/pwsh.rs` | modify | Keep structured JSON parsing path (`ConvertFrom-Json`) and confirm parity fallback behavior. |
| `src/cli/menu.rs` | modify | Keep stdout action contract stable (`noop`/`replace`) and ensure cancel/no-TTY semantics stay aligned with approved C4 behavior. |
| `src/menu/action.rs` | modify (if needed) | Maintain/extend contract tests for action field schema (`action`, `replaceStart`, `replaceEnd`, `value`). |
| `tests/menu_cli.rs` | modify | Add/adjust generator contract assertions for C4 parity and safer C5 parsing expectations. |
| `tests/init_cli.rs` | modify (if needed) | Keep shell-init generation checks aligned with menu integration and fallback structure. |
| `docs/shell-hook-guarding.md` | modify (supporting docs only) | Clarify final implemented fallback/parsing behavior if Phase 2 code deviates from current wording. |
| `docs/configuration.md` | modify (conditional) | Update only if Phase 2 changes user-visible env/config semantics (currently not expected). |

## Required Context

| File | Why |
|------|-----|
| `plans/architectural-doc-code-alignment/plan.md` | Global constraints (no new deps, four-shell alignment, thin wrappers, verification bar). |
| `plans/architectural-doc-code-alignment/phases/phase-2.md` | Phase 2 scope/acceptance criteria boundary. |
| `plans/architectural-doc-code-alignment/implementation/phase-1-impl.md` | Continuity from Phase 1 adjudication and handoff assumptions. |
| `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` | Authoritative C4/C5 approved target behavior and payload contract. |
| `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` | Required future smoke scenarios to preserve while adding automated checks now. |
| `docs/shell-hook-guarding.md` | Supporting contract doc that may need narrow updates after implementation. |
| `docs/cd-extras-cli-prd.md` | Current architecture constraints (thin wrappers, split I/O, dependency-free parsing). |
| `docs/configuration.md` | Ensure no accidental config-contract drift. |
| `src/cli/menu.rs` | Runtime action emission for selected/cancelled/no-TTY flows. |
| `src/menu/action.rs` | Canonical action JSON schema used by all shell hooks. |
| `src/menu/buffer.rs` | Byte-offset semantics for `replaceStart`/`replaceEnd`. |
| `src/hooks/bash.rs` | Bash menu wrapper fallback + payload extraction behavior. |
| `src/hooks/zsh.rs` | Zsh menu widget behavior where C4 parity gap currently exists. |
| `src/hooks/fish.rs` | Fish menu wrapper fallback and regex extraction behavior. |
| `src/hooks/pwsh.rs` | PowerShell structured parsing/fallback behavior baseline. |
| `src/hooks/mod.rs` | Shell generation dispatch and supported shell surface. |
| `tests/menu_cli.rs` | Existing cross-shell menu/init contract test anchors. |
| `tests/init_cli.rs` | Existing init generation expectations per shell. |

## Implementation Steps

### Step 1: Lock C4/C5 behavior into explicit code-level targets

- **What**: Translate Phase 1 C4/C5 contract language into implementation-level acceptance notes mapped to concrete hook/menu symbols before editing code.
- **Where**: `src/hooks/{bash,zsh,fish,pwsh}.rs`, `src/cli/menu.rs`, `src/menu/action.rs`, `tests/menu_cli.rs`.
- **Why**: Prevent partial per-shell drift while preserving scope (hardening only, no unrelated UX redesign).
- **Considerations**: Preserve constraints: no new required shell dependencies, per-shell parsing may differ, wrappers remain thin, Rust selector resolution remains in `dx navigate`.

### Step 2: Implement Zsh fallback parity and keep existing shell parity paths intact

- **What**: Update `__dx_menu_widget` failure/noop branch to perform native completion-equivalent fallback instead of prompt redraw-only behavior, while preserving successful replace behavior.
- **Where**: `src/hooks/zsh.rs` menu section around `__dx_menu_widget` fallback branch.
- **Why**: C4 explicitly targets parity for noop/error fallback across shells; Zsh is the current known divergence.
- **Considerations**: Keep buffer unchanged on noop/error and avoid introducing side effects that would violate existing ZLE completion expectations.

### Step 3: Harden replace payload parsing in POSIX hooks without adding dependencies

- **What**: Replace fragile string/regex slicing paths with safer deterministic extraction/validation logic for `action`, `replaceStart`, `replaceEnd`, and `value` in Bash/Zsh/Fish wrappers (while keeping PowerShell structured JSON path as-is).
- **Where**: `src/hooks/bash.rs`, `src/hooks/zsh.rs`, `src/hooks/fish.rs`; confirm no regressions in `src/hooks/pwsh.rs`.
- **Why**: C5 requires a dependency-free but explicit and safer shell boundary for quoting/escaping-sensitive replacements.
- **Considerations**: Maintain split I/O contract (`dx menu` JSON on stdout, interactive I/O via tty/dev-tty/PSReadLine); do not add `jq`/Python helpers.

### Step 4: Keep Rust-side menu action contract stable and verify cancel/degraded semantics

- **What**: Ensure `run_menu` action outputs remain consistent for select/cancel/no-candidate/no-TTY paths and extend unit coverage only if needed to close C4/C5 gaps.
- **Where**: `src/cli/menu.rs`, `src/menu/action.rs`, `src/menu/buffer.rs`.
- **Why**: Shell hardening must not silently change the underlying Rust-owned action schema/offset semantics.
- **Considerations**: Preserve byte-offset interpretation from buffer parsing and existing cancel-with-query-change behavior.

### Step 5: Update automated tests and supporting docs in lockstep

- **What**: Add/adjust tests that assert the generated shell scripts implement the approved fallback + parsing contract; update `docs/shell-hook-guarding.md` only where final implemented behavior/limitations need clarification.
- **Where**: `tests/menu_cli.rs` (primary), `tests/init_cli.rs` (if changed expectations), optionally `docs/shell-hook-guarding.md`.
- **Why**: Phase 2 DoD requires automated coverage of changed behavior before Phase 3 manual smokes.
- **Considerations**: Do not mark shell-smoke matrix rows complete in this phase; preserve Phase 3 ownership of manual/smoke evidence.

## Testing Plan

Primary verify command (exercises changed behavior through generated hooks and menu contract tests):

```bash
cargo test menu_cli::
```

| Test Type | What to Test | Expected Outcome |
|-----------|-------------|-----------------|
| Hook generation contract tests | `dx init <shell> --menu` outputs include hardened parsing and approved noop/error fallback structure for Bash/Zsh/Fish/PowerShell. | Tests fail if any shell diverges from approved C4/C5 script contract. |
| Menu action contract tests | `dx menu` noop/replace JSON schema and cancel/degraded behavior remain stable under hardened wrappers. | Tests confirm `action` schema and fallback behavior remain machine-readable and deterministic. |
| Regression tests for non-menu paths | Non-menu init/completion behaviors remain present and unchanged by menu hardening. | Existing init/completion expectations still pass. |

### Test Integrity Constraints

- Existing tests must not be weakened, skipped, or removed to pass Phase 2; update expectations only where behavior intentionally changes to meet approved C4/C5 contract.
- Preserve the single global env lock discipline for env-mutating tests (no module-local lock reintroduction).
- Maintain coverage for all four shells in automated generator/contract tests now; Phase 3 still must execute/record Bash/Zsh/Fish/PowerShell smoke scenarios in `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md`.
- Supporting docs updates in this phase must describe implemented behavior only; no speculative claims of manual smoke completion.

## Rollback Strategy

If hardening causes regressions, revert Phase 2 deltas in this order: (1) hook parsing/fallback edits in `src/hooks/*.rs`, (2) any Rust-side menu contract edits in `src/cli/menu.rs` or `src/menu/*.rs`, (3) corresponding test expectation changes, and (4) supporting docs lines tied to reverted behavior. Re-run the verify command to confirm restored baseline before reapplying smaller shell-scoped changes.

## Open Decisions

| Decision | Options | Chosen | Rationale |
|----------|---------|--------|-----------|
| POSIX-shell parsing hardening depth for C5 | A) Keep current ad-hoc slicing/regex; B) Introduce deterministic field extraction/validation without external deps | B | Meets C5 safety goal while preserving dependency-free constraint and allowing per-shell implementation differences. |

## Reality Check

### Code Anchors Used

| File | Symbol/Area | Why it matters |
|------|-------------|----------------|
| `src/hooks/zsh.rs:307-311` | `__dx_menu_widget` noop/error branch | Confirms current Zsh divergence (reset prompt, no native completion fallback). |
| `src/hooks/bash.rs:304-330` | `__dx_try_menu` / `_dx_menu_wrapper` | Baseline fallback shape and current string-based replace extraction. |
| `src/hooks/fish.rs:238-257` | `__dx_menu_complete` replace parsing | Shows regex extraction path for `value`/offsets to harden under C5. |
| `src/hooks/pwsh.rs:336-347` | `ConvertFrom-Json` + PSReadLine replace | Structured parsing baseline already aligned with dependency constraints. |
| `src/cli/menu.rs:220-281` | `run_menu` selected/cancelled/none branches | Rust-owned action semantics for replace/noop and degraded behavior. |
| `src/menu/action.rs:5-19` | `MenuAction` enum schema | Canonical JSON action contract (`action`, `replaceStart`, `replaceEnd`, `value`). |
| `src/menu/buffer.rs:10-14,102-173` | replace byte offsets + parse logic | Grounds C5 offset-unit requirements and shell replacement semantics. |
| `tests/menu_cli.rs:305-338` | `hook_scripts_contain_fallback_on_noop` | Existing cross-shell fallback contract test anchor to update for parity expectations. |
| `tests/menu_cli.rs:415-432` | `hook_scripts_apply_replace_action_contract` | Existing replace-contract test anchor for hardened parsing expectations. |

### Mismatches / Notes

- Phase 1 docs claim Zsh should align to native-completion-equivalent noop/error fallback, but current code still uses prompt redraw without completion fallback (`src/hooks/zsh.rs:307-311`); this is the central Phase 2 implementation gap.
- POSIX hooks currently parse replace payloads with ad-hoc slicing/regex (`src/hooks/bash.rs:310-312`, `src/hooks/zsh.rs:314-319`, `src/hooks/fish.rs:249-251`), which satisfies dependency-free constraints but is brittle for escaping-sensitive payloads; Phase 2 should harden this without introducing external tools.
- `tests/menu_cli.rs` currently contains a comment asserting Zsh already falls back to expand-or-complete (`tests/menu_cli.rs:317-321`), which conflicts with generated-hook reality; test assertions must be corrected to match actual code and then tightened to enforce intended parity after implementation.
- Single-command verify uses `cargo test menu_cli::` for focused Phase 2 behavior; full-suite run and four-shell manual/smoke validation remain required by plan-level verification and are completed in/after later phase workflows.
