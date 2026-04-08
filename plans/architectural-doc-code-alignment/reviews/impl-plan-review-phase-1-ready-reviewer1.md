---
type: review
entity: implementation-plan-review
plan: "architectural-doc-code-alignment"
phase: 1
status: final
reviewer: "reviewer-1"
created: "2026-04-08"
---

# Implementation Plan Review: Phase 1 - Refresh Architecture Docs and Contracts (Ready Check)

> Reviewing [Phase 1 Implementation Plan](../implementation/phase-1-impl.md)
> Against [Phase 1 Scope](../phases/phase-1.md) and [Plan](../plan.md)
> Final readiness check: focus on (1) verify command satisfiability, (2) C4/C5 handoff contract explicitness in Step 1, and (3) execution readiness while remaining docs/contracts-only.

## Overall Assessment

**Verdict**: Ready (with one noted weakness in the verify command that does not block execution)

The plan is execution-ready. The Critical finding from the prior recheck — conflict inventory file included in stale-string absence checks — has been resolved: absence checks (items 20–24) now target only `docs/cd-extras-cli-prd.md` and `docs/shell-hook-guarding.md`. The C4/C5 handoff contract in Step 1 is now explicit enough for Phase 2 to inherit without guessing. One weakness persists: the verify command's absence check for `dx complete <type> <word>` uses a `dx`-prefixed pattern that does not match the actual stale string at line 69 of `docs/cd-extras-cli-prd.md` (which omits the `dx ` prefix). This creates a gap where stale completion language could survive Phase 1 undetected. It is a Minor finding that should be addressed during execution but does not block the plan.

## Scope Alignment

### Findings

- **[Pass]** The plan is correctly scoped to docs/contracts artifacts only. Steps 1–4 produce changes to `plans/.../phase-1-conflict-inventory.md`, `docs/shell-hook-guarding.md`, `docs/cd-extras-cli-prd.md`, and optionally `docs/configuration.md`. No runtime behavior changes are proposed.
- **[Pass]** All four deliverables from `phases/phase-1.md` are addressed: conflict inventory (Step 1), `docs/shell-hook-guarding.md` (Step 2), `docs/cd-extras-cli-prd.md` (Step 3), and conditional `docs/configuration.md` (Step 2 considerations).
- **[Pass]** Step 1 explicitly requires the C4/C5 labeled contract outputs that Phase 2 can inherit. Scope is correctly bounded: no runtime change deliverables in Phase 1.
- **[Pass]** Step 4 provides four explicit done-checks, preventing undetected scope bleed into runtime changes.
- **[Minor]** The Affected Modules table marks `docs/configuration.md` as conditional with "update only if needed" but no trigger criterion is specified. This has been carried across multiple review iterations without resolution. The current `docs/configuration.md` uses `dx complete <mode>` syntax at line 72 which is already current-contract — so no update is likely needed. This is acceptable at docs-only scope.

## Technical Feasibility

### Findings

- **[Pass]** The docs-only approach is technically sound. No code changes are proposed, so there is zero risk of test breakage or runtime regression.
- **[Pass]** The plan correctly frames Zsh noop/error fallback divergence as a Phase 2 runtime implementation item, not a Phase 1 documentation fix. This accurately reflects current code: `src/hooks/zsh.rs:307-312` confirms Zsh's divergent behavior (`zle reset-prompt` without native completion fallback) while `src/hooks/bash.rs:317-330` and `src/hooks/fish.rs:238-247` confirm Bash/Fish fall back to native completion.
- **[Pass]** The Open Decisions table (document current divergence vs document intended aligned target) is well-reasoned: choosing to document intended aligned behavior (Option B) while deferring runtime convergence to Phase 2 is the correct Phase 1 approach.
- **[Pass]** C5 parsing asymmetry (Bash/Zsh/Fish string-slicing vs PowerShell `ConvertFrom-Json`) is correctly identified as requiring explicit contract documentation rather than immediate unification. The plan correctly requires the `Approved C5 Payload/Escaping Contract` to define `replaceStart`/`replaceEnd` offset-unit interpretation without mandating parser unification.
- **[Note]** `docs/configuration.md` is already substantially current-contract (uses `dx complete <mode>` at line 72, documents `DX_MENU`, `DX_SESSION`, etc. correctly). No contradiction requiring its update was found during codebase inspection.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries with code-grounded decisions | Yes | Yes | C4/C5 outputs are explicitly named and enumerated with required branches. Clear handoff contract. |
| 2 | Refresh shell-hook contract docs to current semantics | Mostly | Yes | Conditional `docs/configuration.md` trigger still not specified, but current doc is likely already correct. |
| 3 | Rewrite PRD to current command surface and architecture baseline | Yes | Yes | Clear negation list of obsolete strings; Step 1 must-not list is complete except for the `complete <type> <word>` prefix gap. |
| 4 | Cross-check docs against source and phase scope boundaries | Yes | Yes | Four explicit completion signals make this verifiable. Not advisory-only. |

