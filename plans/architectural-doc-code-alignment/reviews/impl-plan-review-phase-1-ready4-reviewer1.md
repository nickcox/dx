---
type: review
entity: implementation-plan-review
plan: "architectural-doc-code-alignment"
phase: 1
status: final
reviewer: "reviewer-1"
created: "2026-04-08"
---

# Implementation Plan Review: Phase 1 - Refresh Architecture Docs and Contracts

> Reviewing [Phase 1 Implementation Plan](../implementation/phase-1-impl.md)
> Against [Phase 1 Scope](../phases/phase-1.md) and [Plan](../plan.md)
> Ready-4 review: final pass focusing on (1) verify command satisfiability and gate strength, (2) C4/C5 handoff explicitness for Phase 2, and (3) execution-readiness while remaining docs/contracts-only.

## Overall Assessment

**Verdict**: Ready

This implementation plan is execution-ready. The previously reported Major finding (verify command PRD-assertions scanned both `prd` and `guard` files together, allowing the guard doc to satisfy PRD-specific checks without any PRD changes) has been resolved in this version: lines 137–142 now assert against `$prd` only, and lines 144–148 assert against `$guard` only. Step 1 makes C4/C5 handoff requirements explicit with named, labeled contract outputs and enumerated behavior branches that Phase 2 can reference without reopening scope. The verify command is not satisfiable against the current pre-implementation state (all five inventory rows are `Open`, five PRD-specific checks fail, two guard-specific checks fail, and five absence checks fail), which confirms it is a genuine acceptance gate. Three minor weaknesses remain; none block execution for a docs-only phase.

## Scope Alignment

### Findings

- **[Pass]** All four affected modules are docs/contracts artifacts: `phase-1-conflict-inventory.md`, `docs/shell-hook-guarding.md`, `docs/cd-extras-cli-prd.md`, and conditional `docs/configuration.md`. No `src/` files are in the modification list, consistent with the phase-1.md explicit exclusion of runtime changes.
- **[Pass]** C4 and C5 are correctly identified as `Code + Docs` conflicts in the inventory but limited to the contract-recording work in Phase 1. Runtime convergence (Zsh fallback alignment, parser unification decisions) is explicitly deferred to Phase 2 with rationale.
- **[Pass]** Step 4 completion checklist (four explicit done-checks) provides a clear done signal without expanding scope.
- **[Minor]** Step 3's prohibition of "verbatim obsolete command spellings anywhere in refreshed docs" is stricter than the phase-1.md acceptance criterion AC4, which permits "intentionally retained historical context in the PRD" provided it is "clearly distinguishable from current requirements." The impl plan prohibition covers `dx add`, `dx undo`, `dx redo`, `dx complete <type> <word>`, and the bare `Invoke-Expression (& dx init pwsh)` form. An implementer wanting to add a brief historical migration note would need to deviate from the impl plan text or strip content the phase explicitly permits. This is an ambiguity rather than a blocking conflict; the implementer should decide before editing begins.
- **[Note]** `plans/.../verification/shell-smoke-matrix.md` is correctly listed as read-only Required Context. Phase 1 must not mark unexecuted scenarios as passed, and the Test Integrity Constraints section states this explicitly.

## Technical Feasibility

### Findings

- **[Pass]** All code anchors cited in the Reality Check have been independently verified against the actual codebase:
  - `src/cli/mod.rs:20-54` — `Commands` enum confirmed: `Resolve`, `Init`, `Complete`, `Navigate`, `Bookmarks`, `Stack`, `Menu`. No `add`, `undo`, `redo` top-level commands. ✓
  - `src/cli/stacks.rs:12-17` — `StackCommand` is `Push`/`Undo`/`Redo` subcommands under `dx stack`. ✓
  - `src/menu/buffer.rs:10-14` — `ParsedBuffer.replace_start` / `replace_end` confirmed as byte offsets ("Byte offset where the replacement region starts"). ✓
  - `src/menu/action.rs:6-19` — `MenuAction` with `#[serde(tag = "action")]`, camelCase `replaceStart`/`replaceEnd`/`value` confirmed. ✓
  - `src/cli/menu.rs:245-281` — `MenuResult::Cancelled { changed_query: true }` → `replace` action; `changed_query: false` and `None` (no-TTY) → `noop`. ✓
  - `src/hooks/zsh.rs:307-311` — Zsh divergence confirmed: noop/error path calls `zle reset-prompt; return` without native completion fallback. ✓
  - `src/hooks/bash.rs:304-330` — Bash falls back to mode-based native completion on non-replace via `_dx_menu_wrapper`. ✓
  - `src/hooks/pwsh.rs:336-347` — PowerShell uses `ConvertFrom-Json` and `Replace` API for structured JSON parsing. ✓
  - `src/menu/tui.rs:82-116` — `CleanupGuard::drop` restores cursor/raw-mode on every exit path, including tty and stderr paths. ✓
  - `src/menu/tui.rs:139-220` — Empty candidates → `Some(Cancelled{changed_query:false})`, which becomes `noop` in menu.rs. No-TTY returns `None` (terminal::size fails). No-candidates is categorically distinct from no-TTY in action output. ✓
  - `src/hooks/mod.rs:25-37` — Canonical shell list: `bash, zsh, fish, pwsh`. ✓
