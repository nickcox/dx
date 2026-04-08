---
type: review
entity: implementation-plan-review
plan: "architectural-doc-code-alignment"
phase: 1
status: final
reviewer: "reviewer-1"
created: "2026-04-08"
---

# Implementation Plan Review: Phase 1 - Refresh Architecture Docs and Contracts (Recheck)

> Reviewing [Phase 1 Implementation Plan](../implementation/phase-1-impl.md)
> Against [Phase 1 Scope](../phases/phase-1.md) and [Plan](../plan.md)
> Second-pass quality gate: assessing whether prior findings are now resolved, whether the verify command is strong enough for Phase 1 acceptance, and whether the plan is execution-ready while staying docs/contracts-only.

## Overall Assessment

**Verdict**: Needs Revision

The revised plan has meaningfully addressed the prior review's findings: the conflict-inventory open-row check is now in the verify command, required context files are complete, and the C4/C5 required contract outputs are explicit in Step 1. However, a **Critical flaw** in the verify command was introduced in the revision that was not present before: the absence-check lines (`! rg -n -F "dx undo" ...`, `! rg -n -F "dx add" ...`, `! rg -n -F "Invoke-Expression (& dx init pwsh)" ...`, `! rg -n -F "dx complete <type> <word>" ...`) all include `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` in the file list. That file describes those exact obsolete strings in its C1/C2/C3 conflict rows and will always contain them — the conflict inventory is a description of the problems, not a target to be cleaned of them. This makes it **structurally impossible for items 4–8 of the verify command to pass**, even after a fully correct implementation. The verify command will produce a permanent false-failure after a correct Phase 1 implementation. This must be fixed before execution.

## Scope Alignment

### Findings

- **[Pass]** The plan remains correctly scoped to docs/contracts only. Steps 1–4 all target plan artifacts and docs; no runtime changes are implied.
- **[Pass]** All four deliverables from phase-1.md are covered: conflict inventory (Step 1), `docs/shell-hook-guarding.md` (Step 2), `docs/cd-extras-cli-prd.md` (Step 3), and conditional `docs/configuration.md` (Step 2 considerations).
- **[Pass]** Step 1 now explicitly requires (a) an approved cross-shell noop/error/replace matrix and (b) the approved shell-to-`dx menu` payload/escaping contract. This addresses the previously missing C4/C5 output specification.
- **[Pass]** Step 4 completion signals are more concrete than the prior version: four explicit done-checks are listed.
- **[Minor]** The Affected Modules table still marks `docs/configuration.md` as conditional and the plan body still lacks a clear trigger criterion. The prior reviewer flagged this as a minor gap; it remains un-resolved. However, this is acceptable at Phase 1's docs-only scope.

## Technical Feasibility

### Findings

- **[Pass]** The docs-only approach remains technically appropriate. No code changes are proposed, so there is no risk of test breakage or behavior regression.
- **[Pass]** The Open Decisions table correctly documents the target-vs-current documentation strategy (Option B). No change needed here.
- **[Pass]** The plan correctly frames Zsh fallback divergence as a Phase 2 implementation item rather than trying to resolve it in Phase 1 docs.
- **[Note]** The Reality Check now includes `src/cli/init.rs`, `src/menu/tui.rs`, and `src/hooks/mod.rs` — all missing from the prior version. This materially improves the coverage.
- **[Note]** The parsing asymmetry between Bash/Zsh/Fish (string slicing) and PowerShell (`ConvertFrom-Json`) is still correctly identified as requiring C5 contract documentation rather than unification. Verified from source: `src/hooks/bash.rs:309-311` uses string slicing; `src/hooks/pwsh.rs:338` uses `ConvertFrom-Json`.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries with code-grounded decisions | Yes | Yes | Step now requires C4/C5 outputs explicitly. No gap. |
| 2 | Refresh shell-hook contract docs to current semantics | Mostly | Yes | Conditional `docs/configuration.md` trigger still undefined, but minor. |
| 3 | Rewrite PRD to current command surface and architecture baseline | Yes | Yes | Clear and well-grounded in current CLI structure. |
| 4 | Cross-check docs against source and phase scope boundaries | Yes | Yes | Four explicit done-checks now make this verifiable. Improvement over prior version. |

