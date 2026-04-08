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

**Verdict**: Ready (with minor reservations)

The plan is execution-ready for a docs/contracts-only phase. Scope alignment is tight, code anchors have been verified against actual source files, and the plan correctly defers runtime changes to Phase 2. The verify command is syntactically valid and runnable, and its negative checks are well-chosen. Two minor weaknesses are noted: (1) the verify command does not confirm the "no-candidates" C4 branch or the "dependency-free" C5 constraint explicitly, leaving those requirements partially unverified at acceptance; (2) the shell-hook-guarding.md refresh is checked by only one positive marker ("dx stack undo"), meaning a superficial edit could pass acceptance while leaving the menu-fallback and navigation-wrapper wording unaddressed.

## Scope Alignment

### Findings

- **[Note]** Scope is tightly constrained to docs/contracts artifacts (`phase-1-conflict-inventory.md`, `docs/shell-hook-guarding.md`, `docs/cd-extras-cli-prd.md`, and conditionally `docs/configuration.md`). No runtime code changes are included, correctly matching the phase-1 gate.
- **[Note]** Affected modules table is accurate. `docs/configuration.md` is properly marked conditional (only if contradiction found). No scope creep detected.
- **[Note]** The plan correctly flags that C4 and C5 are "Code + Docs" in the conflict inventory but limits Phase 1 to the contract/decision record, deferring runtime code changes to Phase 2. This is consistent with phase-1.md's explicit exclusions.
- **[Minor]** The phase-1.md deliverables do not explicitly enumerate the "Approved C4 Target Behavior" and "Approved C5 Payload/Escaping Contract" labeled outputs as distinct deliverables. The impl plan promotes these to first-class outputs (correctly), but this is additive precision beyond what the phase doc states. No conflict; worth noting for phase artifact consistency.

## Technical Feasibility

### Findings

- **[Note]** Code anchors verified against actual source:
  - `src/cli/mod.rs:20-54` `Commands` enum confirmed: `Resolve`, `Init`, `Complete`, `Navigate`, `Bookmarks`, `Stack`, `Menu` — no `add`, `undo`, `redo` at top level. ✓
  - `src/cli/stacks.rs:12-17` `StackCommand` confirmed as `Push`/`Undo`/`Redo` under `dx stack`. ✓
  - `src/menu/buffer.rs:10-14` `ParsedBuffer.replace_start`/`replace_end` confirmed as byte offsets. ✓
  - `src/menu/action.rs:6-19` `MenuAction` schema confirmed: `action` tag + `replaceStart`/`replaceEnd`/`value` camelCase fields. ✓
  - `src/cli/menu.rs:245-281` Cancel-with-changed-query emits `replace`; unchanged/no-TTY emits `noop`. ✓
  - `src/hooks/zsh.rs:307-311` Zsh noop/error path resets prompt without native fallback — confirmed divergence from Bash/Fish. ✓
  - `src/hooks/bash.rs:304-330` Bash falls back to native completion on non-replace. ✓
  - `src/hooks/fish.rs:238-247` Fish falls back to `commandline -f complete` on error/non-replace. ✓
  - `src/hooks/pwsh.rs:336-347` PowerShell uses `ConvertFrom-Json` + Replace API. ✓
  - `src/hooks/mod.rs:25-37` Canonical shell list (`bash, zsh, fish, pwsh`) and generator dispatch confirmed. ✓