- **[Pass]** Step 1 is the key handoff step: it requires `Approved C4 Target Behavior` with per-shell branches (Bash/Zsh/Fish/PowerShell) for menu-disabled, successful replace/select, cancel-with-query-change, noop/error fallback, no-TTY/degraded, and no-candidates when distinct. It also requires `Approved C5 Payload/Escaping Contract` with payload fields, `replaceStart`/`replaceEnd` offset-unit interpretation, `value` escaping expectations, and dependency-free parsing constraints. This is explicit and sufficient for Phase 2.
- **[Pass]** Step 3 includes an explicit "must not appear verbatim" list covering `dx add`, top-level `dx undo`, `dx redo`, `dx complete <type> <word>`, and `Invoke-Expression (& dx init pwsh)`. All five are verified stale in the current PRD.
- **[Minor]** Step 3's prohibition on `dx complete <type> <word>` (with `dx ` prefix) does not match the actual stale string at PRD line 69: `  complete <type> <word>  Generates completion options for the shell hook` — this form lacks the `dx ` prefix. An implementer following the step literally might not flag this line for removal.

## Required Context Assessment

### Missing Context

- None identified. All 14 entries in the Required Context table were verified to exist in the current codebase. All source file anchors match current code.

### Unnecessary Context

- `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` is included as a constraint reference (Phase 1 must not mark unexecuted scenarios as passed). This is a minor specification over-inclusion — the file is read-only for Phase 1. Not a problem.

## Testing Plan Assessment

### Test Integrity Check

- **[Pass]** The Test Integrity Constraints section explicitly states no existing Rust/unit/integration tests are modified, removed, skipped, or weakened during Phase 1. This is correct for a docs-only phase.
- **[Pass]** Phase 2 and Phase 3 automated test and smoke requirements are preserved and not weakened.
- **[Pass]** The `shell-smoke-matrix.md` constraint is correctly stated: Phase 1 must not mark unexecuted scenarios as passed.

### Verify Command Analysis

**Primary verdict: Satisfiable after correct implementation.** The verify command is structurally sound and will exit 0 after a correct Phase 1 implementation. The prior Critical flaw (conflict inventory in absence check file lists) has been resolved.

**Detailed check-by-check analysis:**

