## Context

`dx menu` currently seeds its interactive filter from the query parsed out of the shell buffer, but once the TUI is open it treats that filter as a freely editable string. In practice, Backspace can remove characters from the original prompt-derived query and broaden the candidate set beyond what the user initially asked for.

That behavior is mechanically simple, but it creates two UX issues. First, the filter semantics shift from "refine this query" to "rewrite this query," which is less predictable in a completion menu launched from an already-typed shell token. Second, broader result sets make dynamic menu resizing more relevant because the menu can expand again after initially narrowing.

## Goals / Non-Goals

**Goals:**
- Treat the prompt-derived query as the lower-bound seed for interactive filtering.
- Allow users to type additional refinement characters and backspace only those characters typed during the current menu session.
- Preserve existing candidate re-query behavior, selection behavior, and shell action protocol.
- Make menu result size monotonic relative to the initial query so the menu does not need to grow due to in-session filter broadening.

**Non-Goals:**
- Dynamically resizing the menu during filtering in this change.
- Redesigning replacement formatting, cancellation JSON structure, or shell hook protocols.
- Adding new configuration toggles for broadening vs clamped filter behavior.

## Decisions

### D1: Interactive filtering is modeled as `seed_query + typed_suffix`

The menu runtime will track the initial parsed query as an immutable seed and store only the incremental refinement typed during the session as mutable state.

The effective query sent to `query_fn` is always `seed_query` plus the current typed suffix.

Rationale: this makes the refinement boundary explicit in state rather than trying to reconstruct it from a single editable string.

Alternatives considered:
- Keep a single mutable filter string and clamp Backspace when it reaches the seed length. Rejected because it is easier to make mistakes in cancel/noop logic and less explicit in tests.

### D2: Backspace only removes the typed suffix

When the typed suffix is non-empty, Backspace removes one character and re-queries.

When the typed suffix is empty, Backspace is a no-op for filter state.

Rationale: this preserves a familiar edit loop while preventing the query from broadening past the starting point.

Alternatives considered:
- Exit the menu or beep at the seed boundary. Rejected because it is more disruptive than simply ignoring the boundary-crossing edit.

### D3: Net-zero filter edits remain `noop` on cancel

Cancel behavior will continue to be based on whether the final effective query differs from the initial query.

If the user types and then backspaces back to the seed query, cancellation returns `noop` rather than a redundant replace action.

Rationale: this matches the existing meaning of "no effective query change" and avoids unnecessary shell buffer churn.

### D4: Empty initial queries still behave naturally

If the parsed initial query is empty, the seed is the empty string and the typed suffix can grow and shrink back to empty during the session.

Rationale: this preserves the common "open menu on bare command, then type to narrow" flow without introducing special-case user-facing behavior.

## Risks / Trade-offs

- [Risk] Some users may expect Backspace to broaden the result set beyond the original token → Mitigation: document the new refinement-only behavior and keep it consistent across all menu-supported commands.
- [Risk] Ignored Backspace at the seed boundary may feel inert → Mitigation: keep behavior simple and predictable; the menu still remains interactive for navigation, typing, selection, or cancel.
- [Trade-off] This avoids menu-growth pressure by narrowing behavior rather than solving dynamic resizing directly → acceptable because it simplifies the interaction model and addresses the immediate UX concern.

## Migration Plan

1. Update menu filtering specs to define the seed-clamped edit model and revised cancel semantics.
2. Refactor TUI filter state to track immutable seed plus typed suffix.
3. Add tests for boundary Backspace behavior, net-zero cancel behavior, and empty-seed flows.
4. Update any docs/comments that describe filter editing as arbitrary query rewriting.
5. Rollback strategy: restore single-string filter editing and prior Backspace broadening behavior.

## Open Questions

- Should the UI eventually expose the seed boundary visually, or is invisible clamping sufficient?
- If users want broadening again later, should that come from an explicit keybinding rather than plain Backspace?