- **[Note]** The Reality Check mismatch inventory accurately identifies stale wording in `docs/shell-hook-guarding.md` line 41 ("dx undo"/"dx redo") and the legacy PRD command surface. These have been verified against current source.
- **[Note]** `src/menu/buffer.rs:10-14` correctly establishes that `replace_start`/`replace_end` are byte offsets (not character offsets). The impl plan correctly propagates this into the C5 contract requirement. The "offset-unit" keyword check in the verify command will enforce this is documented.
- **[Minor]** The plan anchors `src/menu/tui.rs:82-116` (`CleanupGuard::drop`) and `src/menu/tui.rs:139-220` (`select`) for TUI cleanup and no-candidate/cancel flow. Verified: `CleanupGuard` uses `terminal::disable_raw_mode()` in `drop`, and `select` returns `Some(MenuResult::Cancelled{...})` immediately when `initial_candidates.paths.is_empty()` (no-candidates path). The no-candidates case is distinct from no-TTY (which returns `None`). The impl plan notes "and no-candidates when distinct" in Step 1's C4 requirement — this is correct and code-grounded. However, the verify command does not check for a "no-candidates" keyword. See Testing Plan findings.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries with code-grounded decisions | Yes | Yes | See Minor: "no-candidates" branch and "dependency-free" C5 constraint required by step prose but not verified by the verify command |
| 2 | Refresh shell-hook contract docs to current semantics | Yes | Yes | See Minor: verify command only checks one positive marker ("dx stack undo") for this step's output |
| 3 | Rewrite PRD to current command surface and architecture baseline | Yes | Yes | Verify checks cover main PRD markers well; sole gap is that bookmarks section currently has "dx bookmarks add" — after refresh, this will still appear unless implementer is careful about the `! rg -F "dx add"` check (see Reference Consistency) |
| 4 | Cross-check docs against source and phase scope boundaries | Yes | Yes | Completion-pass step is substantive; done-signal criteria are explicit |

## Required Context Assessment

### Missing Context

- **[Note]** `src/cli/complete.rs` is listed in Required Context but the relevant anchor (`run_navigate`, lines 146-166) is correctly cited. No gap.
- **[Note]** No notable missing files. All hook generators (`bash.rs`, `zsh.rs`, `fish.rs`, `pwsh.rs`) are included; `src/menu/buffer.rs` and `src/menu/action.rs` anchor the C5 contract requirements appropriately.

### Unnecessary Context

- **[Note]** `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` is listed as required context with the note that Phase 1 "must not mark unexecuted scenarios as passed." This is a boundary constraint, not an active editing target in Phase 1. Its inclusion is justified as a guard against scope bleed, but it is more a constraint reference than an active read requirement. Low-concern note only.

## Testing Plan Assessment

### Test Integrity Check

- The plan explicitly states no Rust/unit/integration tests are modified, removed, skipped, or weakened during Phase 1. ✓
- Phase 2 and Phase 3 are explicitly called out as the phases that must satisfy the automated tests + four-shell smokes verification bar. ✓
- `shell-smoke-matrix.md` is explicitly protected from being marked passed in Phase 1. ✓
- No existing tests are touched; this is correct and clearly stated.

### Test Gaps

- **[Minor]** The verify command checks for `"no-TTY/degraded"` as a C4 branch label but does NOT check for `"no-candidates"`. Step 1 explicitly requires the C4 contract to enumerate "no-candidates when distinct" as a branch. An implementer who writes the inventory without the word "no-candidates" (using e.g. "empty candidate list" or "zero-candidate path") will pass the verify command but violate the step's stated requirement. The verify command should include: `rg -n -F "no-candidates" plans/.../phase-1-conflict-inventory.md >/dev/null`.

- **[Minor]** The verify command checks for `"value escaping"` (C5 requirement) but does NOT check for `"dependency-free"`. Step 1 requires the C5 contract to define "dependency-free parsing constraints." An implementer could write a C5 section that covers all checked keywords but omits the dependency-free constraint. Recommended addition: `rg -n -F "dependency-free" plans/.../phase-1-conflict-inventory.md >/dev/null`.

- **[Minor]** For Step 2 (shell-hook-guarding.md refresh), the verify command has only ONE positive marker: `rg -n -F "dx stack undo" docs/shell-hook-guarding.md`. The step requires also refreshing menu fallback semantics (cross-shell notes, target alignment guidance) and navigation wrapper semantics. A minimal edit that adds only "dx stack undo" somewhere in the doc would pass acceptance while leaving the navigation wrapper and menu fallback sections stale. Stronger checking could add: `rg -n -F "dx stack redo" docs/shell-hook-guarding.md >/dev/null` and `rg -n -F "noop" docs/shell-hook-guarding.md >/dev/null`.

- **[Note]** The verify command uses `bash -lc` with `set -euo pipefail`. Verified: bash -lc runs from `/Users/nick/code/personal/dx` in this environment, so relative paths are valid. The `!` negation syntax works correctly under `set -e` in bash. The `| Open |` fixed-string pattern matches the exact pipe-space-Open-space-pipe format in the current inventory table. The command is syntactically valid and terminates correctly.

