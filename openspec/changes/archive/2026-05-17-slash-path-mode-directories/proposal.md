## Why

Mapped `path` mode can select both files and directories, but directory selections currently replace the active token without a trailing slash. That makes repeated Tab/menu expansion into a selected directory less smooth than `directory` mode or built-in `paths` mode.

## What Changes

- Append a trailing `/` when a `dx menu --mode path` selection resolves to a directory.
- Continue leaving selected files in `path` mode unsuffixed.
- Preserve existing trailing-slash behavior for built-in `paths` mode and mapped `directory` mode.
- When the path-mode directory replacement requires quoting, include the trailing slash inside the quoted path token.
- Preserve no-slash behavior for mapped `file` mode and non-filesystem completion modes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dx-menu`: Explicit `path` mode replacement formatting changes for selected directories.

## Impact

- Affects replacement value formatting in `src/cli/menu.rs`.
- Requires tests for path-mode directory and file selections, including relative query-style replacements and quoted directory replacements.
- No shell hook changes are expected because hooks already apply the `dx menu` JSON replacement value.
