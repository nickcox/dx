---
type: review
entity: implementation-plan-review
plan: "architectural-doc-code-alignment"
phase: 1
status: final
reviewer: "reviewer-1"
created: "2026-04-08"
---

# Implementation Plan Review: Phase 1 - Refresh Architecture Docs and Contracts (Final)

> Reviewing [Phase 1 Implementation Plan](../implementation/phase-1-impl.md)
> Against [Phase 1 Scope](../phases/phase-1.md) and [Plan](../plan.md)
> Final readiness review: focusing on whether the verify command is now satisfiable and strong enough for Phase 1 acceptance, whether Step 1 makes C4/C5 contract outputs explicit enough, and whether the plan is execution-ready while remaining docs/contracts-only.

## Overall Assessment

**Verdict**: Needs Revision

The plan is well-grounded in reality, correctly scoped to docs/contracts only, and the prior Critical finding (conflict inventory included in absence-check file lists) has been successfully resolved in this version. However, one **Major** structural weakness remains in the verify command: positive assertions for `dx navigate`, `dx complete paths`, `dx menu`, and `Out-String` scan both `docs/cd-extras-cli-prd.md` and `docs/shell-hook-guarding.md` together, and since `shell-hook-guarding.md` already contains all four of these terms, the verify command passes these checks today without any changes to the PRD — meaning the verify command cannot catch a missed or incomplete PRD rewrite (the largest deliverable in Phase 1). This is a structural gap that should be fixed before execution. With that fix, the plan is execution-ready.

## Scope Alignment

### Findings

- **[Pass]** All affected files are docs/plan artifacts: `phase-1-conflict-inventory.md`, `docs/shell-hook-guarding.md`, `docs/cd-extras-cli-prd.md`, and conditional `docs/configuration.md`. No runtime code changes are proposed or implied.
- **[Pass]** All four Phase 1 deliverables are covered: conflict inventory (Step 1), `docs/shell-hook-guarding.md` (Step 2), `docs/cd-extras-cli-prd.md` (Step 3), and conditional `docs/configuration.md` (Step 2 considerations).
- **[Pass]** Step 1 explicitly names the two required contract outputs — `Approved C4 Target Behavior` (cross-shell noop/error/replace matrix) and `Approved C5 Payload/Escaping Contract` — which was a gap in prior versions.
- **[Pass]** Step 4 completion signals are concrete: four explicit done-checks are listed, and the Phase 2 handoff boundary is unambiguously defined.
- **[Pass]** Runtime/code changes (Zsh fallback convergence, parser unification) are explicitly deferred to Phase 2 with clear rationale.
- **[Minor]** The trigger criterion for updating `docs/configuration.md` remains undefined ("only if contradiction is found"). This is an acceptable ambiguity for a docs-only phase but could be tightened for a cleaner acceptance signal.

## Technical Feasibility

### Findings

- **[Pass]** The docs-only approach is technically appropriate. No runtime behavior changes are proposed; the entire scope is documentation and conflict-inventory authoring grounded in source anchors.
- **[Pass]** The plan correctly treats current Zsh divergence (noop/error resets prompt without native completion fallback, confirmed in `src/hooks/zsh.rs:307-312`) as Phase 2 implementation work, not a Phase 1 documentation normalization error.
- **[Pass]** The plan correctly identifies the parsing asymmetry (string/regex slicing in Bash/Zsh/Fish vs. `ConvertFrom-Json` in PowerShell) as requiring codified boundary contract documentation, not forced unification in Phase 1.
- **[Note]** The plan's approach to C5 (allow per-shell parsing to differ if the contract is explicit and safe) is well-reasoned and consistent with the plan-level non-functional requirement to introduce no new required dependencies.
- **[Minor]** C4 still specifies the required output as a "matrix for Bash/Zsh/Fish/PowerShell" without enumerating the behavior branches that each cell must cover. The smoke matrix (in `shell-smoke-matrix.md`) implies six scenarios per shell (init usage, menu disabled, successful replace, cancel-with-typed-query, noop/error fallback, no-TTY/degraded). The Phase 1 C4 contract output should cover at least the noop/error/replace/no-TTY branches explicitly to serve as a genuine Phase 2 handoff specification.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries with code-grounded decisions | Yes | Yes | C4/C5 named outputs now explicit. C4 behavior branches should be enumerated more fully (minor). |
| 2 | Refresh shell-hook contract docs to current semantics | Yes | Yes | Well-grounded in source; conditional `docs/configuration.md` trigger is implicit but acceptable. |
| 3 | Rewrite PRD to current command surface and architecture baseline | Yes | Yes | Clear, actionable, grounded in `src/cli/` files. Historical context handling is correctly articulated. |
| 4 | Cross-check docs against source and phase scope boundaries | Yes | Yes | Four explicit done-checks make this verifiable. But the primary verify command does not mechanically enforce the PRD-specific checks. |