| Check # | Pattern | Target File(s) | Satisfiable? | Notes |
|---------|---------|----------------|--------------|-------|
| 2 | `! rg "| Open |"` | conflict-inventory.md | Yes | Open rows currently exist; will pass after C1-C5 resolved/deferred. |
| 3 | `rg "Approved C4 Target Behavior"` | conflict-inventory.md | Yes | Not present now; required by Step 1. |
| 4 | `rg "menu-disabled"` | conflict-inventory.md | Yes | Not present now; required by Step 1. |
| 5 | `rg "successful replace/select"` | conflict-inventory.md | Yes | Not present now; required by Step 1. |
| 6 | `rg "cancel-with-query-change"` | conflict-inventory.md | Yes | Not present now; required by Step 1. |
| 7 | `rg "noop/error fallback"` | conflict-inventory.md | Yes | Not present now; required by Step 1. |
| 8 | `rg "no-TTY/degraded"` | conflict-inventory.md | Yes | Not present now; required by Step 1. |
| 9 | `rg "Approved C5 Payload/Escaping Contract"` | conflict-inventory.md | Yes | Not present now; required by Step 1. |
| 10 | `rg "replaceStart"` | conflict-inventory.md | Yes | Not present now; required by Step 1 (C5 payload fields). |
| 11 | `rg "replaceEnd"` | conflict-inventory.md | Yes | Not present now; required by Step 1 (C5 payload fields). |
| 12 | `rg "offset-unit"` | conflict-inventory.md | Yes | Not present now; required by Step 1 (C5 offset contract). |
| 13 | `rg "escaping"` | conflict-inventory.md | **Weak** | Already present at line 21 (exit criteria). Passes pre- and post-implementation. Does not verify C5 contract was added; it only confirms the word exists. |
| 14 | `rg "dx stack"` | PRD | Yes | Not present now; Step 3 adds it. |
| 15 | `rg "dx navigate"` | PRD | Yes | Not present now; Step 3 adds it. |
| 16 | `rg "dx complete paths"` | PRD | Yes | Not present now; Step 3 adds it. |
| 17 | `rg "dx menu"` | PRD | Yes | Not present now; Step 3 adds it. |
| 18 | `rg "Out-String"` | PRD | Yes | Not present now; Step 3 adds it. |
| 19 | `rg "dx stack undo"` | shell-hook-guarding.md | Yes | Not present now; Step 2 adds it. |
| 20 | `! rg "dx add"` | PRD + shell-hook-guarding.md | Yes | Present in PRD line 23; Step 3 removes it. |
| 21 | `! rg "dx undo"` | PRD + shell-hook-guarding.md | Yes | Present in shell-hook-guarding.md line 41; Step 2 removes it. |
| 22 | `! rg "dx redo"` | PRD + shell-hook-guarding.md | Yes | Present in shell-hook-guarding.md line 41; Step 2 removes it. |
| 23 | `! rg "Invoke-Expression (& dx init pwsh)"` | PRD + shell-hook-guarding.md | Yes | Present in PRD line 75; Step 3 removes it. Note: shell-hook-guarding.md line 73 has `Invoke-Expression ((& dx init pwsh --menu \| Out-String))` — the absence check uses `-F` fixed-string and correctly does NOT match this correct form. |
| 24 | `! rg "dx complete <type> <word>"` | PRD + shell-hook-guarding.md | **Weak** | PRD line 69 contains `complete <type> <word>` WITHOUT the `dx ` prefix. The check passes pre-implementation (no match), giving false assurance. Stale content at PRD line 69 could survive undetected. |

**Summary of verify command coverage:**
- 22 of 24 checks are satisfiable and correctly test what they claim.
- Check 13 (`escaping`) is a weak positive: it passes pre-implementation because the exit criteria section already contains the word. It does not prove C5 contract content was added.
- Check 24 (`! rg "dx complete <type> <word>"`) is a weak absence check: the actual stale string at PRD line 69 lacks the `dx ` prefix, so the check passes regardless of whether that stale line is removed.

### Test Gaps

- **[Minor]** Check 13 (`rg "escaping"`) already passes pre-implementation because the word appears in the exit criteria section of the conflict inventory. It does not meaningfully verify that C5 contract content was added to the decision rows. A tighter probe such as `rg "Approved C5 Payload/Escaping Contract"` (already present as check 9) subsumes this, but check 13 provides no additional signal.

- **[Minor]** Check 24 (`! rg "dx complete <type> <word>"`) misses the actual stale pattern at PRD line 69 (`complete <type> <word>` without `dx ` prefix). An implementer rewriting the PRD should remove this line, but the verify command does not catch a failure to do so. The correct absence check would be `! rg -n -F "complete <type> <word>" docs/cd-extras-cli-prd.md` (without `dx ` prefix), or more broadly: `! rg -n -F "<type> <word>" docs/cd-extras-cli-prd.md`.

- **[Note]** No positive assertion verifies that `dx resolve` (the existing command for path resolution) is mentioned in the refreshed PRD — the sanity check at line 98 references `dx resolve`, but this is not in the primary verify command. This is a gap in mechanical coverage but is addressed by the manual sanity checklist.

### Real-World Testing

No real-world shell testing is planned for Phase 1 — and this is explicitly correct per `phases/phase-1.md` Excludes. Shell smoke verification is Phase 3's responsibility. The `shell-smoke-matrix.md` records all scenarios as Pending, which is the correct pre-implementation state.

## Reference Consistency

### Findings

