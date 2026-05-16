## Context

`dx menu` currently computes compact display labels for candidates and reuses those labels for selected-item context in the status row. That means a selected path under the current working directory can appear as `./child` in the status row even though the selected candidate internally resolves to an absolute path.

The status row has a different job from candidate cells: candidate cells should stay compact for scanning, while the status row should confirm the exact destination that Enter will choose. Replacement formatting is a separate concern and already preserves shell-friendly query style where practical.

## Goals / Non-Goals

**Goals:**

- Display the full resolved selected path in the status row by default.
- Preserve compact candidate display labels in list and grid rendering.
- Preserve Enter behavior and replacement formatting exactly as it works today.
- Keep the existing status-row ordering, refinement display, overflow handling, and compression priority.

**Non-Goals:**

- Add configuration for status path display style.
- Change candidate sourcing, filtering, ranking, or selection identity.
- Change shell hook protocols or JSON action shape.
- Change path insertion style for accepted selections.

## Decisions

### Status Uses Resolved Candidate Path

The status row will derive selected-item context from the selected candidate path itself rather than from the precomputed compact display label. This gives the status row the full resolved path while leaving candidate cells unchanged.

### Replacement Formatting Remains Independent

The accepted selection path and the inserted shell replacement remain governed by the existing replacement formatter. For example, a status row may show `/Users/nick/code/personal/dx/src`, while Enter can still insert `./src/` when the user's query style calls for a relative paths-mode replacement.

### No Configuration Yet

The status row will use the full resolved path by default without a user-facing setting. A configuration option can be considered later if there is a concrete need for compact status paths, but adding one now would complicate a small display-contract clarification.

## Risks / Trade-offs

- Full paths can be long and more frequently require truncation. Mitigation: the existing status-row compression rules already preserve selected-item context and tail-truncate long text.
- Full paths may duplicate information visible in the prompt for cwd-local candidates. Mitigation: the status row is intended as exact confirmation, while candidate cells remain compact for scanning.
- Display and insertion can intentionally differ. Mitigation: document and test that status display is confirmation-only and does not affect Enter replacement behavior.
