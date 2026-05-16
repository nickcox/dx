## 1. Status Model

- [x] 1.1 Expose typed refinement separately from effective query for status rendering while preserving effective query behavior for candidate sourcing and exit results.
- [x] 1.2 Replace the current single formatted status string with structured status elements for selected item, overflow metadata, and optional refinement.

## 2. Status Rendering

- [x] 2.1 Render selected-item context as the left-aligned primary status element.
- [x] 2.2 Render typed refinement only when non-empty, right-aligned, and prefixed with `/` instead of `filter:`.
- [x] 2.3 Render overflow metadata as optional secondary text between selection and refinement when width allows.

## 3. Compression Behavior

- [x] 3.1 Add width allocation and truncation logic that drops overflow metadata before truncating selection or refinement.
- [x] 3.2 Cap long refinement text so selected-item context cannot be reduced to zero width.
- [x] 3.3 Hide refinement before selected-item context when terminal width is too narrow to show both usefully.

## 4. Verification

- [x] 4.1 Add unit coverage for status-row element ordering, typed-refinement-only display, absence of refinement before in-menu typing, and removal of the literal `filter:` label.
- [x] 4.2 Add unit coverage for compression priority: overflow omission, long selection truncation, long refinement capping, and tiny-terminal selection priority.
- [x] 4.3 Run the relevant Rust test suite and ensure existing menu behavior remains unchanged outside status rendering.
