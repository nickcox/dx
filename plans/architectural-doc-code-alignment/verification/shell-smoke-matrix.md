# Shell Smoke Matrix

> Verification matrix for [architectural-doc-code-alignment](../plan.md).
>
> Phase 3 marks each row `Pass` or `Not Feasible` with concise evidence notes (command snippet, observed behavior, or reason).

| Shell | Scenario | Expected Outcome | Status | Evidence / Notes |
|-------|----------|------------------|--------|------------------|
| Bash | Init usage | Generated init script loads successfully and exposes the expected wrappers/completions. | Pass | `eval "$(./target/debug/dx init bash)"` PASS; `eval "$(./target/debug/dx init bash --menu)"` PASS. |
| Bash | Menu disabled | With `DX_MENU=0`, Tab behavior falls back to native completion behavior. | Pass | Menu-disabled fallback wrapper path with `DX_MENU=0` PASS. |
| Bash | Successful replace | Menu selection applies the expected replacement behavior. | Not Feasible | Interactive replace-selection scenario not captured in this non-interactive harness; only generated-hook contract and fallback/no-TTY checks were executed. |
| Bash | Cancel with typed query | Cancel preserves typed refinement according to the approved contract. | Not Feasible | Interactive cancel-with-query-change scenario not captured in this non-interactive harness. |
| Bash | Noop/error fallback | Noop or command failure follows the approved fallback contract. | Pass | Bash noop/error fallback wrapper path (simulated `dx menu` noop/error) PASS. |
| Bash | No TTY / degraded path | Degrades according to the approved contract when interactive menu I/O is unavailable. | Pass | `dx menu --buffer "cd foo" --cursor 6 </dev/null` => `{"action":"noop"}` PASS. |
| Zsh | Init usage | Generated init script loads successfully and exposes the expected wrappers/completions. | Pass | `autoload -Uz compinit && compinit; eval "$(./target/debug/dx init zsh)"` PASS; `autoload -Uz compinit && compinit; eval "$(./target/debug/dx init zsh --menu)"` PASS. |
| Zsh | Menu disabled | With `DX_MENU=0`, Tab behavior falls back to native completion behavior. | Not Feasible | Interactive widget scenario requires active ZLE; non-interactive invocation errors with `widgets can only be called when ZLE is active`. |
| Zsh | Successful replace | Menu selection applies the expected replacement behavior. | Not Feasible | Interactive widget scenario requires active ZLE; non-interactive invocation errors with `widgets can only be called when ZLE is active`. |
| Zsh | Cancel with typed query | Cancel preserves typed refinement according to the approved contract. | Not Feasible | Interactive widget scenario requires active ZLE; non-interactive invocation errors with `widgets can only be called when ZLE is active`. |
| Zsh | Noop/error fallback | Noop or command failure follows the approved fallback contract. | Not Feasible | Interactive widget scenario requires active ZLE; non-interactive invocation errors with `widgets can only be called when ZLE is active`. |
| Zsh | No TTY / degraded path | Degrades according to the approved contract when interactive menu I/O is unavailable. | Pass | `dx menu --buffer "cd foo" --cursor 6 </dev/null` => `{"action":"noop"}` PASS. |
| Fish | Init usage | Generated init script loads successfully and exposes the expected wrappers/completions. | Not Feasible | `command -v fish` => missing in environment. |
| Fish | Menu disabled | With `DX_MENU=0`, Tab behavior falls back to native completion behavior. | Not Feasible | `command -v fish` => missing in environment. |
| Fish | Successful replace | Menu selection applies the expected replacement behavior. | Not Feasible | `command -v fish` => missing in environment. |
| Fish | Cancel with typed query | Cancel preserves typed refinement according to the approved contract. | Not Feasible | `command -v fish` => missing in environment. |
| Fish | Noop/error fallback | Noop or command failure follows the approved fallback contract. | Not Feasible | `command -v fish` => missing in environment. |
| Fish | No TTY / degraded path | Degrades according to the approved contract when interactive menu I/O is unavailable. | Not Feasible | `command -v fish` => missing in environment. |
| PowerShell | Init usage | `Invoke-Expression ((& dx init pwsh | Out-String))` loads the generated hook script successfully. | Pass | `Invoke-Expression ((& ./target/debug/dx init pwsh | Out-String))` PASS; `Invoke-Expression ((& ./target/debug/dx init pwsh --menu | Out-String))` PASS. |
| PowerShell | Menu disabled | With `DX_MENU=0`, Tab behavior falls back to the native/PSReadLine completion path. | Not Feasible | Interactive PSReadLine fallback scenario is not reproducible in this non-interactive harness. |
| PowerShell | Successful replace | Menu selection applies the expected replacement behavior through PSReadLine. | Not Feasible | Interactive PSReadLine replace scenario is not reproducible in this non-interactive harness. |
| PowerShell | Cancel with typed query | Cancel preserves typed refinement according to the approved contract. | Not Feasible | Interactive PSReadLine cancel scenario is not reproducible in this non-interactive harness. |
| PowerShell | Noop/error fallback | Noop or command failure follows the approved fallback contract. | Not Feasible | Interactive PSReadLine noop/error fallback scenario is not reproducible in this non-interactive harness. |
| PowerShell | No TTY / degraded path | Degrades according to the approved contract when interactive menu I/O or PSReadLine support is unavailable. | Pass | `dx menu --buffer "cd foo" --cursor 6 </dev/null | ConvertFrom-Json` => `action=noop` PASS. |