- **[Note]** The "Lightweight sanity check (non-invasive)" in the test table is a manual checklist that effectively mirrors the automated verify command's structural checks, plus a requirement to cite concrete line numbers. This is a useful belt-and-suspenders step for an implementer but is not machine-verifiable. Its presence is appropriate given the docs-only nature of Phase 1.

### Real-World Testing

Not applicable for Phase 1 (docs/contracts-only scope). The plan correctly defers real-world shell smoke testing to Phase 3. No shell execution or binary testing is expected in Phase 1 output. The `shell-smoke-matrix.md` remains untouched and all rows stay "Pending."

## Reference Consistency

### Findings

- **[Minor]** The verify command negative check `! rg -n -F "dx add" docs/cd-extras-cli-prd.md docs/shell-hook-guarding.md` may produce a false failure if the refreshed PRD retains the bookmarks section with `dx bookmarks add`. The literal string `"dx add"` is NOT a substring of `"dx bookmarks add"` (confirmed by testing). However, the current PRD line 23 has `` `dx add "/Users/nick/projects/cd-extras" --session $PID` `` which IS a literal `"dx add"` match. After refresh the implementer must ensure this specific line is removed/rewritten. The plan's Step 3 considerations say to avoid "verbatim obsolete command spellings (`dx add`...)" in refreshed docs, which covers this. Risk is low but the implementer needs to be aware that `dx bookmarks add` is safe while `dx add <path>` is not.

- **[Note]** All file paths in code anchors confirmed to exist:
  - `src/cli/mod.rs` ✓ (79 lines, contains `Commands` enum)
  - `src/cli/complete.rs` ✓
  - `src/cli/stacks.rs` ✓ (StackCommand: Push/Undo/Redo confirmed)
  - `src/cli/init.rs` ✓ (run_init confirmed, Shell::parse covers bash/zsh/fish/pwsh)
  - `src/cli/menu.rs` ✓
  - `src/menu/buffer.rs` ✓ (ParsedBuffer with replace_start/replace_end as byte offsets)
  - `src/menu/action.rs` ✓ (MenuAction enum with replace/noop variants)
  - `src/menu/tui.rs` ✓ (CleanupGuard, select function)
  - `src/hooks/mod.rs` ✓ (supported_list, generate dispatch)
  - `src/hooks/bash.rs` ✓
  - `src/hooks/zsh.rs` ✓
  - `src/hooks/fish.rs` ✓
  - `src/hooks/pwsh.rs` ✓
  - All contracts/verification artifacts ✓

- **[Note]** `docs/shell-hook-guarding.md` line 41 confirmed stale ("dx undo"/"dx redo" wording). The `! rg -n -F "dx undo"` negative check will correctly catch this if not updated. After refresh, the implementer must replace "dx undo"/"dx redo" with "dx stack undo"/"dx stack redo" — and the "dx stack undo" positive check will enforce the new wording is present.

- **[Note]** `docs/cd-extras-cli-prd.md` confirmed to contain all targeted obsolete patterns: `dx add` (line 23), `Invoke-Expression (& dx init pwsh)` (line 75), `complete <type> <word>` (line 69), and top-level `undo`/`redo` in the CLI interface table (lines 64-65, as standalone subcommands not "dx undo"/"dx redo" literally). Note: lines 64-65 show `undo` and `redo` without the `dx` prefix. The negative check `! rg -F "dx undo"` will NOT catch standalone `undo` in the CLI table — but the positive check for `dx stack` presence combined with the full PRD rewrite should address this indirectly.

## Reality Check Validation

### Findings

