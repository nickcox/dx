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

## Overall Assessment

**Verdict**: Needs Revision

The implementation plan is well-structured, grounded in actual codebase anchors, and correctly defers runtime behavior changes to Phase 2. However, it has three meaningful gaps: (1) the verify command is too narrow to confirm all deliverables and contains a regex that may produce false-passing results; (2) the plan is missing a required context file (`src/hooks/mod.rs`) that is referenced in the phase doc notes; and (3) it does not articulate a concrete pass/fail test for whether the conflict inventory itself has been fully resolved — the only automated check targets obsolete string patterns in docs, not the inventory artifact. These gaps do not block execution but weaken the quality gate before Phase 2 hand-off.

## Scope Alignment

### Findings

- **[Pass]** The plan stays strictly within docs/contracts scope. No runtime or code changes are proposed in any step. The "Approach" section explicitly defers all hook/menu behavior changes to Phase 2. This is correctly aligned with phase-1.md's "Excludes" section.
- **[Pass]** All four required deliverables from phase-1.md are addressed: conflict inventory (Step 1), `docs/shell-hook-guarding.md` (Step 2), `docs/cd-extras-cli-prd.md` (Step 3), and the conditional `docs/configuration.md` update (Step 2 considerations).
- **[Pass]** The Affected Modules table correctly lists `docs/configuration.md` as conditional ("only if needed"), matching the phase-1.md scope qualifier.
- **[Minor]** The Affected Modules table marks the plan artifact path as `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md`. This file already exists and the plan correctly treats it as a "modify" target — consistent with codebase reality.
- **[Minor]** Step 4 ("cross-check docs against source") is somewhat abstract: it describes validation but does not reference a concrete checklist or enumerate the specific cross-checks to perform. An implementer may interpret it variably.

## Technical Feasibility

### Findings

- **[Pass]** The docs-only approach is technically appropriate. Since no code files are modified, there is no risk of breaking behavior or failing tests.
- **[Pass]** The Open Decisions table correctly resolves the target-vs-current documentation strategy (Option B: document intended aligned target and note current mismatch). This is consistent with the plan's requirement to record decisions for Phase 2.
- **[Note]** The plan implicitly relies on the implementer's judgment to decide whether `docs/configuration.md` has a "real contradiction." No guidance is given on what threshold qualifies, which could lead to under- or over-updating. A note on what to look for (e.g., env var names, menu flags, session variable names) would sharpen this.
- **[Minor]** The Reality Check notes that "parsing strategies are intentionally asymmetric today." This is accurate (verified in source: Bash/Zsh/Fish use string slicing on the JSON, PowerShell uses `ConvertFrom-Json`). However, the plan does not address whether this asymmetry should be *described* in `docs/shell-hook-guarding.md` or merely noted in the conflict inventory. This is an editorial gap that could surface during implementation.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries | Yes | Yes | None — task is clear: update each C1–C5 row to Resolved/Deferred with evidence. |
| 2 | Refresh shell-hook contract docs | Mostly | Yes | The conditional `docs/configuration.md` update lacks a concrete trigger criterion. |
| 3 | Rewrite PRD to current command surface | Yes | Yes | "Preserve useful historical context" is subjective but acceptable given the example guidance provided. |
| 4 | Cross-check docs against source | No | Partial | Step is stated as a "final pass" but does not enumerate specific checks. No checklist, no acceptance signal. |

- **[Minor]** Step 4 lacks a clear "done" signal for the implementer. Without enumerated checks, an implementer cannot confirm this step is fully complete. A short list of assertions (e.g., "confirm no `dx undo`/`dx redo` patterns remain in docs", "confirm no `dx complete <type> <word>` patterns remain") would transform this from advisory to verifiable.
- **[Pass]** Steps 1–3 each have clear What/Where/Why and actionable considerations, well-grounded in real file paths.

## Required Context Assessment

### Missing Context

- `src/hooks/mod.rs` — The phase-1.md notes section explicitly lists this as a reference: "Primary references for this phase are expected to include…`src/hooks/mod.rs`." The implementation plan's Required Context table lists individual shell hook files but omits `src/hooks/mod.rs`. While this file may only hold re-exports, its omission is inconsistent with the phase guidance and could cause an implementer to miss hook-level wiring.
- `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` is listed in Required Context but the plan body never references it substantively. The Testing Plan section cites it as a constraint ("must not mark unexecuted scenarios as passed") but does not explain how Phase 1 interactions with it. Listing it without actionable reference is borderline over-specification.

### Unnecessary Context

- None identified. All listed context files have a clear rationale and were verified to exist in the repo.

## Testing Plan Assessment

### Test Integrity Check