- **[Pass]** `src/cli/mod.rs:20-54` — Verified. `Commands` enum at lines 19–54 covers `Resolve`, `Init`, `Complete`, `Navigate`, `Bookmarks`, `Stack`, `Menu`. Line range accurate.
- **[Pass]** `src/cli/complete.rs:13-55` — Verified. `CompleteCommand` enum spans lines 12–55 with `Paths`, `Ancestors`, `Frecents`, `Recents`, `Stack` variants. Line range accurate.
- **[Pass]** `src/cli/complete.rs:146-166` — Verified. `run_navigate` function at lines 146–166. Correct anchor.
- **[Pass]** `src/cli/stacks.rs:12-17` — Verified. `StackCommand` enum at lines 12–17 with `Push`, `Undo`, `Redo`. Accurate.
- **[Pass]** `src/cli/init.rs:3-14` — Verified. `run_init` function at lines 3–14, calling `Shell::parse`, `Shell::supported_list`, and `hooks::generate`. Accurate. The supported shells (`bash, zsh, fish, pwsh`) are confirmed by `src/hooks/mod.rs`.
- **[Pass]** `src/menu/action.rs:6-19` — Verified. `MenuAction` enum with `Replace { replace_start, replace_end, value }` and `Noop` at lines 6–19. The `#[serde(rename = "replaceStart/replaceEnd")]` camelCase annotation is present. Serialization is confirmed by tests at lines 43–68. Accurate.
- **[Pass]** `src/cli/menu.rs:245-281` — Verified. `MenuResult::Cancelled` handling at lines 245–272: `changed_query` path emits `replace`; unchanged cancel emits `noop`. TUI unavailable path (`None`) emits `noop` at lines 274–279. Accurate.
- **[Pass]** `src/menu/tui.rs:82-116` — Verified. `CleanupGuard::drop` at lines 88–116 restores cursor, clears menu area rows, and calls `terminal::disable_raw_mode()` on all exit paths for both tty-backend and stderr paths. Accurate.
- **[Pass]** `src/menu/tui.rs:139-220` — Verified. `select` function at lines 139–220 covers no-candidate short-circuit (returns `Some(Cancelled)`), single-candidate auto-select (returns `Some(Selected)`), and the main TUI path. Accurate.
- **[Pass]** `src/hooks/mod.rs:25-37` — Confirmed via context: `supported_list()` and `generate()` dispatch by shell. Accurate.
- **[Pass]** `src/hooks/bash.rs:304-330` — Verified. `__dx_try_menu` at 304–315 and `_dx_menu_wrapper` at 317–330. Bash falls back to native completion (`_dx_complete_paths`, etc.) when menu returns non-replace or errors. Accurate.
- **[Pass]** `src/hooks/zsh.rs:307-311` — Verified. Lines 307–312 confirm Zsh noop/error path does `zle reset-prompt; return` without native completion fallback. This is the documented divergence. Accurate.
- **[Pass]** `src/hooks/fish.rs:238-247` — Verified. Lines 238–247 confirm Fish calls `commandline -f complete` on error (`test $status -ne 0`) and on non-replace (`not string match -q '*"action":"replace"*'`). Accurate.
- **[Pass]** `src/hooks/pwsh.rs:336-347` — Verified. Lines 336–347 confirm PowerShell uses `$json | ConvertFrom-Json` (line 338) and `PSConsoleReadLine::Replace` (line 346). Falls back to `PSConsoleReadLine::TabCompleteNext` on non-replace. Accurate.
- **[Pass]** `docs/shell-hook-guarding.md` line 41 stale wording (`dx undo`/`dx redo`) — confirmed present and stale. All four hook generators use `dx stack undo`/`dx stack redo`.
- **[Pass]** `docs/cd-extras-cli-prd.md` obsolete strings confirmed present:
  - Line 23: `dx add` (component interaction example)
  - Line 64: `undo` (CLI interface table)
  - Line 65: `redo` (CLI interface table)
  - Line 69: `complete <type> <word>` (without `dx ` prefix — missed by check 24)
  - Line 75: `Invoke-Expression (& dx init pwsh)` (PowerShell init)

## Reality Check Validation

### Findings

- **[Pass]** The Reality Check section covers 14 code anchors across 8 source files. All anchors were independently verified against the current codebase. No fabricated or stale anchor references were found.
- **[Pass]** All noted mismatches are genuine and confirmed:
  - `docs/shell-hook-guarding.md` line 41 stale stack command wording (C3)
  - `docs/cd-extras-cli-prd.md` obsolete command surface (C1/C2)
  - Zsh fallback divergence from Bash/Fish/PowerShell (C4) — confirmed from source
  - Bash/Zsh/Fish string-slicing vs PowerShell JSON parsing (C5) — confirmed from source
  - `src/menu/tui.rs` TUI cleanup behavior documented for contract purposes only (Phase 1 must not alter TUI code)
