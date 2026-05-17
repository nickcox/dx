## Why

Directory replacements that require quoting currently place the appended drill-in slash outside the quoted path, for example `'/Users/nick/Dropbox (Maestral)'/`. That form works in POSIX-style shells but fails in PowerShell path-command contexts, while including the slash inside the quoted path works across the supported shells for the intended replacement token.

## What Changes

- Change quoted directory replacements so the trailing `/` is included inside the quoted path token.
- Preserve parsing compatibility for existing buffers that use the old outside-slash form.
- Update the active `slash-path-mode-directories` change to inherit the same quote placement rule for mapped `path` mode directories.
- Do not change the JSON action shape or shell hook replacement behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dx-menu`: Quoted replacement formatting changes for selected directories with trailing slashes.

## Impact

- Affects replacement value formatting and quoted-token parsing in `src/cli/menu.rs` and `src/menu/buffer.rs`.
- Requires updates to formatting/parser tests and the active path-mode-directory delta spec.
- No shell hook changes are expected.