## Required Context Assessment

### Missing Context

- **[Minor]** `AGENTS.md` is not listed as required context, despite the conflict inventory (C2 row) citing `AGENTS.md` gotcha as key evidence for the PowerShell init guidance correction. An implementer executing Step 2 or Step 3 should consult the `Out-String` gotcha directly to understand the single-script-block constraint. Adding it as a read-only reference would close the loop.

### Unnecessary Context

- `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` is listed as required context with a test-integrity constraint note. It is correctly read-only for Phase 1 and the constraint is valid; however, no Phase 1 step interacts with it substantively. Listing it is conservative but acceptable.

## Testing Plan Assessment

### Test Integrity Check

- **[Pass]** The Test Integrity Constraints section explicitly states that no existing Rust/unit/integration tests are modified, removed, skipped, or weakened during Phase 1. This satisfies the integrity guard for a docs/contracts-only phase.
- **[Pass]** The plan preserves the Phase 2–3 automated test and four-shell smoke bar. Phase 1 intentionally defers runtime verification.
- **[Pass]** The shell-smoke-matrix constraint is stated correctly: Phase 1 must not mark unexecuted scenarios as passed.

### Test Gaps

- **[Major]** The positive assertions in the verify command (`rg -n -F "dx navigate" docs/cd-extras-cli-prd.md docs/shell-hook-guarding.md`, and analogously for `dx complete paths`, `dx menu`, and `Out-String`) scan both target docs together. Since `docs/shell-hook-guarding.md` already contains all four of these exact strings (verified: `dx navigate` at line 39, `dx menu` at line 56, `dx complete paths` at line 110, `Out-String` at line 73), these positive assertions will pass today without any changes to `docs/cd-extras-cli-prd.md`. An implementer who skips the PRD rewrite entirely would still have these four checks pass. This makes the verify command unable to detect an incomplete PRD update — the single largest Step 3 deliverable.

  **Fix**: Replace the two-file `rg` scan with PRD-specific assertions. For example:
  ```bash
  rg -n -F "dx stack" docs/cd-extras-cli-prd.md >/dev/null
  rg -n -F "dx navigate" docs/cd-extras-cli-prd.md >/dev/null
  rg -n -F "dx complete paths" docs/cd-extras-cli-prd.md >/dev/null
  rg -n -F "dx menu" docs/cd-extras-cli-prd.md >/dev/null
  rg -n -F "Out-String" docs/cd-extras-cli-prd.md >/dev/null
  ```
  These five checks will fail until the PRD is actually updated and will pass once it is. The `shell-hook-guarding.md` assertions can be narrowed or kept separately.

- **[Minor]** The primary verify command does not include `dx resolve` in the positive checks despite it being the primary path-resolution command in the current CLI surface (`src/cli/mod.rs:21`) and a prominent anchor for the PRD refresh. Step 3 instructs the implementer to prioritize current-contract commands; `dx resolve` being absent from the gate reduces coverage of whether it was correctly documented.

- **[Minor]** The C4/C5 contract output checks verify only header/label presence (the exact strings `"Approved C4 Target Behavior"` and `"Approved C5 Payload/Escaping Contract"`) rather than any substantive content. An inventory entry with only the header label and no decision text would satisfy the verify command. The Step 4 done-checks say "C4/C5 contract outputs are present and unambiguous" — this is not mechanically enforced by the verify gate.

- **[Minor]** The `! rg -n -F "dx undo" docs/cd-extras-cli-prd.md docs/shell-hook-guarding.md` absence check is correct: after Phase 1 replaces line 41 of `shell-hook-guarding.md` with `dx stack undo`/`dx stack redo`, the literal string `"dx undo"` will no longer appear. The `-F` fixed-string mode correctly distinguishes `"dx undo"` from `"dx stack undo"`. No false-positive risk.

