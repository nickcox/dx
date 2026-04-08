---
type: review
entity: plan-review
plan: "architectural-doc-code-alignment"
status: final
reviewer: "reviewer-1"
created: "2026-04-08"
---

# Plan Review: architectural-doc-code-alignment

> Reviewing [architectural-doc-code-alignment](../plan.md)

## Overall Assessment

**Verdict**: Needs Revision

The plan correctly identifies real drift between `docs/cd-extras-cli-prd.md` and the current Rust implementation, and the three-phase structure (audit → harden → verify) is logical and sound. However, several gaps reduce confidence: Phase 1 is scoped as documentation-only but lacks a concrete decision-record deliverable format, Phase 2 has a high-impact risk (payload format change) that is deferred without a fallback policy, and the testing strategy for Phase 3 is manual-only for shell smoke without any harness guidance. These are correctable with targeted additions rather than a full rewrite.

## Requirement Coverage

| Requirement | Covered By | Gap? | Notes |
| ----------- | ---------- | ---- | ----- |
| Adjudicate architecture conflicts case by case | Phase 1 (deliverable: documented contract baseline) | Partial | No explicit format specified for the decision record; "a documented contract baseline" is vague |
| Refresh `docs/cd-extras-cli-prd.md` | Phase 1 (refreshed PRD content deliverable) | No | Clearly scoped |
| Refresh shell-hook documentation | Phase 1 (updated docs deliverable) | No | Clearly scoped |
| Align menu noop/error handling across shells incl. Zsh | Phase 2 (code changes + tests) | Partial | Zsh menu fallback behavior differs from Fish (Fish falls through to native; Zsh does NOT — see code analysis below); the plan does not call this out explicitly |
| Harden shell-to-`dx menu` replacement boundary | Phase 2 | Partial | Risk of payload format change is noted but decision deferred with no fallback; Phase 2 acceptance criteria do not verify quoting edge cases beyond "supported escaping/quoting contract" |
| Preserve thin-wrapper architecture | All phases | No | Explicitly called out as a constraint |
| Add/update tests (Bash, Zsh, Fish, PowerShell) | Phase 2 (automated) + Phase 3 (smoke) | Partial | No guidance on what constitutes passing smoke verification; no regression baseline captured |
| No new required dependencies | Non-functional, plan-level constraint | No | Stated clearly |
| Minimal behavioral delta outside targeted corrections | Non-functional | No | Stated clearly |
| PowerShell `Out-String` init guidance | Phase 3 (smoke) | Partial | Testing strategy lists this but Phase 3 scope text references it only as "init usage and fallback behavior" — link between the two is implicit |

## Scope Clarity

### Findings

- **MINOR**: "Refreshing `docs/configuration.md` where relevant" is vague. The current `configuration.md` is already well-structured and covers all environment variables accurately. The plan does not identify a specific gap, making this line potentially a no-op that adds ambiguity about whether Phase 1 is "done" when configuration.md is not touched.

- **MINOR**: The out-of-scope list is appropriately defensive, but the plan does not explicitly exclude OpenSpec workflow changes from Phase 1 deliverables, only from the overall goal. Since Phase 1 touches architectural docs, any overlap with open changes (e.g., delta specs) should be called out explicitly.

- **NOTE**: "Records case-by-case decisions" in Phase 1 is a good intent, but the plan does not specify where those decisions land — are they updated in `AGENTS.md` `Decision` section, in a dedicated `docs/arch-decisions.md`, or inline in the refreshed docs? Without this, Phase 2 cannot reliably reference them as "approved contract."

- **MAJOR**: The plan's requirement to "align menu noop/error handling across supported shells, including Zsh" does not acknowledge that the current Zsh and Fish hooks have *different* cancel-path fallback policies. Zsh's `__dx_menu_widget` explicitly does **not** fall through to native completion on noop/error (it calls `zle reset-prompt` instead). Fish's `__dx_menu_complete` **does** fall through (`commandline -f complete`). Bash's `__dx_try_menu` returns 1 on noop, causing `_dx_menu_wrapper` to fall through to native completion. This behavioral asymmetry is an undocumented drift that Phase 1 should explicitly surface and adjudicate before Phase 2 can "align" it. The plan needs to call this out.

## Definition of Done Assessment

### Findings

- **MINOR**: The DoD item "The targeted architecture conflicts are resolved and documented case by case" is not objectively checkable because "targeted" is undefined. There is no list of specific conflict areas. An implementer cannot verify this criterion without that list.

- **MINOR**: The DoD item "Plan and phase artifacts are updated to reflect what was implemented and verified" is standard housekeeping. It is correct but adds no discriminating signal — it will be marked done by any plan completion flow. It is not a quality criterion for the specific work.

- **MAJOR**: The DoD has no criterion covering whether the **Zsh menu cancel fallback asymmetry** (identified above) was consciously decided and implemented. If Phase 2 inadvertently preserves the current asymmetry, the DoD as written would still pass. A specific criterion like "Zsh, Bash, Fish, and PowerShell menu cancel/noop behavior is documented as intentional and consistent or intentionally differentiated" would close this gap.

