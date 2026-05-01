## ADDED Requirements

### Requirement: Dynamic Height Reduction in Multicolumn Mode
When multicolumn mode is active, menu height SHALL be recomputed from the current filtered grid row count after each filter update.

The rendered multicolumn menu SHALL shrink as filtered results require fewer rows, while respecting configured maximum rows.

This behavior SHALL apply in both bordered and borderless modes.

#### Scenario: Multicolumn height shrinks with fewer grid rows
- **WHEN** a multicolumn menu initially requires several grid rows
- **AND** filtering reduces results to a single grid row
- **THEN** the menu height SHALL shrink to match the reduced row count within configured limits

#### Scenario: Bordered multicolumn shrink keeps border integrity
- **WHEN** bordered multicolumn mode shrinks from a taller to a shorter height
- **THEN** the resulting border SHALL remain visually complete with no stale border fragments below the new bottom edge

#### Scenario: Borderless multicolumn shrink clears trailing separator and scrollbar artifacts
- **WHEN** borderless multicolumn mode shrinks and no longer needs previously rendered trailing rows
- **THEN** vacated rows and any prior scrollbar/separator artifacts SHALL be cleared
