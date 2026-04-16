---
type: planning
entity: phase
plan: "whole-repo-review-remediation"
phase: 2
status: completed
created: "2026-04-16"
updated: "2026-04-16"
---

# Phase 2: Fix Menu CWD and Flagged `cd` Parsing

> Part of [whole-repo-review-remediation](../plan.md)

## Objective

Bring the menu core back into contract by honoring explicit cwd overrides for `paths` mode and correctly isolating the path argument for approved flagged `cd` forms.

## Scope

### Includes

- Threading an explicit cwd through the `dx menu` → menu sourcing → completion pipeline for `paths` mode.
- Updating `src/menu/buffer.rs` parsing to support approved flagged `cd` forms (`-L`, `-P`, `--`) with correct replacement boundaries.
- Adding unit/integration coverage for explicit cwd behavior, flagged parsing, and fallback cases.

### Excludes (deferred to later phases)

- Deleting legacy hook prototypes.
- Broad shell-hook refactors unrelated to the parsing/cwd fixes.
- PowerShell wrapper architecture changes beyond what is required for correctness parity.

## Prerequisites

- [x] Phase 1 is completed and verified.
- [x] The supported flagged-`cd` grammar and fallback behavior are explicitly approved in the Phase 2 implementation plan.
- [x] The Phase 2 implementation plan explicitly decides support/fallback for `cd -L <path>`, `cd -P <path>`, `cd -- <path>`, and unknown/grouped flag forms before any parser changes land.

## Deliverables

- [x] Updated completion/menu APIs that honor explicit cwd for `paths` mode.
- [x] Updated menu buffer parsing and regression tests for approved flagged `cd` forms.
- [x] Phase-local smoke evidence recorded in `../verification/shell-smoke-matrix.md` for the minimum Phase 2 scenarios.
- [x] Verification evidence showing no regression to existing menu noop/fallback behavior.

## Acceptance Criteria

- [x] `dx menu --cwd <path>` resolves `paths` mode candidates from `<path>` rather than process cwd.
- [x] Approved flagged `cd` forms target only the actual path argument for query/replacement semantics.
- [x] Unsupported/ambiguous forms abort `dx menu` intervention and fall back predictably to native completion / execution behavior per the documented contract.
- [x] Targeted tests and `cargo test` pass after the phase completes.
- [x] The smoke matrix records, where feasible, Bash/Zsh/PowerShell evidence for `dx menu --cwd` path sourcing plus approved flagged `cd` handling, with Fish marked `Pass` or `Not Feasible` and rationale.

## Completion Notes / Evidence

- Final verification run passed with full-suite evidence: `cargo test` reported **286 passed** (as captured in final Phase 2 reviews).
- Cross-shell smoke evidence for Phase 2 is populated in `../verification/shell-smoke-matrix.md:9-14`, including Bash/Zsh/PowerShell `Pass`, Bash/Zsh flagged-form evidence, PowerShell PSReadLine fallback evidence, and Fish `Not Feasible` with rationale.
- Unified implementation-review decision and close-out summary are recorded in `../reviews/impl-review-phase-2-collated-2026-04-16.md`.

## Dependencies on Other Phases

| Phase | Relationship | Notes |
|-------|-------------|-------|
| 1 | blocked-by | Execution order keeps the highest-severity persistence fix first. |
| 3 | blocks | Hook-authority cleanup should validate the settled menu/command contract from this phase. |
| 4 | blocks | Final shell smoke and PowerShell decision work should reflect the corrected parser/completion behavior. |

## Notes

- The impacted contract is documented in `tech-docs/cd-extras-cli-prd.md` and reinforced by the review/impact artifacts.
- The PowerShell `ProxyCommand` question is explicitly not the mechanism for fixing the Rust-side menu parsing issue in this phase.