- **[Pass]** Steps 1–3 each have clear What/Where/Why, are grounded in real file paths and symbols (all verified in current repo), and are actionable for an implementer.
- **[Pass]** Step 4 now has an explicit done-signal: four listed completion checks, correcting the prior "advisory only" status.

## Required Context Assessment

### Missing Context

- None identified. The revised plan adds `src/cli/init.rs`, `src/menu/tui.rs`, and `src/hooks/mod.rs` to the Required Context table, resolving all prior gaps. All listed files were verified to exist in the repo.

### Unnecessary Context

- `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` is still listed in Required Context with only a constraint reference (Phase 1 must not mark unexecuted scenarios as passed). It is correctly read-only context for Phase 1 but the plan does not interact with it substantively. This is a minor specification over-inclusion, not a problem.

## Testing Plan Assessment

### Test Integrity Check

- **[Pass]** The Test Integrity Constraints section continues to explicitly state no existing Rust/unit/integration tests are modified, removed, skipped, or weakened in Phase 1.
- **[Pass]** Phase 2 and Phase 3 automated test and smoke requirements are preserved and not weakened.
- **[Pass]** The shell-smoke-matrix.md constraint is correctly stated.

### Test Gaps

- **[Critical]** The verify command includes `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` in the file list for all absence-check assertions (`! rg -n -F "dx add" ...`, `! rg -n -F "dx undo" ...`, `! rg -n -F "dx redo" ...`, `! rg -n -F "Invoke-Expression (& dx init pwsh)" ...`, `! rg -n -F "dx complete <type> <word>" ...`). This is fatally wrong because the conflict inventory itself documents the existence of these obsolete strings in conflict rows C1, C2, and C3 — and after Phase 1 implementation the inventory rows will still contain those strings as historical conflict descriptions. Running the verify command post-implementation will produce a permanent exit-1 even on a correct implementation.

  **Confirmed by direct inspection**: `phase-1-conflict-inventory.md` line 11 contains `dx add`, `dx undo`, `dx redo`, and `dx complete <type> <word>` in the C1 row conflict description. Line 12 contains `Invoke-Expression (& dx init pwsh)` in the C2 row. Line 13 contains `dx undo` and `dx redo` in the C3 row. These strings document what Phase 1 must fix in the target docs — they are not themselves target docs that need to be cleaned of these strings.

  **Fix**: Remove `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` from the absence-check file lists (items 4–8). The conflict inventory file does not need to be free of these strings; only `docs/cd-extras-cli-prd.md` and `docs/shell-hook-guarding.md` do.

- **[Major — prior finding now partially resolved]** The prior review flagged that the verify command did not check whether inventory rows remain `Open`. The revised verify command now includes `test "$(rg -n -F "| Open |" ...)" = "0"` as the first check. This directly resolves finding #1 from the prior review. However, the fix introduced the Critical issue described above.

- **[Minor]** The verify command does not include absence checks for `docs/shell-hook-guarding.md` for the `dx undo`/`dx redo` stale wording (only `docs/cd-extras-cli-prd.md` would be fixed by removing the inventory from the file list, but `docs/shell-hook-guarding.md` line 41 also contains `dx undo`/`dx redo`). Running the corrected verify command (with inventory removed from file list) would check both files for `dx undo`/`dx redo` — so this is acceptable coverage once the Critical fix is applied.

- **[Minor — prior finding partially addressed]** The prior review noted the verify command checks only absence of stale strings rather than positive assertions that current commands appear. The revised plan does not add positive assertions (e.g., `rg -n "dx stack" docs/cd-extras-cli-prd.md`). A positive check would increase confidence that the PRD was rewritten (not just stripped), but the Step 4 done-checks provide some compensating coverage.

