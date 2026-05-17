## Why

In a fresh shell session, the first `cdf`/`z` jump can record only the destination in the session stack, so `cd-` reports `dx stack: nothing to undo` instead of returning to the origin. Jump wrappers should seed the current directory before changing directories, matching the existing `cd` and `up` behavior.

## What Changes

- Ensure generated jump wrappers record the current shell cwd before a successful `cdf`/`z`/`cdr` directory change.
- Preserve the existing post-change push so the destination remains the tracked current directory.
- Apply the behavior consistently across Bash, Zsh, Fish, and PowerShell hooks.
- Leave `back`/`forward` stack wrappers unchanged because they are stack operations, not new navigation origins.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `shell-hooks`: Jump wrappers seed the origin before changing directories so first-session undo works.

## Impact

- Affects generated shell hook code in `src/hooks/{bash,zsh,fish,pwsh}.rs`.
- Requires hook-generation tests for push-before-cd and push-after-success behavior in jump wrappers.
- No changes to stack storage semantics are expected because duplicate current-directory pushes are already no-ops.