- **NOTE**: The DoD does not mention that `docs/cd-extras-cli-prd.md` historical context should be clearly distinguished from current requirements (this is in the requirements but not in the DoD). Verify step is easier when both are in the DoD.

## Phase Structure Assessment

| Phase | Title | Verdict | Issue |
| ----- | ----- | ------- | ----- |
| 1 | Refresh Architecture Docs and Contracts | OK with minor gap | No format specified for decision record; acceptance criterion for "no known contradictions" is not verifiable without an initial contradiction inventory |
| 2 | Harden Cross-Shell Menu and Hook Boundaries | Needs clarification | High-impact payload format risk left unresolved in the plan; acceptance criteria are underspecified for quoting edge cases; phase can begin without resolving Zsh fallback asymmetry if Phase 1 does not surface it |
| 3 | Verify Across Shells and Finalize Documentation | OK | Well-structured; notes are specific and actionable; prerequisite gate is clear |

### Phase-Specific Findings

- **MAJOR**: Phase 1 acceptance criterion "No known contradictions remain in the targeted docs" is not verifiable without a pre-phase inventory of known contradictions. If Phase 1 starts without listing what conflicts exist, there is no way to confirm they are all resolved at the gate. The phase should include a step to produce that inventory first.

- **MINOR**: Phase 2 prerequisite "A reviewed implementation plan exists for this phase" is appropriate but means Phase 2 cannot be estimated until Phase 1 completes. This is acceptable given the adaptive nature of the work, but it should be acknowledged explicitly as a known scheduling uncertainty, not just an implicit dependency.

- **NOTE**: Phase 3 notes are good — they explicitly call out `init usage`, `menu-disabled behavior`, `noop/error fallback`, and `at least one successful replacement flow` per shell. This is more specific than the acceptance criteria, which only say "smoke checks completed." Consider promoting these from notes to acceptance criteria.

## Testing Strategy Assessment

### Test Coverage Gaps

- The plan does not specify what automated test coverage is expected to be **added** in Phase 2 vs what already exists. The existing `src/hooks/mod.rs` tests cover structural generation (balanced braces, function presence) but not behavioral equivalence of menu action parsing. The plan should require at least: (a) a unit test for each shell's menu JSON parsing logic (value extraction, replaceStart, replaceEnd), and (b) an integration test exercising the noop/cancel fallback path for each shell.

- Fish's menu JSON parsing uses `string replace -r` regex to extract `value`, `replaceStart`, `replaceEnd`. If the key ordering in the JSON output ever changes, these extractions silently return wrong values. The plan should require a test that validates Fish's parsing against the actual `MenuAction::to_json()` output format.

- The plan lists "smoke verification for Bash, Zsh, Fish, and PowerShell" but does not define a minimum smoke script, success criteria, or how results are recorded. Without this, Phase 3 smoke verification is unevaluable — a reviewer cannot confirm it was adequate.

- PowerShell menu behavior is identified as a risk (PSReadLine / terminal capability variance), but the plan does not require a smoke variant covering the degraded (no-PSReadLine) path.

### Real-World Testing

Real-world testing **is planned** via smoke verification in Phase 3. However, the plan does not define a minimum reproducible smoke procedure (e.g., a script or checklist) for any of the four shells. This is a gap: without a defined procedure, smoke verification is informally "what the implementer chose to test" and cannot be independently reproduced or audited. The plan should require that a smoke checklist or procedure be recorded as a Phase 3 deliverable, not just the results.

## Reference Consistency

### Findings

- All source file references in the phase notes (`src/hooks/bash.rs`, `src/hooks/zsh.rs`, `src/hooks/fish.rs`, `src/hooks/pwsh.rs`, `src/hooks/mod.rs`, `src/menu/action.rs`, `src/cli/menu.rs`, `src/cli/complete.rs`, `src/cli/stacks.rs`) were verified as existing and correct.

- All doc file references (`docs/shell-hook-guarding.md`, `docs/configuration.md`, `docs/cd-extras-cli-prd.md`) were verified as existing.

- The phase table in `plan.md` correctly references `phases/phase-1.md`, `phases/phase-2.md`, `phases/phase-3.md` — all files exist.

- **NOTE**: `docs/cd-extras-cli-prd.md` still contains the proposed legacy CLI surface (`add`, `undo`, `redo`, `frecent`, `recent`) as the "current" API in Section 4.1. The actual current CLI surface (`dx stack`, `dx navigate`, `dx complete`, `dx menu`) is not listed there. Section 4.2 also still shows the PowerShell init as `Invoke-Expression (& dx init pwsh)` without the `Out-String` guidance required by the AGENTS.md gotcha. These are confirmed drifts that Phase 1 should fix — the plan is correctly targeting them, but the scope should call them out explicitly rather than leaving discovery to the implementation.

## Completeness Check

### Findings