- **[Note]** Step 3's Historical Background guidance (`"preserve useful historical context only when clearly labeled as historical and separated from normative sections"`) could create a minor conflict with the absence-check assertions. The verify command bans `"dx add"`, `"dx undo"`, and `"dx redo"` from the doc targets — but if a clearly-labeled Historical Background section uses these command names to describe legacy behavior, the absence check would incorrectly flag them. This is a theoretical issue: the plan says to "summarize without restating obsolete command spellings verbatim in active contract sections," implying verbatim spellings in historical sections remain allowed. The verify command does not reflect that nuance. This should either be enforced explicitly in the step text or accepted as a known limitation.

### Real-World Testing

No real-world shell testing is planned for Phase 1. This is explicitly correct per `phase-1.md` Excludes section ("End-to-end shell smoke verification beyond quick sanity checks needed while editing docs"). Shell smoke verification is the Phase 3 responsibility. The deferral is intentional and correct — it is not a limitation.

## Reference Consistency

### Findings

- **[Pass]** `src/cli/mod.rs:20-54` — Verified. `Commands` enum at lines 19–54: `Resolve`, `Init`, `Complete`, `Navigate`, `Bookmarks`, `Stack`, `Menu`. Accurate.
- **[Pass]** `src/cli/complete.rs:13-55` — Verified. `CompleteCommand` enum with `Paths`, `Ancestors`, `Frecents`, `Recents`, `Stack` modes at lines 12–55. Accurate.
- **[Pass]** `src/cli/complete.rs:146-166` — Verified. `run_navigate` handles `NavigateMode::Up/Back/Forward` selector resolution in Rust. Accurate.
- **[Pass]** `src/cli/stacks.rs:12-17` — Verified. `StackCommand` enum with `Push`, `Undo`, `Redo` at lines 12–17. Confirms current stack command surface. Accurate.
- **[Pass]** `src/menu/action.rs:6-19` — Verified. `MenuAction` with `Replace { replace_start, replace_end, value }` and `Noop`, serialized with camelCase JSON keys (`replaceStart`, `replaceEnd`). Accurate.
- **[Pass]** `src/cli/menu.rs:245-281` — Verified. `MenuResult::Cancelled` with `changed_query` at lines 245–281: cancel-with-changed-query emits `replace`; cancel-without-change emits `noop`; TUI-unavailable emits `noop`. Accurate.
- **[Pass]** `src/menu/tui.rs:82-116` — Verified. `CleanupGuard::drop` at lines 88–116 restores cursor visibility, clears menu area rows, disables raw mode on all exit paths including `/dev/tty` backend. Accurate.
- **[Pass]** `src/menu/tui.rs:139-220` — Verified. `select()` function shows no-candidate handling (returns `Cancelled` with `changed_query: false`), TTY-unavailable via `terminal::size().ok()?` returning `None`. Accurate.
- **[Pass]** `src/hooks/mod.rs:25-37` — Verified. `supported_list()` returns `"bash, zsh, fish, pwsh"` and `generate()` dispatches by shell. Accurate.
- **[Pass]** `src/hooks/bash.rs:304-330` — Verified. `__dx_try_menu` at 304–315 and `_dx_menu_wrapper` at 317–330. Bash falls back to native completion handlers on non-replace/error. Accurate.
- **[Pass]** `src/hooks/zsh.rs:307-311` — Verified. Lines 307–312 confirm Zsh noop/error path resets prompt (`zle reset-prompt`) and returns without native completion fallback. Divergence from Bash/Fish/PowerShell is genuine. Accurate.
- **[Pass]** `src/hooks/fish.rs:238-247` — Verified. Lines 239–247 confirm Fish falls back to `commandline -f complete` on both error (`test $status -ne 0`) and non-replace. Accurate.
- **[Pass]** `src/hooks/pwsh.rs:336-347` — Verified. Lines 336–347 confirm PowerShell uses `ConvertFrom-Json` and PSReadLine `Replace` API on successful replace action. Accurate.
- **[Pass]** The Reality Check mismatch claims are all genuine: `docs/shell-hook-guarding.md` line 41 uses `dx undo`/`dx redo` (confirmed stale — actual hooks use `dx stack undo|redo`), and `docs/cd-extras-cli-prd.md` line 23/64/65/69/75 confirms all four obsolete command/init references.

## Reality Check Validation

### Findings