- **[Note]** Reality Check section is honest and grounded. The four mismatches listed (stale "dx undo"/"dx redo" in shell-hook-guarding.md, legacy PRD command surface, Zsh runtime divergence, and parsing asymmetry) are all confirmed against actual source code. No inflation or omission detected.
- **[Note]** The plan correctly notes that "conflict inventory may intentionally retain legacy command strings as historical conflict evidence" — this is important to protect the implementer from confusing historical evidence rows with refreshed docs targets when running the negative stale-string checks.
- **[Note]** The plan correctly identifies that `src/hooks/zsh.rs:307-311` shows the Zsh divergence (reset-prompt without native fallback) and explicitly defers runtime correction to Phase 2. The Phase 2 handoff is documented in Step 1 as "Phase 2 implementation constraints" in each conflict row.
- **[Minor]** The Reality Check does not note that the PRD's section 3.3 title ("Frecency & Recent Locations (`add`, `recent`, `frecent`)") uses legacy frecency command names. The verify command only checks for absence of "dx add" as a literal command string; the section heading using `add` as a method name (not "dx add") will not trigger the negative check. After PRD rewrite the section title will need updating, but the verify command won't enforce this. Low risk since the full PRD rewrite is required anyway.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Minor | Testing Plan / Verify Command | `no-candidates` required as C4 branch by Step 1 but not checked in verify command — an implementer could omit this branch and pass acceptance | Add `rg -n -F "no-candidates" plans/.../phase-1-conflict-inventory.md >/dev/null` to the verify command, or document that "no-candidates" behavior is covered implicitly under "noop/error fallback" |
| 2 | Minor | Testing Plan / Verify Command | `dependency-free` required as C5 constraint by Step 1 but not checked in verify command | Add `rg -n -F "dependency-free" plans/.../phase-1-conflict-inventory.md >/dev/null` to the verify command |
| 3 | Minor | Testing Plan / Verify Command (Step 2 coverage) | Only one positive marker checks `docs/shell-hook-guarding.md` refresh ("dx stack undo"); menu-fallback wording and navigation wrapper semantics updates have no positive acceptance signal | Add `rg -n -F "dx stack redo" docs/shell-hook-guarding.md >/dev/null` at minimum; consider adding a noop/fallback wording check |
| 4 | Minor | Reference Consistency | The PRD's standalone `undo`/`redo` entries in the CLI table (lines 64-65) will not be caught by `! rg -F "dx undo"` — the implementer must rewrite the full CLI table but the verify command won't enforce removal of bare `undo`/`redo` entries | Low risk given full PRD rewrite is required; note for implementer awareness |
| 5 | Note | Step Quality | "no-candidates" C4 branch is properly code-grounded (tui.rs returns Cancelled with changed_query=false when paths empty, which maps to noop) but Step 1 says "(and no-candidates when distinct)" — the parenthetical phrasing introduces minor ambiguity about whether it must be a distinct labeled sub-entry or can be discussed inline | Clarify in Step 1 whether no-candidates must appear as a distinct named branch or can be collapsed with noop/error fallback semantics |
| 6 | Note | Required Context | `shell-smoke-matrix.md` is required context primarily as a boundary constraint (Phase 1 must not mark it done), not an active editing target | No action required; the constraint is properly stated in Test Integrity section |
| 7 | Note | Scope Alignment | Phase-1.md deliverables list does not explicitly call out "Approved C4 Target Behavior" and "Approved C5 Payload/Escaping Contract" as named outputs — impl plan adds this precision correctly | No action required; the precision is an improvement, not a conflict |

## Recommendations

1. **[Minor #1 — highest value]** Add `rg -n -F "no-candidates"` check to the verify command for the conflict inventory, or add a clarifying note that "no-candidates" behavior is explicitly subsumed under the "noop/error fallback" keyword (acceptable if documented). Currently, Step 1 requires this branch to be enumerated and the verify command silently ignores it.

2. **[Minor #2]** Add `rg -n -F "dependency-free"` check to the verify command for the conflict inventory. The C5 contract must document dependency-free parsing constraints and nothing in the current verify command enforces this.

3. **[Minor #3]** Add at minimum `rg -n -F "dx stack redo" docs/shell-hook-guarding.md >/dev/null` to the verify command. The Step 2 output for `docs/shell-hook-guarding.md` is only validated by a single positive marker; adding the redo check and optionally a menu-fallback wording check would make acceptance more robust.

4. **[Note #4]** Brief implementer note in Step 3: bare `undo` and `redo` (without `dx` prefix) in the PRD CLI table won't be caught by the `! rg -F "dx undo"` negative check — the implementer must ensure the full CLI table is replaced with current command surface, not just the `dx add`/`dx undo` qualified forms.
