---
type: review
entity: implementation-plan-review
plan: "architectural-doc-code-alignment"
phase: 1
status: final
reviewer: "general"
created: "2026-04-08"
---

# Implementation Plan Review: Phase 1 - Refresh Architecture Docs and Contracts

> Reviewing [Phase 1 Implementation Plan](../implementation/phase-1-impl.md)
> Against [Phase 1 Scope](../phases/phase-1.md) and [Plan](../plan.md)

## Overall Assessment

**Verdict**: Needs Revision

The implementation plan is close: it stays within the docs/contracts-only boundary, cites current code accurately, and the primary verify command is now satisfiable because it only checks artifacts Phase 1 is allowed to change. However, the primary acceptance gate is still not strong enough for final Phase 1 sign-off because it can pass without proving the C4 handoff is explicitly per-shell and without proving each conflict row has meaningful adjudication text rather than marker-only compliance.

## Scope Alignment

### Findings

- The plan remains properly scoped to Phase 1: it refreshes docs and the conflict inventory, and it explicitly defers runtime hook/menu changes to Phase 2.
- **Minor**: Step 3's blanket prohibition on obsolete spellings anywhere in refreshed docs is slightly stricter than the phase acceptance criteria, which allow historical context as long as it is clearly distinguished from the current contract.

## Technical Feasibility

### Findings

- The proposed approach is technically sound and aligned with the codebase: `src/cli/mod.rs`, `src/cli/complete.rs`, `src/cli/stacks.rs`, `src/cli/menu.rs`, `src/menu/action.rs`, `src/menu/buffer.rs`, and `src/hooks/{bash,zsh,fish,pwsh}.rs` support a documentation-first adjudication pass without runtime edits.
- The C4/C5 handoff requirements are materially stronger now. Step 1 explicitly requires per-shell C4 branches plus C5 payload fields, offset semantics, escaping expectations, dependency-free parsing constraints, and split I/O expectations grounded in current code.

## Step Quality Assessment

| Step | Title | Concrete? | Actionable? | Issue |
| ---- | ----- | --------- | ----------- | ----- |
| 1 | Resolve conflict inventory entries with code-grounded decisions | yes | yes | Clear and grounded; C4/C5 handoff expectations are now explicit enough for execution. |
| 2 | Refresh shell-hook contract docs to current semantics | yes | yes | Concrete and appropriately docs-only. |
| 3 | Rewrite PRD to current command surface and architecture baseline | yes | yes | Actionable, but the ban on obsolete spellings may be stricter than necessary if clearly labeled historical context is retained. |
| 4 | Cross-check docs against source and phase scope boundaries | yes | no | The completion pass is useful, but the main verify gate still allows incomplete C4 adjudication structure to slip through. |

## Required Context Assessment

### Missing Context

- None.

### Unnecessary Context

- None.

## Testing Plan Assessment

### Test Integrity Check

- The plan explicitly protects test integrity for this docs/contracts-only phase: it says existing Rust/unit/integration tests must not be modified, removed, skipped, or weakened.
- There is exactly one primary verify command, and it is satisfiable within the phase scope.

### Test Gaps

- **Major**: the primary verify command is still too marker-oriented for a final Phase 1 acceptance gate. It checks for `Approved C4 Target Behavior`, scenario labels, and C5 keywords, but it does not assert that Bash, Zsh, Fish, and PowerShell are each explicitly represented in the C4 handoff, and it does not assert that every C1-C5 row has non-empty decision/rationale text after `Open` is removed.
- **Minor**: the primary verify command's doc checks are weighted toward the conflict inventory and PRD markers; the shell-hook doc still relies mostly on the manual checklist rather than strong command-level assertions for current contract items like `dx stack redo` and `dx navigate` wording.

### Real-World Testing

For a docs/contracts-only phase, the lightweight manual line-citation checklist is an appropriate real-world validation step. Deferring four-shell smoke execution to later phases is consistent with the phase and plan boundaries, provided Phase 1 claims only documented target behavior rather than implemented convergence.

## Reference Consistency

### Findings

- The referenced file paths and symbols are real and current.
- The plan correctly anchors the C5 contract to `src/menu/action.rs` for JSON fields, `src/menu/buffer.rs` for byte-based replacement offsets, `src/menu/tui.rs` for split I/O expectations, and `src/hooks/pwsh.rs` for PSReadLine handling.

## Reality Check Validation

### Findings

- The reality check is honest and materially improved. It correctly identifies stale docs, current Zsh fallback divergence, asymmetric parsing across shells, and the fact that this phase must codify the boundary contract without changing runtime behavior.
- **Note**: because the primary verify command does not enforce per-shell C4 structure, some of the reality check's useful nuance still depends on reviewer discipline rather than the automated gate.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Major | Testing Plan | The primary verify command is satisfiable but not yet strong enough for final Phase 1 acceptance because it does not verify explicit Bash/Zsh/Fish/PowerShell coverage in `Approved C4 Target Behavior` and does not ensure each conflict row contains real adjudication text. | Strengthen the command with structural checks for all four shell labels and for non-empty decision/rationale content on each C1-C5 row. |
| 2 | Minor | Testing Plan | The shell-hook doc remains under-asserted by the primary command, leaving some acceptance coverage to the manual checklist. | Add command-level checks for key `docs/shell-hook-guarding.md` markers such as `dx stack undo`, `dx stack redo`, and `dx navigate`. |
| 3 | Minor | Scope Alignment | Step 3's blanket ban on obsolete spellings is stricter than the phase acceptance criteria and may over-constrain clearly labeled historical context. | Allow clearly labeled historical references where useful, while keeping current-contract sections free of obsolete commands. |
| 4 | Note | Technical Feasibility | The C4/C5 handoff requirements themselves are now explicit and grounded enough for a docs/contracts-only Phase 1 execution pass. | Keep the strengthened Step 1 wording and focus revisions on the acceptance gate rather than reopening the handoff content. |

## Recommendations

1. Strengthen the primary verify command so it proves explicit four-shell C4 coverage and non-empty adjudication text for each C1-C5 inventory row.
2. Add a few `docs/shell-hook-guarding.md` assertions to the primary verify command so Phase 1 acceptance does not rely as heavily on the manual checklist.
3. Relax Step 3 only enough to permit clearly labeled historical context if the refreshed PRD needs it; keep current-contract sections strict.
