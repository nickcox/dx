# Shell Smoke Matrix

> Verification matrix for [architectural-doc-code-alignment](../plan.md).
>
> Phase 3 should mark each row `Pass`, `Fail`, `Not Run`, or `Not Feasible`, and add concise evidence notes (command snippet, observed behavior, or reason).

| Shell | Scenario | Expected Outcome | Status | Evidence / Notes |
|-------|----------|------------------|--------|------------------|
| Bash | Init usage | Generated init script loads successfully and exposes the expected wrappers/completions. | Pending | |
| Bash | Menu disabled | With `DX_MENU=0`, Tab behavior falls back to native completion behavior. | Pending | |
| Bash | Successful replace | Menu selection applies the expected replacement behavior. | Pending | |
| Bash | Cancel with typed query | Cancel preserves typed refinement according to the approved contract. | Pending | |
| Bash | Noop/error fallback | Noop or command failure follows the approved fallback contract. | Pending | |
| Bash | No TTY / degraded path | Degrades according to the approved contract when interactive menu I/O is unavailable. | Pending | |
| Zsh | Init usage | Generated init script loads successfully and exposes the expected wrappers/completions. | Pending | |
| Zsh | Menu disabled | With `DX_MENU=0`, Tab behavior falls back to native completion behavior. | Pending | |
| Zsh | Successful replace | Menu selection applies the expected replacement behavior. | Pending | |
| Zsh | Cancel with typed query | Cancel preserves typed refinement according to the approved contract. | Pending | |
| Zsh | Noop/error fallback | Noop or command failure follows the approved fallback contract. | Pending | |
| Zsh | No TTY / degraded path | Degrades according to the approved contract when interactive menu I/O is unavailable. | Pending | |
| Fish | Init usage | Generated init script loads successfully and exposes the expected wrappers/completions. | Pending | |
| Fish | Menu disabled | With `DX_MENU=0`, Tab behavior falls back to native completion behavior. | Pending | |
| Fish | Successful replace | Menu selection applies the expected replacement behavior. | Pending | |
| Fish | Cancel with typed query | Cancel preserves typed refinement according to the approved contract. | Pending | |
| Fish | Noop/error fallback | Noop or command failure follows the approved fallback contract. | Pending | |
| Fish | No TTY / degraded path | Degrades according to the approved contract when interactive menu I/O is unavailable. | Pending | |
| PowerShell | Init usage | `Invoke-Expression ((& dx init pwsh | Out-String))` loads the generated hook script successfully. | Pending | |
| PowerShell | Menu disabled | With `DX_MENU=0`, Tab behavior falls back to the native/PSReadLine completion path. | Pending | |
| PowerShell | Successful replace | Menu selection applies the expected replacement behavior through PSReadLine. | Pending | |
| PowerShell | Cancel with typed query | Cancel preserves typed refinement according to the approved contract. | Pending | |
| PowerShell | Noop/error fallback | Noop or command failure follows the approved fallback contract. | Pending | |
| PowerShell | No TTY / degraded path | Degrades according to the approved contract when interactive menu I/O or PSReadLine support is unavailable. | Pending | |
