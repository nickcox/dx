# PowerShell ProxyCommand Evaluation (Phase 4)

Plan: `whole-repo-review-remediation`  
Phase: 4 (`Finalize Hygiene and PowerShell Decision`)

## Decision

**Reject `ProxyCommand` adoption for `cd` wrapper in Phase 4.**

The current explicit wrapper in `src/hooks/pwsh.rs` remains the baseline. Evaluation did not show a correctness win across the required scenario set, and it introduced behavior risk for unsupported flags while adding complexity.

## Evidence Summary

### Baseline contract markers and fallback behavior

- Verified generated hook/menu markers and noop contract (cross-shell):

```json
{"init_checks":{"bash":{"</dev/tty":true,"__dx_try_menu":true,"dx menu --buffer":true,"if __dx_try_menu; then":true,"return 0":true},"zsh":{"</dev/tty":true,"__dx_menu_widget":true,"bindkey '^I' __dx_menu_widget":true,"dx menu --buffer":true,"zle expand-or-complete":true},"fish":{"</dev/tty":true,"bind \\t __dx_menu_complete":true,"commandline -f complete":true,"dx menu --buffer":true,"function __dx_menu_complete":true},"pwsh":{"--psreadline-mode":true,"ConvertFrom-Json":true,"Set-PSReadLineKeyHandler -Key Tab":true,"TabCompleteNext":true}},"menu_noop":{"action":"noop"}}
```

- PowerShell one-script-block check (`Invoke-Expression ((& dx init pwsh --menu | Out-String))`) returned exactly:

```text
Handler=True
ConvertFromJson=True
TabCompleteNext=True
Menu={"action":"noop"}
```

### ProxyCommand prototype evaluation output

Command returned exactly:

```text
CommandType=Cmdlet
ProxyLineCount=95
HasSteppable=True
DashRestored=True
QuotedSpaceExists=True
UnsupportedFlagResult=AmbiguousParameter,Microsoft.PowerShell.Commands.SetLocationCommand
```

## Required Scenario Set Assessment

| Scenario | Result | Evidence-driven interpretation |
|---|---|---|
| Unflagged path (`cd project`) | No demonstrated correctness improvement | Current wrapper already handles the normal resolve/fallback path; evaluation output showed no net win beyond baseline behavior. |
| Previous dir (`cd -`) | No demonstrated correctness improvement | Baseline already handles `cd -` explicitly in `src/hooks/pwsh.rs`; prototype did not demonstrate superior behavior. |
| Quoted path (`cd 'path with space'`) | Parity only | `QuotedSpaceExists=True` indicates quoted-path handling exists, but not better than current wrapper contract. |
| Unsupported flags fallback | **Worse / riskier** | `UnsupportedFlagResult=AmbiguousParameter,Microsoft.PowerShell.Commands.SetLocationCommand` indicates ambiguity/error behavior instead of clear fallback parity. |
| `--menu` fallback contract (`ConvertFrom-Json` + `TabCompleteNext`) | Must remain unchanged; baseline already satisfies | Verified by one-script-block evidence and generated markers (`--psreadline-mode`, `ConvertFrom-Json`, `TabCompleteNext`, noop action). No ProxyCommand-specific improvement shown. |

## External Baseline Comparison

Reference reviewed:  
`https://raw.githubusercontent.com/nickcox/cd-extras/master/cd-extras/public/Set-LocationEx.ps1`

Observed shape is a large handwritten advanced function (`CmdletBinding` + steppable pipeline + substantial custom replace/history logic), not a thin direct `ProxyCommand` wrapper. This reinforces that adopting ProxyCommand here would move toward higher PowerShell-specific complexity, contrary to Phase 4’s default reject posture unless a clear win is proven.

## Repository Anchors Supporting Rejection

- No ProxyCommand-based implementation exists in current `src/hooks/pwsh.rs` (explicit wrapper remains authoritative).
- Existing tests anchor the required fallback contract and generated marker invariants:
  - `tests/menu_cli.rs::hook_scripts_contain_fallback_on_noop`
  - `tests/menu_cli.rs::init_pwsh_with_menu_flag_includes_psreadline_handler`
  - `tests/menu_cli.rs::menu_psreadline_mode_keeps_posix_flagged_cd_as_fallback`
  - `src/hooks/mod.rs::menu_enabled_scripts_keep_cross_shell_menu_invocation_marker`
  - `src/hooks/mod.rs::generated_scripts_do_not_leak_internal_placeholder_tokens`
- Contract reference: `tech-docs/shell-hook-guarding.md` (menu fallback + JSON parsing expectations).

## Outcome for Phase 4 artifacts

- Keep explicit `src/hooks/pwsh.rs` wrapper path.
- Mark shell smoke matrix ProxyCommand row as **Not Applicable (evaluated and rejected)**.