- **[Note]** The `rg` backslash-escaped-parens issue from the prior review (finding #2) has been addressed: the revised verify command uses `-F` (fixed-strings) flag for all patterns, which ensures literal matching. The pattern `Invoke-Expression (& dx init pwsh)` with `-F` is correct.

### Real-World Testing

No real-world shell testing is planned for Phase 1 — and this is explicitly correct per phase-1.md Excludes. Shell smoke verification is the Phase 3 responsibility. This deferral is not a limitation; it is the correct Phase 1 scoping decision.

## Reference Consistency

### Findings

- **[Pass]** `src/cli/mod.rs:20-54` — Verified. `Commands` enum at lines 19–54 covers `Resolve`, `Init`, `Complete`, `Navigate`, `Bookmarks`, `Stack`, `Menu`. Accurate.
- **[Pass]** `src/cli/complete.rs:13-55` — Verified. `CompleteCommand` enum with modes `Paths`, `Ancestors`, `Frecents`, `Recents`, `Stack` spans lines 12–55. Accurate.
- **[Pass]** `src/cli/stacks.rs:12-17` — Verified. `StackCommand` enum with `Push`, `Undo`, `Redo` at lines 12–17. Accurate.
- **[Pass]** `src/cli/init.rs:3-14` — Verified. `run_init` calls `Shell::supported_list()` and `hooks::generate(shell, command_not_found, menu)`. The supported shell list is `"bash, zsh, fish, pwsh"` (confirmed in `src/hooks/mod.rs:26`). Accurate.
- **[Pass]** `src/menu/action.rs:6-19` — Verified. `MenuAction` enum with `Replace { replace_start, replace_end, value }` and `Noop`, using `#[serde(rename = "replaceStart/replaceEnd")]` camelCase JSON keys. Accurate.
- **[Pass]** `src/cli/menu.rs:245-281` — Verified. `MenuResult::Cancelled` with `changed_query` at lines 245–281. Cancel-with-changed-query emits `replace`; unchanged cancel emits `noop`. Accurate.
- **[Pass]** `src/menu/tui.rs:82-116` — Verified. `CleanupGuard::drop` at lines 88–116 restores cursor visibility, clears menu area rows, and disables raw mode on all exit paths including `/dev/tty` backend. Accurate.
- **[Pass]** `src/hooks/bash.rs:304-330` — Verified. `__dx_try_menu` at 304–315 and `_dx_menu_wrapper` at 317–330. Bash falls back to native completion handlers on non-replace/error. Accurate.
- **[Pass]** `src/hooks/zsh.rs:307-311` — Verified. Lines 307–312 confirm Zsh noop/error path resets prompt and returns without native completion fallback. Plan's claimed divergence is accurate.
- **[Pass]** `src/hooks/fish.rs:238-247` — Verified. Lines 238–247 confirm Fish falls back to `commandline -f complete` on error or non-replace. Accurate.
- **[Pass]** `src/hooks/pwsh.rs:336-347` — Verified. Lines 336–347 confirm PowerShell uses `ConvertFrom-Json` and PSReadLine Replace API. Accurate.
- **[Pass]** `src/hooks/mod.rs:25-37` — Verified. `supported_list()` returns `"bash, zsh, fish, pwsh"` and `generate()` dispatches by shell. Accurate.
- **[Pass]** The Reality Check mismatch: `docs/shell-hook-guarding.md` line 41 reads "Stack-transition wrappers use `dx undo`/`dx redo`" — confirmed stale. Actual hooks use `dx stack undo`/`dx stack redo` (verified via `src/hooks/mod.rs` test at line 103 confirming `"dx stack \"$__dx_undo_or_redo\""`).
- **[Pass]** The `docs/cd-extras-cli-prd.md` obsolete commands (`dx add` line 23, `dx undo` line 64, `dx redo` line 65, `dx complete <type> <word>` line 69, `Invoke-Expression (& dx init pwsh)` line 75) — all confirmed present and stale.

## Reality Check Validation

### Findings

- **[Pass]** The Reality Check now covers 14 code anchors across 8 source files — materially stronger than the 10-anchor prior version. The additions of `src/cli/init.rs`, `src/menu/tui.rs`, and `src/hooks/mod.rs` fill the prior gaps.
- **[Pass]** All noted mismatches were independently verified and are genuine. No false claims found.
- **[Pass]** The fish.rs stack wrapper behavior gap from the prior review (finding #7) is now addressed: the plan includes `src/hooks/fish.rs:238-247` in the Reality Check table.
- **[Pass]** The plan correctly treats the Zsh fallback divergence as Phase 2 implementation work, not a Phase 1 documentation fix.
- **[Pass]** The `src/hooks/mod.rs:25-37` anchor is now present, addressing the prior review's finding #3 about its absence.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Critical | Testing Plan | The verify command includes `phase-1-conflict-inventory.md` in the absence-check file lists for items 4–8. The inventory file documents the stale strings in its conflict rows and will permanently contain them — the verify command structurally cannot exit 0 after a correct implementation. This was introduced while trying to fix prior finding #1. | Remove `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` from the `! rg -F` file lists for items 4–8. Keep it only in the `Open` rows count check (item 1) and the C4/C5 marker checks (items 2–3). |
| 2 | Minor | Testing Plan | No positive assertions confirm that the PRD was actually rewritten to reference current commands, only that stale strings were removed. A stripped-but-empty PRD would still pass. | Add at least one positive assertion: e.g., `rg -n -F "dx stack" docs/cd-extras-cli-prd.md` and `rg -n -F "dx navigate" docs/cd-extras-cli-prd.md`. |
| 3 | Minor | Testing Plan | The lightweight sanity check (second test in the table) remains non-executable as stated: "Confirm refreshed docs still cite existing source anchors" cannot be mechanically verified by the stated procedure. | Convert to a concrete scripted check or enumerate it in Step 4's done-checklist rather than presenting it as a test row. |
| 4 | Note | Prior Findings | Prior finding #1 (Open rows check) is now resolved — the verify command includes the `Open` row count check. | No action — confirmed resolved. |
| 5 | Note | Prior Findings | Prior finding #2 (`rg` parens issue) is now resolved — all patterns use `-F` (fixed-strings). | No action — confirmed resolved. |
| 6 | Note | Prior Findings | Prior finding #3 (`src/hooks/mod.rs` missing from Required Context) is now resolved. | No action — confirmed resolved. |
| 7 | Note | Prior Findings | Prior finding #4 (Step 4 lacks done-signal) is now resolved — Step 4 lists four explicit completion checks. | No action — confirmed resolved. |
| 8 | Note | Prior Findings | Prior finding #5 (stale `dx undo`/`dx redo` not in verify) is partially resolved — `dx undo` and `dx redo` are now in the absence check for `docs/shell-hook-guarding.md` (once the Critical fix is applied). | Confirm after Critical fix is applied. |

## Recommendations

1. **[Critical — must fix before execution]** Remove `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` from the `! rg -n -F` file lists in verify command items 4–8. The conflict inventory is a description of problems to fix — not a target doc to be cleaned. Leaving it in the absence check list makes the verify command permanently fail on a correct implementation. The fix is a one-line edit to the bash verify command.

2. **[Minor — recommended before execution]** Add at least one positive assertion to the verify command confirming current-contract terms appear in the rewritten PRD (e.g., `rg -n -F "dx stack" docs/cd-extras-cli-prd.md >/dev/null`). Absence-only checks are weak for a full PRD rewrite.

3. **[Minor — can be addressed during execution]** Convert the "Lightweight sanity check" test row into a concrete scripted check or fold it into Step 4's done-checklist. As stated, it cannot be mechanically verified and does not add testing rigor.