- **[Pass]** The mismatch inventory in the Reality Check is accurate and complete based on direct inspection:
  - `docs/shell-hook-guarding.md` line 41 confirms: "Stack-transition wrappers ... use `dx undo`/`dx redo`" — stale vs. actual hook behavior using `dx stack undo/redo`.
  - `docs/cd-extras-cli-prd.md` lines 23, 64–65, 69, 75 confirm: `dx add`, generic `undo`/`redo` as top-level commands, `dx complete <type> <word>`, `Invoke-Expression (& dx init pwsh)` all present and obsolete.
  - `docs/shell-hook-guarding.md` line 73 already has the correct `Out-String` form for `--menu` init; only the PRD and the stale line 41 require fixing.
- **[Note]** Single-candidate auto-select behavior (tui.rs:154–160, `len == 1 && !has_more → Selected`) is distinct from the no-candidates scenario. The plan's request to document no-candidates "when distinct" is correct: no-candidates goes to `Cancelled{changed_query:false}` → `noop`, while single-candidate goes to `Selected` → `replace`. These are distinct paths that should be in the C4 contract.
- **[Note]** The parsing asymmetry (string/regex in Bash/Zsh/Fish vs. `ConvertFrom-Json` in PowerShell) is confirmed by inspection and the plan correctly records it as acceptable-but-explicit rather than requiring unification in Phase 1.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries with code-grounded decisions | Yes | Yes | Explicit per-shell labels, all C4 branch scenarios, all C5 payload/offset/escaping/split-I/O items are enumerated. Named outputs (`Approved C4 Target Behavior`, `Approved C5 Payload/Escaping Contract`) make Phase 2 handoff requirements unambiguous. |
| 2 | Refresh shell-hook contract docs to current semantics | Yes | Yes | Well-grounded in source files. Conditional `docs/configuration.md` trigger is implicit ("only if contradiction found") but acceptable for a docs-only phase. |
| 3 | Rewrite PRD to current command surface and architecture baseline | Yes | Yes | Clear prohibition list is actionable. Minor ambiguity with historical-context allowance in phase-1.md AC4 (see Scope Alignment). |
| 4 | Cross-check docs against source and phase scope boundaries | Yes | Yes | Four-item completion checklist provides a concrete done signal. Confirms that any remaining runtime divergence must be logged as Phase 2 work, not marked implemented. |

## Required Context Assessment

### Missing Context

- **[Note]** `AGENTS.md` is listed in Required Context (line 38) and is correct. It directly informs the C2/C3 decisions and the PowerShell init gotcha. No gap.

### Unnecessary Context

- **[Note]** `plans/.../verification/shell-smoke-matrix.md` is listed as read-only reference. It is not substantively used in any Phase 1 step but serves as an integrity constraint reminder. The listing is conservative and acceptable.

## Testing Plan Assessment

### Test Integrity Check

- **[Pass]** The Test Integrity Constraints section explicitly states that no existing Rust/unit/integration tests are modified, removed, skipped, or weakened during Phase 1. This satisfies the guard for a docs/contracts-only phase.
- **[Pass]** Phase 2–3 automated test and four-shell smoke bar is preserved. Phase 1 defers all runtime verification.
- **[Pass]** Shell-smoke-matrix constraint is correctly stated: Phase 1 must not mark unexecuted scenarios as passed.

### Test Gaps

- **[Minor]** Three of five guard-doc positive assertions (lines 146–148: `dx navigate`, `dx menu`, `Out-String`) already pass against the current pre-implementation `docs/shell-hook-guarding.md`. This means those three checks do not enforce that the guard doc was actually improved; they only confirm the guard doc still references these terms. The actual guard-required changes (removing stale `dx undo`/`dx redo` and adding `dx stack undo`/`dx stack redo`) are enforced by the absence checks (lines 151–152) and the `dx stack undo`/`dx stack redo` presence checks (lines 144–145). The net effect is acceptable: the verify command cannot be gamed past these two presence checks and two absence checks, but the three pre-passing guard checks add no incremental enforcement. This is a known, bounded weakness.

- **[Minor]** The `dx resolve` check at line 137 (`rg -n -F "dx resolve" "$prd"`) passes against the current pre-implementation PRD because `docs/cd-extras-cli-prd.md` already mentions `dx resolve` at line 20 ("calls: `dx resolve "pr/cd"`"). This single check yields a false positive pre-implementation. However, the other five PRD checks (lines 138–142: `dx stack`, `dx navigate`, `dx complete paths`, `dx menu`, `Out-String`) all fail pre-implementation, so the PRD section of the verify command is still not collectively satisfiable without a genuine PRD rewrite. The false positive on `dx resolve` is a minor cosmetic weakness, not a gate gap.

