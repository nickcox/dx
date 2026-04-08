---
type: planning
entity: todo
plan: "architectural-doc-code-alignment"
updated: "2026-04-08"
---

# Todo: architectural-doc-code-alignment

> Tracking [architectural-doc-code-alignment](plan.md)

## Active Phase: None (Plan Completed)

### Phase Context

- **Scope**: [Phase 3](phases/phase-3.md)
- **Implementation**: [Phase 3 Plan](implementation/phase-3-impl.md)
- **Latest Handover**: _Not created yet_ (`handovers/session-2026-04-08.md` when available)
- **Relevant Docs**:
  - `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md`
  - `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md`
  - `docs/shell-hook-guarding.md`
  - `docs/cd-extras-cli-prd.md`
  - `docs/configuration.md`

### Pending

- _None._

### In Progress

- _None._

### Completed

- [x] Create the initial plan, phase documents, and tracking scaffold. <!-- completed: 2026-04-08 -->
- [x] Run plan review and incorporate accepted findings into the baseline plan. <!-- completed: 2026-04-08 -->
- [x] Complete Phase 1 docs/contracts refresh and pass the Phase 1 verify command. <!-- completed: 2026-04-08 -->
- [x] Author the Phase 2 implementation plan. <!-- completed: 2026-04-08 -->
- [x] Reconcile stale Phase 2 artifacts to completed status before Phase 3 closeout. <!-- completed: 2026-04-08 -->
- [x] Record Phase 3 automated verification outcomes (`menu_cli`, full `cargo test`, targeted `init_cli`/`key_event_mapping_`/exact `menu_cli` checks). <!-- completed: 2026-04-08 -->
- [x] Finalize shell smoke matrix with explicit Pass/Not Feasible evidence for Bash, Zsh, Fish, and PowerShell scenarios. <!-- completed: 2026-04-08 -->
- [x] Close Phase 3 and finalize plan artifacts to completed state. <!-- completed: 2026-04-08 -->

### Blocked

- _None._

## Changelog

### 2026-04-08

- Created the initial three-phase plan and queued the Phase 1 documentation-alignment work.
- Incorporated parallel plan-review findings by adding an explicit conflict inventory, boundary-contract guardrails, and a shell smoke matrix scaffold.
- Completed Phase 1 documentation and contract alignment work; refreshed the PRD and shell-hook docs, resolved the conflict inventory, and verified the phase successfully.
- Authored the Phase 2 implementation plan and shifted active work to Phase 2 review/execution preparation.
- Reconciled stale Phase 2 planning artifacts to completed state (phase + implementation + todo preflight hygiene).
- Completed Phase 3 verification/finalization: recorded automated test outcomes and finalized shell smoke matrix with explicit Pass/Not Feasible rows (including Fish-missing, ZLE-active requirement, and non-interactive PSReadLine constraints).
