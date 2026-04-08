---
type: planning
entity: todo
plan: "architectural-doc-code-alignment"
updated: "2026-04-08"
---

# Todo: architectural-doc-code-alignment

> Tracking [architectural-doc-code-alignment](plan.md)

## Active Phase: 2 - Harden Cross-Shell Menu and Hook Boundaries

### Phase Context

- **Scope**: [Phase 2](phases/phase-2.md)
- **Implementation**: [Phase 2 Plan](implementation/phase-2-impl.md)
- **Latest Handover**: _Not created yet_ (`handovers/session-2026-04-08.md` when available)
- **Relevant Docs**:
  - `plans/architectural-doc-code-alignment/contracts/phase-1-conflict-inventory.md`
  - `plans/architectural-doc-code-alignment/verification/shell-smoke-matrix.md`
  - `docs/shell-hook-guarding.md`
  - `docs/cd-extras-cli-prd.md`
  - `docs/configuration.md`

### Pending

- [ ] Implement Zsh noop/error fallback parity while preserving successful replace behavior. <!-- added: 2026-04-08 -->
- [ ] Harden Bash, Zsh, and Fish replace payload parsing without introducing new required dependencies. <!-- added: 2026-04-08 -->
- [ ] Preserve the Rust-side menu action contract and extend automated tests where needed for C4/C5 coverage. <!-- added: 2026-04-08 -->
- [ ] Update supporting docs as needed for final implemented behavior and run the Phase 2 verify command. <!-- added: 2026-04-08 -->

### In Progress


- [ ] Review the Phase 2 implementation plan and resolve findings before execution. <!-- started: 2026-04-08 -->

### Completed

- [x] Create the initial plan, phase documents, and tracking scaffold. <!-- completed: 2026-04-08 -->
- [x] Run plan review and incorporate accepted findings into the baseline plan. <!-- completed: 2026-04-08 -->
- [x] Complete Phase 1 docs/contracts refresh and pass the Phase 1 verify command. <!-- completed: 2026-04-08 -->
- [x] Author the Phase 2 implementation plan. <!-- completed: 2026-04-08 -->

### Blocked

## Changelog

### 2026-04-08

- Created the initial three-phase plan and queued the Phase 1 documentation-alignment work.
- Incorporated parallel plan-review findings by adding an explicit conflict inventory, boundary-contract guardrails, and a shell smoke matrix scaffold.
- Completed Phase 1 documentation and contract alignment work; refreshed the PRD and shell-hook docs, resolved the conflict inventory, and verified the phase successfully.
- Authored the Phase 2 implementation plan and shifted active work to Phase 2 review/execution preparation.