- **[Pass]** The verify command correctly prevents the inventory-based checks from being satisfied by the current state (all five C1–C5 rows are `Open`, all required C4/C5 marker strings are absent from the inventory). This is the primary acceptance gate and it is sound.

### Real-World Testing

Not applicable for Phase 1 (docs/contracts-only scope). The plan explicitly notes that runtime verification (cargo test, four-shell smokes) is deferred to Phases 2 and 3, and the Test Integrity Constraints section preserves those later requirements. No waiver is needed for Phase 1.

## Reference Consistency

### Findings

- **[Pass]** All `src/` file paths and symbol references in the Required Context and Reality Check sections have been verified against the actual codebase. All paths exist. All symbol references are accurate.
- **[Pass]** `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` exists and matches the referenced table structure.
- **[Pass]** `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` exists and is correctly referenced as read-only.
- **[Pass]** Cross-references to `plan.md` and `phase-1.md` are correct in the frontmatter and section headers.
- **[Minor]** `src/hooks/mod.rs:25-37` is cited as confirming the `supported_list` and `generate` entrypoint. Confirmed: `supported_list()` is at lines 25–27 and `generate` at lines 30–37. The anchor is slightly misattributed in the Reality Check row description ("supported_list / generate") but the line range is accurate. ✓

## Reality Check Validation

### Findings

- **[Pass]** The Reality Check section identifies the two current doc mismatches accurately: `docs/shell-hook-guarding.md` line 41 (stale `dx undo`/`dx redo` wording) and `docs/cd-extras-cli-prd.md` (proposal-era command surface and obsolete PowerShell init). Both were verified by direct inspection.
- **[Pass]** The Zsh divergence mismatch (noop/error behavior differs from Bash/Fish/PowerShell) is correctly noted and correctly scoped as a Phase 2 implementation item.
- **[Pass]** The parsing asymmetry (Bash/Zsh/Fish string/regex vs. PowerShell `ConvertFrom-Json`) is correctly noted as intentional and within-scope for the C5 contract codification.
- **[Pass]** The note that "conflict inventory may intentionally retain legacy command strings as historical conflict evidence" is accurate and correctly scopes the stale-string removal checks to refreshed docs only, not the inventory table itself.
- **[Note]** The Reality Check does not explicitly note that `docs/shell-hook-guarding.md` line 73 already has the correct `((& dx init pwsh --menu | Out-String))` form, which means C2 for the guard doc is already partially aligned. The C2 fix primarily targets `docs/cd-extras-cli-prd.md` line 75 (bare `Invoke-Expression (& dx init pwsh)`). This is a documentation omission in the mismatch list, not a gap in the plan's steps.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Minor | Scope Alignment | Step 3 prohibition of all legacy command spellings is stricter than phase-1.md AC4, which permits clearly-labeled historical context. An implementer wanting a migration note would need to deviate. | Implementer should decide before editing whether to retain any historical context section; if retained, clearly frame it as background to satisfy both constraints. |
| 2 | Minor | Testing Plan | Three of five guard-doc positive assertions (`dx navigate`, `dx menu`, `Out-String`) already pass pre-implementation and add no incremental enforcement. Gate strength relies on the two presence checks (`dx stack undo/redo`) and two absence checks (`dx undo/redo`). | No action required. The gate is still sound overall; this is a note on redundant checks, not a gap. |
| 3 | Minor | Testing Plan | `dx resolve` PRD check (line 137) passes pre-implementation because `docs/cd-extras-cli-prd.md` already contains `dx resolve "pr/cd"` at line 20. This single check produces a false positive pre-implementation. | No action required. Five other PRD checks fail pre-implementation, so the PRD section as a whole correctly requires genuine implementation. |
| 4 | Note | Reality Check | The Reality Check does not note that `docs/shell-hook-guarding.md` line 73 already has the correct `Out-String` form (for `--menu` init), meaning C2 for the guard doc is partially resolved. | Informational only. The impl plan steps still cover the C2 guard-doc fix (removing stale line 41 language); no action required. |
| 5 | Note | Scope Alignment | The trigger for updating `docs/configuration.md` is implicitly scoped to "if contradiction found." | Acceptable ambiguity for a docs-only phase. Implementer should document any contradiction decision in Step 4 if triggered. |

## Recommendations

1. **Before starting Step 3**: Decide whether any historical/migration note is retained in the refreshed PRD (phase-1.md AC4 permits it if clearly labeled; impl plan Step 3 prohibits verbatim obsolete spellings). Align on this before editing to avoid ambiguity.
2. **Step 1 — no action needed**: C4/C5 named contract outputs and per-shell/labeled requirements are explicit enough for Phase 2 inheritance. The verify command will properly gate on these markers being present.
3. **After execution**: Confirm that the refreshed `docs/shell-hook-guarding.md` no longer contains the standalone `dx undo`/`dx redo` references (line 41) and that the refreshed `docs/cd-extras-cli-prd.md` omits `dx add`, bare `Invoke-Expression (& dx init pwsh)`, and `complete <type> <word>`. The absence checks in the verify command gate these directly.
