## Context

Generated shell hooks maintain dx session stacks by calling `dx stack push` around shell directory changes. The `cd` wrapper and `up` navigation wrapper already push the current directory before changing and push the destination after a successful change. Jump wrappers (`cdf`, `z`, and `cdr`) should follow the same pattern so a fresh session records the origin before the first jump destination.

The stack implementation already treats pushing the current tracked directory as a no-op, so pre-pushing the current cwd is safe and prevents duplicate consecutive entries.

## Goals / Non-Goals

**Goals:**

- Record the current shell cwd before jump wrappers change directories.
- Preserve the post-success destination push.
- Apply consistently to Bash, Zsh, Fish, and PowerShell generated hooks.
- Keep `back`/`forward` stack wrappers unchanged.

**Non-Goals:**

- Changing stack storage semantics or undo/redo behavior.
- Seeding session stacks at hook initialization time.
- Changing frecent/recent candidate sourcing or ranking.
- Changing shell command names or aliases.

## Decisions

1. Use pre-push plus post-push for jump wrappers.

Jump wrappers should call `__dx_push_pwd` after resolving a target and before invoking the native directory change. After a successful directory change, they should call `__dx_push_pwd` again to record the destination. This matches existing `cd` and `up` semantics and resynchronizes the stack with the actual shell cwd even if the user previously bypassed dx wrappers.

Alternative considered: push the cwd at hook initialization and only push after each successful directory change. That is simpler but less robust when users bypass dx wrappers with native shell commands.

2. Do not change stack operations.

`back`, `forward`, `cd-`, and `cd+` should continue to delegate to stack undo/redo without seeding a new origin. They are consuming stack history, not creating a new navigation branch.

3. Test generated hook ordering.

The safest regression coverage is generated-code tests that assert jump wrappers include `__dx_push_pwd` before the native cd/Set-Location call and a second push after success.

## Risks / Trade-offs

- Additional `dx stack push` call per jump -> Existing pushes are lightweight and duplicate current-directory pushes are no-ops.
- Hook tests can become brittle if they assert large script chunks -> Prefer focused section/order assertions around jump wrapper functions.
