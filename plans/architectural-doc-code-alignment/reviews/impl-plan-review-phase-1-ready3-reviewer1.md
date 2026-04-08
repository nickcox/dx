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

**Verdict**: Ready

The plan is execution-ready. Scope is tightly constrained to docs/contracts-only artifacts. Step 1 now makes the C4/C5 handoff requirements sufficiently explicit: it enumerates all four shells, all required C4 branches (including "no-candidates when distinct"), all required C5 payload/offset/escaping/split-I/O elements, and produces named, labeled contract outputs that Phase 2 can reference without reopening scope. The verify command is satisfiable as written and would correctly fail the current pre-implementation state (all five inventory rows are `Open`), meaning it is functional as an acceptance gate. Three minor weaknesses persist and are documented below — none block execution for a docs-only phase.

## Scope Alignment

### Findings

- **[Note]** All four affected modules are docs/contracts artifacts (`phase-1-conflict-inventory.md`, `docs/shell-hook-guarding.md`, `docs/cd-extras-cli-prd.md`, conditionally `docs/configuration.md`). No source files under `src/` are in the modification list. This matches phase-1.md's explicit exclusion of runtime hook/menu changes.
- **[Note]** The plan correctly labels C4 and C5 as `Code + Docs` conflicts in the inventory while limiting Phase 1 work to the contract/decision record. Runtime convergence is deferred to Phase 2, which is consistent with the phase gate.
- **[Minor]** Step 3 prohibits all legacy command spellings ("anywhere in refreshed docs") including `dx add`, `dx undo`, `dx redo`, and the old PowerShell init form. Phase-1.md acceptance criterion AC4 permits "intentionally retained historical context in the PRD" as long as it is "clearly distinguishable from current requirements." The impl plan's stricter prohibition is enforceable but narrower than what the phase allows. An implementer wanting to retain a dated migration section would have to deviate from the impl plan or strip content that the phase explicitly permits. This creates an ambiguity rather than a blocking conflict, but should be resolved by the implementer before editing begins.
- **[Note]** The plan correctly excludes `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` from Phase 1 modification while listing it in Required Context. This is appropriate: Phase 1 must not mark unexecuted smoke scenarios as passed.

## Technical Feasibility

### Findings

- **[Note]** All code anchors have been independently verified against the actual codebase:
  - `src/cli/mod.rs:20-54` — `Commands` enum confirmed; no top-level `add`, `undo`, `redo`. Current surface: `Resolve`, `Init`, `Complete`, `Navigate`, `Bookmarks`, `Stack`, `Menu`. ✓
  - `src/cli/stacks.rs:12-17` — `StackCommand` is `Push`/`Undo`/`Redo` under `dx stack`. ✓
  - `src/menu/buffer.rs:10-14` — `ParsedBuffer.replace_start`/`replace_end` confirmed as byte offsets; field doc says "Byte offset where the replacement region starts". ✓
  - `src/menu/action.rs:6-19` — `MenuAction` schema confirmed; `#[serde(tag = "action")]` with camelCase `replaceStart`/`replaceEnd`/`value` fields. ✓
  - `src/cli/menu.rs:245-281` — `MenuResult::Cancelled { changed_query: true }` emits `replace`; `changed_query: false` emits `noop`; `None` (no-TTY) emits `noop`. ✓
  - `src/hooks/zsh.rs:307-311` — Confirmed divergence: Zsh resets prompt and returns without native fallback on noop/error; line 309: `if [[ $__dx_exit -ne 0 ]] || [[ "$__dx_json" != *'"action":"replace"'* ]]; then zle reset-prompt; return; fi`. ✓
  - `src/hooks/bash.rs:304-330` — Bash falls back to mode-based native completion on non-replace. ✓
  - `src/hooks/mod.rs:25-37` — Canonical shell list: `"bash, zsh, fish, pwsh"`. ✓
  - `src/menu/tui.rs:82-116` — `CleanupGuard::drop` restores cursor/raw-mode on every exit path. ✓
  - `src/menu/tui.rs:139-220` — `select` returns `Some(Cancelled{changed_query:false})` immediately for empty candidates, and `Some(Selected{...})` for single-candidate auto-select. No-candidates path is distinct from no-TTY (`None`). ✓
