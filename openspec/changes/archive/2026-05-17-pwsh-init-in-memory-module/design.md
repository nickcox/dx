## Context

`dx init pwsh` currently emits one large script that is evaluated directly into the user's PowerShell session. That script installs session-level behavior by defining functions, replacing aliases, registering argument completers, optionally binding PSReadLine menu handling, and optionally assigning `CommandNotFoundAction`.

This works for startup, but the resulting integration has no native PowerShell lifecycle boundary. Users cannot remove the integration as a unit with `Remove-Module dx`, and cleanup for import-time side effects is not centralized. `cd-extras` demonstrates a simpler PowerShell-native pattern: define a real location wrapper command, alias `cd` to that command, and restore the original alias from module unload cleanup.

This change keeps the current single-binary `dx init pwsh` distribution model. It does not introduce packaged `.psm1` files or a cached module file; instead, the generated init script creates and imports an in-memory module.

## Goals / Non-Goals

**Goals:**

- Make `dx init pwsh` load a module named `dx` so PowerShell can inspect and unload the integration as one unit.
- Move helper functions and saved state into module scope rather than loose caller scope.
- Replace the current literal `function cd` wrapper with a real dx location wrapper command and `Set-Item Alias:cd <wrapper>`.
- Save prior session state before replacing aliases or hooks, then restore or remove dx-owned state during module `OnRemove`.
- Preserve existing `dx init pwsh` feature behavior for navigation wrappers, completions, menu handling, command-not-found handling, environment-based mappings, and menu key configuration.

**Non-Goals:**

- Creating a file-backed generated module cache.
- Shipping a static PowerShell module in release packages.
- Publishing through PowerShell Gallery.
- Rewriting the location wrapper as a full `Set-Location`-compatible advanced function with steppable-pipeline forwarding.
- Changing Bash, Zsh, or Fish hook behavior.

## Decisions

1. Generate an in-memory module, not a file-backed module.

`dx init pwsh` should continue printing a profile-evaluable script. That script will create the module with `New-Module -Name dx -ScriptBlock { ... }` and import it into the session. This is the smallest change that gives PowerShell module identity, module-scoped state, and `OnRemove` cleanup while avoiding cache invalidation, filesystem writes, and package layout decisions.

Alternative considered: write a generated `.psm1` into a versioned cache and import it by path. That improves stack traces and inspectability, but introduces file lifecycle concerns that are better handled after the module boundary and cleanup semantics are proven.

2. Use a named wrapper command plus a `cd` alias.

The module should define a real command such as `Set-DxLocation` for the current cd behavior, then install `cd` with `Set-Item Alias:cd Set-DxLocation`. This follows the `cd-extras` pattern and avoids exporting or defining a command literally named `cd`.

Alternative considered: keep defining `function cd` inside the generated script. That preserves the current shape but misses the PowerShell-native alias pattern and makes cleanup less consistent with other alias-backed commands.

3. Keep the existing location-wrapper semantics for now.

`Set-DxLocation` should initially preserve the current `cd` wrapper behavior: no-arg home navigation, `cd -` oldpwd handling, dx resolution for ordinary path arguments, and stack push before and after successful directory changes. It should not attempt full `Set-Location` parameter parity in this change.

Alternative considered: implement a full `Set-LocationEx`-style advanced function now. That would improve PowerShell correctness but expands scope into provider paths, `-LiteralPath`, `-PassThru`, pipeline input, and native parameter binding. That deeper wrapper can be specified as a separate follow-up.

4. Save and restore session state through module scope.

At module import, capture prior state for each side effect the module will modify. At minimum this includes the prior `cd` alias target. Where feasible, also capture prior aliases/functions for dx-owned command names, the prior PSReadLine key handler metadata, and the prior `CommandNotFoundAction`. During `OnRemove`, restore captured prior state or remove dx-installed state when no prior state existed.

Alternative considered: only restore `cd`, matching the minimum `cd-extras` pattern. That leaves other dx-installed aliases and hooks behind after `Remove-Module dx`, which weakens the value of moving to a module lifecycle.

5. Preserve environment-driven configuration at init generation time.

The Rust side should continue reading `DX_MENU_COMMAND_MAPPINGS` and `DX_PWSH_MENU_KEY` when generating the PowerShell hook, and the generated script should preserve current runtime checks for menu enablement and command-not-found enablement. `Import-Module -ArgumentList` may be used internally for module options if useful, but introducing a public options hashtable is not required by this change.

Alternative considered: move all configuration parsing into PowerShell module import parameters. That is attractive long-term, but this change should avoid changing the established `dx init pwsh --menu --command-not-found` contract.

## Risks / Trade-offs

- In-memory module source is less inspectable than a cached `.psm1` -> Keep generated tests focused on module markers and consider file-backed generation as a follow-up.
- Alias/function restoration can accidentally remove user state created after import -> Restore only captured state, and when removing dx-owned state, check that the current target still matches what dx installed.
- PowerShell command precedence may differ from the current literal `function cd` wrapper -> Use `Set-Item Alias:cd Set-DxLocation` explicitly and test generated output for that contract.
- `Remove-Module dx` cleanup may not be able to perfectly restore every PSReadLine handler shape -> Preserve the existing best-effort fallback logic and centralize it in `OnRemove`.
- Keeping current `Set-DxLocation` argument parsing preserves existing limitations -> Document full `Set-LocationEx` parity as a non-goal and follow-up rather than mixing lifecycle and behavior changes.
