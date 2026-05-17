## 1. Hook Behavior

- [x] 1.1 Update Bash, Zsh, Fish, and PowerShell jump wrappers to push the current cwd before changing to resolved `cdf`/`z`/`cdr` targets.
- [x] 1.2 Preserve post-success destination pushes and keep failed-resolution or failed-cd paths from recording a destination.
- [x] 1.3 Leave `back`, `forward`, `cd-`, and `cd+` traversal wrappers unchanged.

## 2. Verification

- [x] 2.1 Add focused generated-hook tests for jump wrapper pre-push and post-success push ordering.
- [x] 2.2 Run the relevant Rust test suite and fix regressions.