- **[Note]** The Reality Check mismatch inventory is accurate. `docs/shell-hook-guarding.md` line 41 currently reads "Stack-transition wrappers ... use `dx undo`/`dx redo`" — confirmed stale vs. `dx stack undo`/`dx stack redo` in generated hooks. `docs/cd-extras-cli-prd.md` still contains `dx add`, generic `complete <type> <word>`, and `Invoke-Expression (& dx init pwsh)` — confirmed by direct inspection.
- **[Note]** The no-candidates distinction noted in Step 1 ("and no-candidates when distinct") is code-grounded: `tui.rs` returns `Some(Cancelled)` (not `None`) for empty candidates, so no-candidates is categorically different from no-TTY in the action output. The impl plan correctly calls for documenting them distinctly.
- **[Note]** The parsing asymmetry noted in the Reality Check (string/regex slicing in Bash/Zsh/Fish vs. structured `ConvertFrom-Json` in PowerShell) is confirmed by examining `src/hooks/bash.rs`, `src/hooks/zsh.rs`, and `src/hooks/pwsh.rs`. The plan correctly records this as acceptable-but-explicit in the C5 contract, defers unification decisions to Phase 2.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries with code-grounded decisions | yes | yes | Step text is detailed and code-grounded. Enumerates all four shells, all C4 scenario branches, and all C5 payload/offset/escaping/split-I/O elements explicitly. |
| 2 | Refresh shell-hook contract docs to current semantics | yes | yes | Concrete file targets and source anchors. Minor: verify command checks `dx stack undo` but not `dx stack redo`; the C3 pair is only half-gated. |
| 3 | Rewrite PRD to current command surface and architecture baseline | yes | yes | Concrete prohibition list is clear. Minor tension with phase allowance for labeled historical context (see Scope Alignment). |
| 4 | Cross-check docs against source and phase scope boundaries | yes | yes | Four-item completion checklist provides clear done signal without expanding scope. |

## Required Context Assessment

### Missing Context

- None critical. All files referenced in steps and verify command are listed.

### Unnecessary Context

- `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` is listed but not written during Phase 1. Its inclusion is appropriate as a cross-reference boundary marker.

## Testing Plan Assessment

### Test Integrity Check

The plan explicitly states: "No existing Rust/unit/integration tests are modified, removed, skipped, or weakened during Phase 1 (docs/contracts-only scope)." This is the correct constraint for a docs-only phase. Phase 2 and Phase 3 verification bars (automated tests + four-shell smokes) are preserved. The smoke matrix is not pre-filled by Phase 1 work.

### Test Gaps

- **[Minor]** The verify command checks for `rg -n -F "dx stack undo" "$guard"` but not `dx stack redo`. The documented C3 conflict is an undo/redo pair drift; a Phase 1 delivery that updates only the `undo` half would pass this gate while leaving the `redo` wording stale. The step text correctly says "matches current implementation (`dx stack undo|redo`)" but the verify command does not enforce the redo half.
  
- **[Minor]** The verify command checks for the string `"no-candidates"` in the inventory file using `rg -n -F "no-candidates" "$inv"`. This probe is technically correct because Step 1 explicitly requires the C4 contract to enumerate "no-candidates when distinct." However, "no-candidates" could appear as a descriptive label in any sentence — including in a table header or note — without actually defining the no-candidates branch behavior. This is a weak coupling between requirement and enforcement, not a blocking gap for a docs-only phase.

- **[Minor]** The `! rg -n -F "Invoke-Expression (& dx init pwsh)" "$prd" "$guard"` negation check has a shell-escaping caveat: when this line is embedded inside a `bash -lc '...'` heredoc with the literal parentheses `(& ...)`, `bash -F` with fixed-string mode correctly handles it because `-F` disables regex interpretation. The parentheses are safe for `rg -F`. However, within the outer `bash -lc '...'` single-quoted block, the inner literal is safe. **But**: `docs/shell-hook-guarding.md` already contains `Invoke-Expression ((& dx init pwsh | Out-String))` (the corrected form) and NOT `Invoke-Expression (& dx init pwsh)` — so after Phase 1 implementation, this absence check would need to ensure the old form without `Out-String` does not exist. Direct rg inspection of the current files confirms the *old* form does currently exist in `docs/cd-extras-cli-prd.md:75`. This check is valid and will correctly fail the pre-implementation state and pass after remediation. No action needed.

- **[Note]** The C4/C5 verify probes confirm token presence (e.g., `rg -n -F "menu-disabled" "$inv"`) but not section structure or adjacency. A well-intentioned implementer could write all C4/C5 tokens in a prose paragraph outside the labeled section headers and still pass the gate. This is an inherent limitation of token-presence verification for prose documents. For a docs/contracts-only phase this is an acceptable tradeoff; the lightweight sanity check (second test row in the table) compensates by requiring a human reviewer to cite specific lines for each item.

### Real-World Testing

No real-world shell execution is expected for Phase 1 (docs/contracts-only scope). The lightweight sanity check in the testing table provides a structured manual pass for a human reviewer to confirm each C4/C5 branch and the doc refresh markers are present and contextually correct. Full four-shell smoke verification is correctly deferred to Phase 3 via `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md`.

## Reference Consistency

### Findings