- **[Pass]** The Test Integrity Constraints section explicitly states: "No existing Rust/unit/integration tests are modified, removed, skipped, or weakened during Phase 1 (docs/contracts-only scope)." This satisfies the integrity requirement.
- **[Pass]** The plan correctly notes that Phase 2 and Phase 3 must still satisfy the plan-level verification bar, and defers automated tests to those phases.
- **[Pass]** The shell-smoke-matrix.md is referenced as a constraint: Phase 1 must not mark unexecuted scenarios as passed.

### Test Gaps

- **[Major]** The verify command only checks for obsolete string patterns in the three named files. It does not verify that `contracts/phase-1-conflict-inventory.md` has all rows in `Resolved` or `Deferred` status — the primary Phase 1 acceptance criterion. A passing verify command is entirely consistent with the conflict inventory being left in `Open` state. Supplementary check needed (e.g., `rg -n "| Open |" plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md`).
- **[Minor]** The verify regex pattern `Invoke-Expression \(& dx init pwsh\)` uses a literal `\(` that may not work as intended with `rg` without flag adjustments. In `rg`, parentheses are not metacharacters by default, so `\(` will actually match a literal backslash + `(`, not just `(`. The correct pattern should be either `Invoke-Expression \(& dx init pwsh\)` without the backslashes (since `rg` default mode treats parens as literals) or use `--fixed-strings` for string matching. This risk of a false-passing result is a notable quality concern.
- **[Minor]** The verify command checks `docs/cd-extras-cli-prd.md` but does not verify positive assertions — e.g., that the PRD now *contains* references to `dx stack`, `dx navigate`, or `dx complete paths`. Absence of stale strings is a weak confirmation. A positive check (e.g., `rg -n "dx stack" docs/cd-extras-cli-prd.md`) would increase confidence.
- **[Note]** There is no automated check for `docs/shell-hook-guarding.md` to verify the `dx undo`/`dx redo` stale wording has been removed. The pattern `dx undo|dx redo` could be added to the verify command.

### Real-World Testing

**Not planned for Phase 1 — and appropriately so.** Phase 1 is docs/contracts-only. No code changes are made, so real-world shell testing is explicitly out of scope (per phase-1.md Excludes). The shell smoke matrix is the Phase 3 responsibility, not Phase 1. This is a correct and justified deferral — not a limitation to flag.

## Reference Consistency

### Findings

- **[Pass]** `src/cli/mod.rs:20-54` — Verified. The `Commands` enum at lines 19–54 includes `Resolve`, `Init`, `Complete`, `Navigate`, `Bookmarks`, `Stack`, `Menu`. The implementation plan's claimed line range is accurate.
- **[Pass]** `src/cli/complete.rs:13-55` — Verified. `CompleteCommand` enum with modes `Paths`, `Ancestors`, `Frecents`, `Recents`, `Stack` spans approximately lines 12–55. Accurate.
- **[Pass]** `src/cli/complete.rs:146-166` — Verified. `run_navigate` is at lines 146–166. Accurate.
- **[Pass]** `src/cli/stacks.rs:12-17` — Verified. `StackCommand` enum with `Push`, `Undo`, `Redo` is at lines 12–17. Accurate.
- **[Pass]** `src/menu/action.rs:6-19` — Verified. `MenuAction` enum with `Replace` (fields `replace_start`, `replace_end`, `value`) and `Noop` is at lines 6–19. The JSON keys match: `replaceStart`, `replaceEnd`, `value`. Accurate.
- **[Pass]** `src/cli/menu.rs:245-281` — Verified. `MenuResult::Cancelled` with `changed_query` branch is at lines 245–281. Noop for unchanged cancel is confirmed at lines 266–271. Accurate.
- **[Pass]** `src/hooks/bash.rs:304-330` — Verified. `__dx_try_menu` at lines 304–315 and `_dx_menu_wrapper` at lines 317–330. Bash falls back to native completion on non-replace/error. Accurate.
- **[Pass]** `src/hooks/zsh.rs:307-311` — Verified. Lines 307–312 confirm Zsh noop/error path resets prompt and returns without native completion fallback. The plan's claimed divergence is accurate.
- **[Pass]** `src/hooks/fish.rs:238-247` — Verified. Lines 238–247 confirm Fish falls back to `commandline -f complete` on error or non-replace. Accurate.
- **[Pass]** `src/hooks/pwsh.rs:336-347` — Verified. Lines 336–347 confirm PowerShell uses `ConvertFrom-Json` and PSReadLine Replace API. Accurate.
- **[Pass]** The mismatch noted in the Reality Check — "`docs/shell-hook-guarding.md` says `dx undo`/`dx redo`" — is verified: line 41 of `docs/shell-hook-guarding.md` reads "Stack-transition wrappers (`back`/`forward`/`cd-`/`cd+`) use `dx undo`/`dx redo` (and `--target` for selector-based jumps)". This is confirmed stale; actual hooks use `dx stack undo`/`dx stack redo` (verified: `src/hooks/bash.rs:73,75`, `src/hooks/zsh.rs:73,75`, `src/hooks/pwsh.rs:112,114`).
- **[Pass]** The `docs/cd-extras-cli-prd.md` obsolete commands — `dx add`, `dx undo`, `dx redo`, `dx complete <type> <word>` — are verified present in the current file (lines 23, 64–65, 69, 75). The plan's characterization is accurate.