- **[Pass]** The Reality Check covers 14 code anchors across 8 source files. All anchors were independently verified against the current codebase and are accurate.
- **[Pass]** Noted mismatches are genuine and completely grounded in current code. No false claims found.
- **[Pass]** The plan honestly distinguishes current-divergence-that-needs-Phase-2-code-work from documentation-only corrections that Phase 1 can address. This boundary is critical for maintaining scope discipline.
- **[Pass]** The inventory-as-evidence note at the end of the Reality Check section is correct: the conflict inventory may legitimately retain legacy command strings as conflict evidence, so stale-string removal checks are only applied to the refreshed docs targets.
- **[Minor]** The Reality Check does not explicitly call out that `MenuAction`'s `replaceStart`/`replaceEnd` are **byte offsets** (not character or grapheme offsets). The C5 payload contract must make this interpretation explicit so Phase 2 shell hook implementations produce correct replacements for non-ASCII paths. The current code uses `usize` integer fields without documented unit. This is a content gap for the C5 contract output, not a reference error.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Major | Testing Plan | The positive assertions (`dx navigate`, `dx complete paths`, `dx menu`, `Out-String`) scan both docs together. Since `docs/shell-hook-guarding.md` already contains all four terms today, these checks pass without any PRD update — making the verify command blind to an incomplete PRD rewrite. | Scope PRD-specific positive assertions to `docs/cd-extras-cli-prd.md` alone (add `rg -n -F "dx stack" docs/cd-extras-cli-prd.md`, `rg -n -F "dx navigate" docs/cd-extras-cli-prd.md`, etc.). |
| 2 | Minor | Testing Plan | C4/C5 verify checks confirm only label/header presence, not that substantive contract content exists. An empty or placeholder entry would satisfy the gate. | Add content-probe assertions (e.g., `rg -n -F "Bash" plans/.../phase-1-conflict-inventory.md` and `rg -n -F "Zsh" ...`) as a minimal content check, or rely on Step 4's done-checklist as the human gate for content adequacy. |
| 3 | Minor | Testing Plan | The PRD Historical Background allowance (Step 3) permits retaining verbatim legacy command spellings in a clearly labeled section, but the absence-check assertions would flag those occurrences as failures. | Either add a note that historical sections must not use verbatim legacy command spellings, or accept this as a known limitation and note it in the verify command description. |
| 4 | Minor | Step Quality / Reality Check | C4 behavioral contract does not enumerate the required subcases (menu disabled, select/replace, cancel-with-query-change, noop/error-fallback, no-TTY/degraded) that the Phase 2 handoff must cover for each shell. | Expand Step 1's C4 output specification to list required behavior branches; this ensures Phase 2 implementers receive a complete per-shell contract rather than a high-level matrix label. |
| 5 | Minor | Required Context | `AGENTS.md` is not listed in Required Context even though C2 (PowerShell init guidance) cites it as key evidence and the implementer must correctly interpret the `Out-String` single-script-block constraint. | Add `AGENTS.md` (PowerShell init gotcha section) to the Required Context table as a read-only reference. |
| 6 | Note | Reality Check | `replaceStart`/`replaceEnd` offset unit (byte vs. character) is not documented in the plan's Reality Check or in the C5 contract output requirement. The current code uses `usize` without annotation. | Explicitly require the C5 contract output to state the offset unit and its implications for non-ASCII paths. |

## Recommendations

1. **[Major — fix before execution]** Replace the two-file positive assertions in the verify command with PRD-specific checks that will fail until `docs/cd-extras-cli-prd.md` is actually updated. At minimum: `rg -n -F "dx stack" docs/cd-extras-cli-prd.md >/dev/null`, `rg -n -F "dx navigate" docs/cd-extras-cli-prd.md >/dev/null`, and `rg -n -F "Out-String" docs/cd-extras-cli-prd.md >/dev/null`. Without this fix, the verify command cannot confirm the largest Phase 1 deliverable was completed.

2. **[Minor — recommended before execution]** Expand Step 1's `Approved C4 Target Behavior` specification to enumerate the required behavior branches for each shell: menu disabled, successful replace, cancel with query change, noop/error fallback, and no-TTY/degraded. This converts the C4 output from a label into a genuine Phase 2 handoff specification.

3. **[Minor — can be addressed during execution]** Acknowledge the Historical Background / absence-check tension in either the verify command description or Step 3 considerations so the implementer knows whether verbatim legacy command names in historical sections are allowed.

4. **[Minor — can be addressed during execution]** Add `AGENTS.md` to Required Context for the PowerShell `Out-String` gotcha, and require the C5 contract output to state the byte-offset unit interpretation for `replaceStart`/`replaceEnd`.