- **MAJOR**: The plan identifies the payload format change risk as "High" but says only "Decide this explicitly during the implementation-plan step." There is no gate, fallback policy, or minimum constraint on what the new format must satisfy. If the implementation-plan step reaches a decision to change the payload format, there is no plan-level guidance on backward compatibility (e.g., must all four hooks be updated atomically, can the binary emit both formats temporarily). This is a significant executable gap.

- **MINOR**: The plan does not mention `src/hooks/mod.rs` or the existing structural tests at all. Phase 2 work will likely need to update or extend these tests. Calling them out as starting-point context would improve Phase 2 implementability.

- **NOTE**: The plan changelog only has the creation entry. This is expected at plan creation and is not a gap.

- **NOTE**: `todo.md` correctly reflects the plan state and marks the review as in-progress. No inconsistency found.

## Findings Summary

| # | Severity | Area | Finding | Recommendation |
| - | -------- | ---- | ------- | -------------- |
| 1 | Major | Scope / Code Reality | Zsh menu cancel fallback does NOT fall through to native completion (uses `zle reset-prompt`), while Fish and Bash DO fall through. This undocumented asymmetry is the main alignment target but is not explicitly called out in the plan. | Add a specific requirement or note that identifies this asymmetry and requires Phase 1 to adjudicate and Phase 2 to implement the agreed behavior. |
| 2 | Major | Phase Structure | Phase 1 acceptance criterion "no known contradictions remain" is unverifiable without a pre-phase conflict inventory. Phase 1 may complete without actually capturing all conflicts. | Require Phase 1 to begin by producing an explicit conflict inventory, and make resolving all listed items the acceptance criterion. |
| 3 | Major | Definition of Done | The high-impact payload format change risk is deferred to the implementation-plan step with no plan-level constraints (atomicity, backward compatibility, dependency prohibition). | Add a DoD or risk mitigation note specifying minimum constraints on any payload format change. |
| 4 | Major | Testing Strategy | No minimum smoke procedure is defined for Phase 3 shell verification. Results cannot be reproduced or audited. | Require Phase 3 deliverables to include a recorded smoke checklist per shell, not just pass/fail outcomes. |
| 5 | Minor | Scope Clarity | "Refreshing `docs/configuration.md` where relevant" is vague; the file appears up-to-date and no specific gap is identified. | Either identify the specific gap in configuration.md or remove it from scope to avoid ambiguity about Phase 1 completion. |
| 6 | Minor | Scope Clarity | Phase 1 decision records have no specified location or format. Phase 2 cannot reliably reference them as "approved contract." | Specify where decisions land (e.g., inline in refreshed docs, `AGENTS.md` Decision section, or a dedicated doc). |
| 7 | Minor | Testing Strategy | Fish's menu JSON parsing uses regex extractions that are order-sensitive; no test validates Fish parsing against the actual `MenuAction::to_json()` format. | Require a Phase 2 test explicitly covering each shell's menu JSON parsing against the canonical serialization. |
| 8 | Minor | Definition of Done | DoD item "targeted architecture conflicts resolved" is not objectively checkable without a conflict list. | Tie this DoD item to the conflict inventory produced in Phase 1. |
| 9 | Note | Reference Consistency | `docs/cd-extras-cli-prd.md` Section 4.1 lists legacy CLI surface and Section 4.2 shows outdated PowerShell init. These are confirmed Phase 1 targets — the plan should name them explicitly rather than leaving them for discovery. | Name these two specific sections as Phase 1 refresh targets in the phase doc or plan scope. |
| 10 | Note | Phase Structure | Phase 3 notes are more specific than Phase 3 acceptance criteria. | Promote the notes content (init usage, menu-disabled, noop/error, replacement flow per shell) into formal acceptance criteria. |

## Recommendations

1. **[Major] Surface and adjudicate the Zsh/Fish/Bash menu cancel fallback asymmetry explicitly.** Add it to the Phase 1 conflict inventory scope and ensure Phase 2 acceptance criteria verify the chosen policy for all four shells.

2. **[Major] Require Phase 1 to start by producing a conflict inventory.** Change the Phase 1 acceptance criteria so that resolving all items in that inventory (rather than "no known contradictions") is the gate for Phase 2.

3. **[Major] Add plan-level constraints on any payload format change.** Define minimum requirements (e.g., all shell hooks updated atomically, no new required external tool, format must remain a flat JSON object) so the implementation-plan step is bounded.

4. **[Major] Define a minimum smoke procedure as a Phase 3 deliverable.** At minimum: a per-shell checklist covering init, menu-disabled, noop/cancel, and successful replacement. Record results as the Phase 3 completion artifact.

5. **[Minor] Clarify or remove `docs/configuration.md` from Phase 1 scope.** Either name the specific gap or remove it.

6. **[Minor] Specify where Phase 1 decision records are recorded** so Phase 2 has an unambiguous reference.

7. **[Minor] Add a Phase 2 test requirement** for each shell's menu JSON parsing against the canonical `MenuAction::to_json()` serialization format.
