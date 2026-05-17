## Why

Mapped external commands currently treat a rooted `/` query as both a root filesystem query and a cwd-child query, causing menus like `Get-ChildItem /<tab>` to show root entries mixed with current-directory entries. Rooted path input in mapped commands should behave like filesystem path input: `/` means the filesystem root, not the current directory.

## What Changes

- Fix mapped `path`, `directory`, and `file` candidate sourcing so a rooted `/` query uses `/` as the parent directory.
- Ensure `/<prefix>` queries for mapped commands list matching children under `/` and do not inject cwd children.
- Preserve existing cwd behavior for empty queries and bare relative queries.
- Preserve existing `./`, `../`, and `~/` mapped-command behavior.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `menu-command-mappings`: Clarify rooted path candidate sourcing for mapped external commands.

## Impact

- Affected code: `src/menu/mod.rs` mapped filesystem candidate sourcing and tests.
- Affected behavior: mapped external commands such as PowerShell `Get-ChildItem=path` no longer show cwd children for `/` or rooted absolute prefixes.
- No shell hook protocol changes, config changes, or JSON action shape changes.