## Reality Check Validation

### Findings

- **[Pass]** The Reality Check section examined 10 code anchors across 6 source files, covering all major conflict areas (C1–C5). This is sufficient breadth for a docs-only phase.
- **[Pass]** All noted mismatches are genuine. Each was independently verified during this review and confirmed accurate.
- **[Pass]** The plan correctly flags that "parsing strategies are intentionally asymmetric today" as a boundary condition Phase 1 must codify without forcing unification.
- **[Pass]** The plan correctly identifies that the Zsh divergence is a runtime issue deferred to Phase 2 and does not try to paper over it in Phase 1 docs.
- **[Minor]** The Reality Check does not examine `src/hooks/fish.rs` stack wrapper behavior to confirm it also uses `dx stack undo`/`dx stack redo`. While the bash and zsh anchor coverage is strong, fish is referenced in C3 ("stack-wrapper command wording") and its exclusion from the reality check is a small gap. (Actual fish code at lines ~56–79 of `src/hooks/fish.rs` confirms `dx stack undo`/`dx stack redo` usage, consistent with plan's claims, but the plan did not explicitly check it.)
- **[Note]** The Reality Check does not mention `src/hooks/mod.rs`. This is low-risk since the file is primarily a module re-export file, but it would have been tidy to check given the phase-1.md reference.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Major | Testing Plan | Verify command does not check whether `phase-1-conflict-inventory.md` rows are resolved. A passing verify is entirely compatible with all rows staying `Open`. | Add `rg -n "\| Open \|" plans/.../contracts/phase-1-conflict-inventory.md` as a second verify check with expected outcome: no matches. |
| 2 | Minor | Testing Plan | Verify regex `Invoke-Expression \(& dx init pwsh\)` uses backslash-escaped parens that will match literal `\(` in `rg` default mode, not `(`. This could produce false-passing results. | Remove the backslashes or use `--fixed-strings` flag for that pattern. |
| 3 | Minor | Required Context | `src/hooks/mod.rs` is referenced in phase-1.md notes as an expected context file but is absent from the Required Context table. | Add `src/hooks/mod.rs` to the Required Context table. |
| 4 | Minor | Step Quality | Step 4 lacks enumerated checks and has no clear done-signal for the implementer. | Add a short checklist of assertions (e.g., no `dx undo`/`dx redo` in docs, no `dx complete <type> <word>`, PRD references current commands). |
| 5 | Minor | Testing Plan | Verify command does not check for `docs/shell-hook-guarding.md` stale `dx undo`/`dx redo` removal, nor for positive presence of current command patterns in the PRD. | Add `dx undo|dx redo` to the verify pattern list, and consider a positive-assertion check for `dx stack` in the refreshed PRD. |
| 6 | Note | Required Context | `shell-smoke-matrix.md` is listed in Required Context but the plan body has only a constraint reference (don't mark scenarios passed). Its relevance to Phase 1 implementation is marginal. | Keep the reference but add a brief note that it is read-only context for Phase 1 to avoid confusion. |
| 7 | Note | Reality Check | `src/hooks/fish.rs` stack wrapper behavior (C3 evidence) was not checked. The anchor is missing from the Reality Check table. | Add a fish.rs anchor to the Reality Check for completeness. |

## Recommendations

1. **[Major — before execution]** Add a supplementary verify step to check that all conflict inventory rows are no longer `Open`. Example: `rg -n "\| Open \|" plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` — expected outcome: no matches. Without this, the primary success criterion for Phase 1 is untestable via the stated verify command.

2. **[Minor — fix before execution]** Correct the `rg` verify pattern: remove backslash escapes around parentheses in the PowerShell init pattern, or switch to `--fixed-strings`. The current pattern `Invoke-Expression \(& dx init pwsh\)` will match `\(` literally in `rg` default mode and is likely to produce a false-passing result if the text uses bare `(`.

3. **[Minor — can be added during execution]** Add `src/hooks/mod.rs` to the Required Context table and enumerate Step 4's cross-checks explicitly. These are minor documentation quality improvements that can be done as the implementer starts work; they do not block execution.
