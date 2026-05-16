## Context

`dx menu` renders an interactive candidate list with a one-line status area. The current status row is assembled as a single string that places `filter: <effective-query>` before the selected path, even when the user has not typed any additional refinement after opening the menu.

Filtering already separates the initial query parsed from the shell buffer from the typed refinement entered during the menu session. Querying must continue to use the full effective query, but status rendering can use only the typed refinement as display state.

## Goals / Non-Goals

**Goals:**

- Make the selected candidate the primary status-row element.
- Show in-menu refinement only after the user types refinement characters.
- Display only the typed refinement, not the initial prompt query.
- Right-align active refinement with a compact `/` marker instead of `filter:`.
- Define narrow-terminal behavior that keeps selected-item context more important than refinement visibility.

**Non-Goals:**

- Change candidate sourcing, query matching, or incremental filtering semantics.
- Change the JSON action protocol returned to shell hooks.
- Add configuration for status-row ordering, labels, or truncation thresholds.
- Redesign menu item rendering, grid layout, borders, or scrollbars.

## Decisions

### Status Row Ordering

The status row will treat selection as the left-aligned primary element and typed refinement as a right-aligned secondary element. Overflow metadata remains optional and sits between selection and refinement when there is room.

This preserves the most important action context: the selected item is what Enter will accept. The right edge gives typed refinement a stable visual anchor without letting it visually merge with path text.

### Typed Refinement Display

The refinement indicator will display only characters typed during the current menu session. It will not repeat the initial query parsed from the shell prompt.

The initial query is already visible in the prompt that opened the menu. Repeating it in the status row wastes width and makes it harder to distinguish the user's additional narrowing.

### Compact Refinement Marker

The refinement indicator will use `/` as a compact search-style prefix, for example `/src`. The literal label `filter:` will be removed from status rendering.

`/` is short, familiar from search/filter interfaces, and avoids consuming status-row width with a word label.

### Compression Priority

The status row will degrade in this order as width tightens:

1. Omit overflow metadata.
2. Truncate selected-item context and refinement independently.
3. Shrink refinement below its normal cap if needed to preserve selected-item context.
4. Hide refinement entirely when the terminal is too narrow to keep both useful.
5. Truncate selected-item context to the available row as the final fallback.

The core invariant is: never hide the selected item just to show refinement text.

## Risks / Trade-offs

- Users may not immediately understand `/abc` as a filter indicator. Mitigation: the marker appears only after they type inside the menu, so the cause/effect relationship should be discoverable.
- Hiding refinement in extremely narrow terminals can remove visible feedback for typed input. Mitigation: selected-item context remains the safer priority because it reflects what Enter will do.
- Tests for exact rendered strings can become brittle if width allocation is over-specified. Mitigation: keep helper-level tests focused on element presence, ordering, and compression priority rather than terminal drawing internals.
