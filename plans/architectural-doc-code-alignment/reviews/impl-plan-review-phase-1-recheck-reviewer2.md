# Implementation Plan Review: phase-1-impl.md (Reviewer 2)

## Verdict: revise

## Summary

The revised plan correctly scopes its work to documentation and contract alignment, accurately reflects the architectural goals, and uses explicit code anchors to ground its work. However, the exact bash `verify` command contains a logical flaw that will force the implementer into an anti-pattern or cause spurious failures upon completion.

## Findings

### 1. Flawed Verify Command Asserts Against Historical Descriptions (Severity: High)
The `verify` bash command negatively asserts the absence of strings like `"dx add"`, `"dx undo"`, `"dx redo"`, `"Invoke-Expression (& dx init pwsh)"`, and `"dx complete <type> <word>"` in `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md`. However, these terms are the exact literal strings used in the initial problem descriptions for conflicts C1, C2, and C3 within the inventory. The implementer is instructed to resolve the items by changing the `Open` status and appending decisions—not by rewriting the historical problem descriptions. If they preserve the historical conflict definitions, the verify command will spuriously fail.
**Recommendation:** Remove `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md` from the negative `! rg -F` search paths in the verify command. The negative assertions should only apply to the target documentation files (`docs/cd-extras-cli-prd.md` and `docs/shell-hook-guarding.md`).

### 2. Missing Explicit Path for Validation Scripts (Severity: Low)
While the `verify` script is provided as a `bash -lc` one-liner, it is somewhat fragile and hard to read.
**Recommendation:** Consider wrapping the complex bash validation into a temporary or dedicated script (e.g., `scripts/verify-phase1.sh`) if the one-liner becomes too unwieldy after corrections. This is optional but improves execution reliability.

## Reality Check
- [x] The plan correctly stays within its docs/contracts-only scope boundaries for Phase 1.
- [x] References to code (`src/hooks/`, `src/cli/`, `src/menu/`) are valid and accurate as anchor points.
- [x] Requirements around PowerShell init single-script block evaluation (`Out-String`) correctly reflect project lore constraints.