- **[Pass]** The Reality Check correctly defers Zsh fallback unification as a Phase 2 implementation item. No scope bleed into Phase 1 docs.
- **[Pass]** The note about conflict inventory intentionally retaining legacy strings (as historical conflict evidence) correctly explains why stale-string removal checks exclude the inventory file.
- **[Minor]** The Reality Check does not note that PRD line 69's stale completion command form lacks the `dx ` prefix (matching the gap in check 24). The plan's Step 3 prohibition covers `dx complete <type> <word>` (with prefix), but the actual stale string differs slightly. An implementer working from the Reality Check alone might not catch this variant.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Minor | Testing Plan / Verify Command | Check 24 (`! rg "dx complete <type> <word>"`) misses the actual stale string at PRD line 69, which reads `complete <type> <word>` WITHOUT the `dx ` prefix. The absence check passes pre-implementation (false assurance), meaning stale completion language could survive Phase 1 undetected. | Change check 24 to: `! rg -n -F "complete <type> <word>" docs/cd-extras-cli-prd.md >/dev/null` (drop the `dx ` prefix). Alternatively, use `! rg -n -F "<type> <word>" docs/cd-extras-cli-prd.md >/dev/null` to catch both variants. |
| 2 | Minor | Testing Plan / Verify Command | Check 13 (`rg "escaping"`) is a false-positive pre-implementation: the word "escaping" already appears at line 21 of the conflict inventory (exit criteria section). It does not verify that C5 contract content was actually added. | Either remove check 13 (it adds no signal beyond check 9 `"Approved C5 Payload/Escaping Contract"`) or replace it with a tighter probe like `rg "value escaping expectations"` that requires C5 decision text, not just the exit criteria. |
| 3 | Minor | Step Quality | Step 3's prohibition list uses `dx complete <type> <word>` (with `dx ` prefix) but the actual stale string at PRD line 69 lacks the prefix. An implementer following the list literally might not flag this line for removal. | Add a note in Step 3 clarifying that the stale `complete <type> <word>` at PRD line 69 (in the CLI interface table, without `dx ` prefix) must also be removed or rewritten. |
| 4 | Note | Testing Plan | The manual sanity check (second test row) references `dx resolve` as a required term in the refreshed PRD, but this assertion is not in the primary verify command. Post-implementation, a reviewer would need to manually check that the refreshed PRD still mentions the current `dx resolve` command. | Add `rg -n -F "dx resolve" docs/cd-extras-cli-prd.md >/dev/null` to the primary verify command. The current PRD mentions it at line 20 in a context example; the refreshed PRD should retain it as a first-class command reference. |
| 5 | Note | Prior Findings | All Critical and Major findings from the prior recheck have been resolved. The conflict inventory is no longer in the absence-check file lists. The `Open` rows check is correctly structured. The `rg -F` fixed-string flag is used throughout. | No action required. Confirmed resolved. |

## Recommendations

1. **[Minor — should fix before execution]** Correct the stale-string absence check for the completion command pattern. Change `! rg -n -F "dx complete <type> <word>" docs/cd-extras-cli-prd.md docs/shell-hook-guarding.md >/dev/null` to `! rg -n -F "complete <type> <word>" docs/cd-extras-cli-prd.md docs/shell-hook-guarding.md >/dev/null` (removing the `dx ` prefix). This ensures PRD line 69 is caught by the verify command. This is a one-character edit.

2. **[Minor — can address during execution]** In Step 3's considerations, add an explicit note: "PRD line 69 contains `complete <type> <word>` (without `dx ` prefix) in the CLI interface table — this line must also be removed or rewritten." This prevents the implementer from overlooking it.

3. **[Note — optional improvement]** Remove or replace check 13 (`rg "escaping"`) in the verify command to eliminate the pre-implementation false-positive. Check 9 (`rg "Approved C5 Payload/Escaping Contract"`) already provides stronger assurance for the C5 escaping contract. Check 13 provides no additional signal.

4. **[Note — optional improvement]** Add a positive assertion for `dx resolve` to the verify command: `rg -n -F "dx resolve" docs/cd-extras-cli-prd.md >/dev/null`. The manual sanity check references it, but mechanical coverage is stronger.