- All file paths referenced in Required Context, code anchors, and verify command exist in the current repository:
  - `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` — exists, 21 lines, all five C1-C5 rows `Open`. ✓
  - `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md` — exists, 32 rows all `Pending`. ✓
  - `docs/shell-hook-guarding.md` — exists, 123 lines. Currently contains stale `dx undo`/`dx redo` on line 41. ✓ (confirms conflict to be resolved)
  - `docs/cd-extras-cli-prd.md` — exists, 84 lines. Currently contains `dx add` (line 23), generic `complete <type> <word>` (line 69), and `Invoke-Expression (& dx init pwsh)` (line 75). ✓ (confirms conflicts to be resolved)
  - `docs/configuration.md` — exists, 83 lines. No apparent contradictions found during this review; conditional update is correct.
  - All `src/cli/*.rs` and `src/hooks/*.rs` and `src/menu/*.rs` files referenced in code anchors confirmed to exist with matching line ranges.
- The plan's `docs/shell-hook-guarding.md` verify assertion `rg -n -F "Out-String" "$guard"` is already satisfiable: `docs/shell-hook-guarding.md` line 73 already contains `Invoke-Expression ((& dx init pwsh --menu | Out-String))`. This means this particular check would pass even without the Phase 1 docs refresh — it is not strictly a new marker introduced by Phase 1 work. The implementer should be aware this single check provides weaker assurance for Step 2 than it appears.
- Cross-references from impl plan to `plan.md` and `phases/phase-1.md` are correctly stated and links are navigable.

## Reality Check Validation

### Findings

- The Reality Check section is honest and well-grounded:
  - All five mismatches/notes in "Mismatches / Notes" are accurate based on independent code and doc inspection.
  - The Zsh divergence note at Reality Check bullet 3 is confirmed by `src/hooks/zsh.rs:309-312`.
  - The parsing asymmetry note is confirmed by comparing bash/zsh string-slicing patterns with `pwsh.rs` `ConvertFrom-Json`.
  - The statement that `docs/shell-hook-guarding.md` currently says `dx undo`/`dx redo` is confirmed by direct read (line 41).
  - The statement that `docs/cd-extras-cli-prd.md` "is still proposal-era" is confirmed: it still contains the 4-phase migration plan and command table.
  - Final Reality Check bullet is an appropriate boundary: "this phase must not alter TUI/runtime code" — consistent with scope.
- One minor gap: the Reality Check notes that "conflict inventory may intentionally retain legacy command strings as historical conflict evidence" in the context of the stale-string removal checks. This is correct behavior but is not cross-referenced in Step 3's prohibition language. An implementer reading only Step 3 might apply the prohibition to inventory rows. The impl plan text does say "stale-string removal checks are therefore limited to refreshed docs, while inventory validation focuses on `Open` status and required decision outputs" — this is the right call; it just lives in the Reality Check section rather than in Step 3 itself.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Minor | Testing Plan | Verify command checks `dx stack undo` in `$guard` but not `dx stack redo`; the C3 pair is only half-enforced. | Add `rg -n -F "dx stack redo" "$guard" >/dev/null` parallel to the undo check. |
| 2 | Minor | Scope Alignment | Step 3 forbids obsolete spellings "anywhere in refreshed docs," which is narrower than phase-1.md AC4, which permits clearly labeled historical context. | Narrow prohibition to current-contract sections or add an explicit exemption for a dated historical-context block. |
| 3 | Minor | Reference Consistency | `docs/shell-hook-guarding.md` already contains `Out-String` (line 73, in the Enabling block), so the verify command assertion `rg -n -F "Out-String" "$guard"` would pass even without Step 2 remediation. The assertion is not a Phase 1 change-marker for that doc. | Add a stronger assertion for Step 2's specific C3 correction (e.g., `rg -n -F "dx stack undo" "$guard"` AND `rg -n -F "dx stack redo" "$guard"`) so verify measures the actual doc refresh, not pre-existing content. |
| 4 | Note | Testing Plan | C4/C5 verify probes check token presence document-wide, not within their labeled section headers. A prose paragraph containing all terms outside the named section would still pass. | Accept as inherent tradeoff for docs-only verification; the manual sanity check provides compensating coverage. |
| 5 | Note | Reality Check | Step 3's prohibition of obsolete strings applies to "refreshed docs" targets, but the boundary exempting the conflict inventory from this prohibition is documented only in the Reality Check section, not cross-referenced in Step 3. | Either add a parenthetical exception note in Step 3 or trust the Reality Check framing (acceptable for docs-only). |

## Recommendations

1. Add `rg -n -F "dx stack redo" "$guard" >/dev/null` to the verify command alongside the existing `dx stack undo` assertion. This closes the C3 half-enforcement gap and is a one-line addition.
2. Clarify Step 3's prohibition scope (current-contract sections only, with an explicit exception for a labeled historical-context block), or confirm the intent is to strip all legacy content from the PRD entirely and update phase-1.md AC4 expectations accordingly.
3. Consider replacing the single `Out-String` assertion for `$guard` with `dx stack undo` + `dx stack redo` assertions so the verify command measures what Step 2 actually changes rather than pre-existing content.
