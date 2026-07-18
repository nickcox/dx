use super::common::{
    MENU_ELIGIBLE_COMMANDS, apply_template_replacements, pwsh_quoted_words,
    render_pwsh_completion_bindings, render_pwsh_menu_mapping_list,
};
use thiserror::Error;

use super::MenuCommandMapping;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PwshMenuKeyError {
    #[error("key contains unsupported character {0:?}")]
    UnsafeCharacter(char),
}

pub fn parse_pwsh_menu_key(raw: &str) -> Result<String, PwshMenuKeyError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok("Tab".to_string());
    }

    for ch in trimmed.chars() {
        if matches!(ch, '\n' | '\r' | '\'' | '"') {
            return Err(PwshMenuKeyError::UnsafeCharacter(ch));
        }
    }

    Ok(trimmed.to_string())
}

pub fn generate_with_mappings_and_menu_key(
    command_not_found: bool,
    menu: bool,
    _mappings: &[MenuCommandMapping],
    menu_key: &str,
) -> String {
    let mut script = String::from(
        r#"if (-not $env:DX_SESSION) {
    $env:DX_SESSION = [string]$PID
}

Get-Module -Name dx | Remove-Module -ErrorAction SilentlyContinue

$__dx_previous_aliases = @{}
foreach ($__dx_alias_name in @('cd', 'up', '..', 'back', 'forward', 'cd-', 'cd+', 'cdf', 'cdr', 'z')) {
    $__dx_alias = Get-Alias -Name $__dx_alias_name -ErrorAction SilentlyContinue
    if ($__dx_alias) {
        $__dx_previous_aliases[$__dx_alias_name] = [PSCustomObject]@{
            Definition = $__dx_alias.Definition
            Options = $__dx_alias.Options
        }
    } else {
        $__dx_previous_aliases[$__dx_alias_name] = $null
    }
}

$Global:__dx_previous_aliases_for_cleanup = $__dx_previous_aliases
function global:__dx_restore_aliases {
    foreach ($__dx_alias_name in @('cd', 'up', '..', 'back', 'forward', 'cd-', 'cd+', 'cdf', 'cdr', 'z')) {
        $__dx_previous = $Global:__dx_previous_aliases_for_cleanup[$__dx_alias_name]
        if ($null -ne $__dx_previous) {
            Set-Alias -Name $__dx_alias_name -Value $__dx_previous.Definition -Scope Global -Option $__dx_previous.Options -Force
        } else {
            Remove-Item -LiteralPath "Alias:\$__dx_alias_name" -Force -ErrorAction SilentlyContinue
        }
    }
    Remove-Variable -Name __dx_previous_aliases_for_cleanup -Scope Global -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath Function:global:__dx_restore_aliases -Force -ErrorAction SilentlyContinue
}

New-Module -Name dx -ScriptBlock {

$script:__dx_oldpwd = $PWD.Path
$script:__dx_has_command_not_found_action = $ExecutionContext.InvokeCommand.PSObject.Properties.Name -contains 'CommandNotFoundAction'
$script:__dx_previous_command_not_found_action = $null
$script:__dx_installed_command_not_found_action = $false
if ($script:__dx_has_command_not_found_action) {
    $script:__dx_previous_command_not_found_action = $ExecutionContext.InvokeCommand.CommandNotFoundAction
}
$script:__dx_installed_menu_key = $null

function __dx_is_path_like {
    param([string]$Cmd)
    return $Cmd -match '(/|^\.|^~|^\.{3,}$|-|_|\.\.)'
}

function __dx_push_pwd {
    if (Get-Command dx -ErrorAction SilentlyContinue) {
        dx stack push $PWD.Path *> $null
    }
}

function __dx_complete_first {
    param([string[]]$Lines)

    foreach ($line in $Lines) {
        if ($line) {
            return $line
        }
    }
    return $null
}

function __dx_complete_mode {
    param(
        [string]$Mode,
        [string]$Word,
        [string[]]$ExtraArgs
    )

    if (-not (Get-Command dx -ErrorAction SilentlyContinue)) {
        return @()
    }

    $args = @("complete", $Mode)
    if ($ExtraArgs) {
        $args += $ExtraArgs
    }
    if ($Word) {
        $args += @($Word)
    }

    $output = (& dx @args 2>$null)
    if ($LASTEXITCODE -ne 0) {
        return @()
    }

    return @($output | Where-Object { $_ -and $_.Trim().Length -gt 0 })
}

function __dx_nav_wrapper {
    param(
        [ValidateSet('up')]
        [string]$Mode,
        [string]$Selector
    )

    if (-not (Get-Command dx -ErrorAction SilentlyContinue)) {
        return
    }

    __dx_push_pwd

    $target = $null
    if ($Selector) {
        $target = (dx navigate $Mode $Selector)
    } else {
        $target = (dx navigate $Mode)
    }

    if ($LASTEXITCODE -ne 0 -or -not $target) {
        return
    }

    __dx_set_location_native @($target)
    if ($?) {
        __dx_push_pwd
    }
}

function __dx_stack_wrapper {
    param(
        [ValidateSet('back', 'forward')]
        [string]$Mode,
        [string]$Selector
    )

    if (-not (Get-Command dx -ErrorAction SilentlyContinue)) {
        return
    }

    $undoOrRedo = if ($Mode -eq 'back') { 'undo' } else { 'redo' }

    $dest = $null
    if ($Selector) {
        $target = (dx navigate $Mode $Selector)
        if ($LASTEXITCODE -ne 0 -or -not $target) {
            return
        }

        $dest = (dx stack $undoOrRedo --target $target)
    } else {
        $dest = (dx stack $undoOrRedo)
    }

    if ($LASTEXITCODE -ne 0 -or -not $dest) {
        return
    }

    __dx_set_location_native @($dest)
}

function __dx_set_location_native {
    param([string[]]$PathArgs)
    Set-Location @PathArgs
}

function __dx_set_alias {
    param(
        [string]$Name,
        [string]$Value,
        [object]$Options
    )

    if ($PSBoundParameters.ContainsKey('Options')) {
        Set-Alias -Name $Name -Value $Value -Scope Global -Option $Options -Force
        return
    }

    $existing = Get-Alias -Name $Name -ErrorAction SilentlyContinue
    if ($existing) {
        Set-Alias -Name $Name -Value $Value -Scope Global -Option $existing.Options -Force
    } else {
        Set-Alias -Name $Name -Value $Value -Scope Global -Force
    }
}

function Set-DxLocation {
    param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Args)

    $script:__dx_oldpwd = $PWD.Path

    if (-not $Args -or $Args.Count -eq 0) {
        __dx_push_pwd
        __dx_set_location_native @("~")
        if ($?) { __dx_push_pwd }
        return
    }

    if ($Args.Count -eq 1 -and $Args[0] -eq '-') {
        __dx_push_pwd
        __dx_set_location_native @($script:__dx_oldpwd)
        if ($?) { __dx_push_pwd }
        return
    }

    $flags = New-Object System.Collections.Generic.List[string]
    $pathArg = $null
    foreach ($arg in $Args) {
        if (-not $pathArg -and $arg.StartsWith('-') -and $arg -ne '-') {
            $flags.Add($arg)
        } elseif (-not $pathArg) {
            $pathArg = $arg
        }
    }

    if (-not $pathArg) {
        __dx_set_location_native $Args
        return
    }

    __dx_push_pwd
    $resolved = $null
    $resolveStatus = 1
    if (Get-Command dx -ErrorAction SilentlyContinue) {
        $resolved = (dx resolve $pathArg 2>$null)
        $resolveStatus = $LASTEXITCODE
    }

    if ($resolveStatus -eq 0 -and $resolved) {
        $nativeArgs = @()
        if ($flags.Count -gt 0) { $nativeArgs += $flags.ToArray() }
        $nativeArgs += @($resolved)
        __dx_set_location_native $nativeArgs
    } else {
        __dx_set_location_native $Args
    }

    if ($?) { __dx_push_pwd }
}

__dx_set_alias cd Set-DxLocation

function Step-Up {
    param([string]$Selector)
    __dx_nav_wrapper -Mode up -Selector $Selector
}

function Undo-Location {
    param([string]$Selector)
    __dx_stack_wrapper -Mode back -Selector $Selector
}

function Redo-Location {
    param([string]$Selector)
    __dx_stack_wrapper -Mode forward -Selector $Selector
}

__dx_set_alias up Step-Up
__dx_set_alias '..' Step-Up
__dx_set_alias back Undo-Location
__dx_set_alias 'cd-' Undo-Location
__dx_set_alias forward Redo-Location
__dx_set_alias 'cd+' Redo-Location

function Set-FrecentLocation {
    param([string]$Query)
    $target = __dx_complete_first (__dx_complete_mode -Mode frecents -Word $Query)
    if ($target) {
        __dx_push_pwd
        __dx_set_location_native @($target)
        if ($?) { __dx_push_pwd }
    }
}

__dx_set_alias cdf Set-FrecentLocation
__dx_set_alias z Set-FrecentLocation

function Set-RecentLocation {
    param([string]$Query)
    $target = __dx_complete_first (__dx_complete_mode -Mode recents -Word $Query)
    if ($target) {
        __dx_push_pwd
        __dx_set_location_native @($target)
        if ($?) { __dx_push_pwd }
    }
}

__dx_set_alias cdr Set-RecentLocation

function __dx_emit_completion {
    param([string[]]$Values)

    foreach ($value in $Values) {
        [System.Management.Automation.CompletionResult]::new($value, $value, 'ParameterValue', $value)
    }
}

__DX_PWSH_COMPLETION_BINDINGS__
"#,
    );

    if menu {
        script.push_str(
            r#"
if (Get-Module -Name PSReadLine -ErrorAction SilentlyContinue) {
    $Global:__dx_pwsh_menu_handler_description = 'dx menu handler'
    $dxNewMenuKey = '__DX_PWSH_MENU_KEY__'
    $script:__dx_installed_menu_key = $dxNewMenuKey
    $dxPreviousMenuKey = $null
    $dxPreviousMenuKeyVariable = Get-Variable -Name __dx_pwsh_menu_key -Scope Global -ErrorAction SilentlyContinue
    if ($dxPreviousMenuKeyVariable) {
        $dxPreviousMenuKey = $dxPreviousMenuKeyVariable.Value
    }

    if ($dxPreviousMenuKey -and $dxPreviousMenuKey -ne $dxNewMenuKey) {
        try {
            $oldHandler = Get-PSReadLineKeyHandler -Chord $dxPreviousMenuKey -ErrorAction SilentlyContinue
            if ($oldHandler -and $oldHandler.Description -eq $Global:__dx_pwsh_menu_handler_description) {
                $dxPreviousFunction = $null
                $dxPreviousFunctionVariable = Get-Variable -Name __dx_pwsh_menu_previous_function -Scope Global -ErrorAction SilentlyContinue
                if ($dxPreviousFunctionVariable) {
                    $dxPreviousFunction = $dxPreviousFunctionVariable.Value
                }
                switch ($dxPreviousFunction) {
                    'AcceptAndGetNext' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function AcceptAndGetNext; break }
                    'AcceptLine' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function AcceptLine; break }
                    'AcceptNextSuggestionWord' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function AcceptNextSuggestionWord; break }
                    'AcceptSuggestion' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function AcceptSuggestion; break }
                    'BeginningOfHistory' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function BeginningOfHistory; break }
                    'ClearHistory' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function ClearHistory; break }
                    'Complete' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function Complete; break }
                    'EndOfHistory' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function EndOfHistory; break }
                    'ForwardSearchHistory' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function ForwardSearchHistory; break }
                    'HistorySearchBackward' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function HistorySearchBackward; break }
                    'HistorySearchForward' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function HistorySearchForward; break }
                    'MenuComplete' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function MenuComplete; break }
                    'NextHistory' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function NextHistory; break }
                    'PossibleCompletions' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function PossibleCompletions; break }
                    'PrependAndAccept' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function PrependAndAccept; break }
                    'PreviousHistory' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function PreviousHistory; break }
                    'ReverseSearchHistory' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function ReverseSearchHistory; break }
                    'TabCompleteNext' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function TabCompleteNext; break }
                    'TabCompletePrevious' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function TabCompletePrevious; break }
                    'ValidateAndAcceptLine' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function ValidateAndAcceptLine; break }
                    'ViAcceptLine' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function ViAcceptLine; break }
                    'ViAcceptLineOrExit' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function ViAcceptLineOrExit; break }
                    'ViSearchHistoryBackward' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function ViSearchHistoryBackward; break }
                    'ViTabCompleteNext' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function ViTabCompleteNext; break }
                    'ViTabCompletePrevious' { Set-PSReadLineKeyHandler -Key $dxPreviousMenuKey -Function ViTabCompletePrevious; break }
                    default { Remove-PSReadLineKeyHandler -Chord $dxPreviousMenuKey -ErrorAction SilentlyContinue }
                }
            }
        } catch { }
    }

    $Global:__dx_pwsh_menu_key = $dxNewMenuKey
    $dxWarnCustomAction = $false
    try {
        $previousHandler = Get-PSReadLineKeyHandler -Chord $Global:__dx_pwsh_menu_key -ErrorAction SilentlyContinue
        if ($previousHandler -and $previousHandler.Description -eq $Global:__dx_pwsh_menu_handler_description) {
            if (-not (Get-Variable -Name __dx_pwsh_menu_previous_function -Scope Global -ErrorAction SilentlyContinue)) {
                $Global:__dx_pwsh_menu_previous_function = $null
            }
        } elseif ($previousHandler) {
            $Global:__dx_pwsh_menu_previous_function = $previousHandler.Function
            if ($Global:__dx_pwsh_menu_previous_function -eq 'CustomAction') { $dxWarnCustomAction = $true }
        } else {
            $Global:__dx_pwsh_menu_previous_function = $null
        }
    } catch { }

    if ($dxWarnCustomAction) {
        [Console]::Error.WriteLine("dx init: warning: PSReadLine key '$Global:__dx_pwsh_menu_key' was bound to a CustomAction; dx cannot replay that handler, so fallback will use TabCompleteNext")
    }

    function global:__dx_pwsh_menu_fallback {
        param($key, $arg)

        switch ($Global:__dx_pwsh_menu_previous_function) {
            'AcceptAndGetNext' { [Microsoft.PowerShell.PSConsoleReadLine]::AcceptAndGetNext($key, $arg); return }
            'AcceptLine' { [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine($key, $arg); return }
            'AcceptNextSuggestionWord' { [Microsoft.PowerShell.PSConsoleReadLine]::AcceptNextSuggestionWord($key, $arg); return }
            'AcceptSuggestion' { [Microsoft.PowerShell.PSConsoleReadLine]::AcceptSuggestion($key, $arg); return }
            'BeginningOfHistory' { [Microsoft.PowerShell.PSConsoleReadLine]::BeginningOfHistory($key, $arg); return }
            'ClearHistory' { [Microsoft.PowerShell.PSConsoleReadLine]::ClearHistory($key, $arg); return }
            'Complete' { [Microsoft.PowerShell.PSConsoleReadLine]::Complete($key, $arg); return }
            'EndOfHistory' { [Microsoft.PowerShell.PSConsoleReadLine]::EndOfHistory($key, $arg); return }
            'ForwardSearchHistory' { [Microsoft.PowerShell.PSConsoleReadLine]::ForwardSearchHistory($key, $arg); return }
            'HistorySearchBackward' { [Microsoft.PowerShell.PSConsoleReadLine]::HistorySearchBackward($key, $arg); return }
            'HistorySearchForward' { [Microsoft.PowerShell.PSConsoleReadLine]::HistorySearchForward($key, $arg); return }
            'MenuComplete' { [Microsoft.PowerShell.PSConsoleReadLine]::MenuComplete($key, $arg); return }
            'NextHistory' { [Microsoft.PowerShell.PSConsoleReadLine]::NextHistory($key, $arg); return }
            'PossibleCompletions' { [Microsoft.PowerShell.PSConsoleReadLine]::PossibleCompletions($key, $arg); return }
            'PrependAndAccept' { [Microsoft.PowerShell.PSConsoleReadLine]::PrependAndAccept($key, $arg); return }
            'PreviousHistory' { [Microsoft.PowerShell.PSConsoleReadLine]::PreviousHistory($key, $arg); return }
            'ReverseSearchHistory' { [Microsoft.PowerShell.PSConsoleReadLine]::ReverseSearchHistory($key, $arg); return }
            'TabCompleteNext' { [Microsoft.PowerShell.PSConsoleReadLine]::TabCompleteNext($key, $arg); return }
            'TabCompletePrevious' { [Microsoft.PowerShell.PSConsoleReadLine]::TabCompletePrevious($key, $arg); return }
            'ValidateAndAcceptLine' { [Microsoft.PowerShell.PSConsoleReadLine]::ValidateAndAcceptLine($key, $arg); return }
            'ViAcceptLine' { [Microsoft.PowerShell.PSConsoleReadLine]::ViAcceptLine($key, $arg); return }
            'ViAcceptLineOrExit' { [Microsoft.PowerShell.PSConsoleReadLine]::ViAcceptLineOrExit($key, $arg); return }
            'ViSearchHistoryBackward' { [Microsoft.PowerShell.PSConsoleReadLine]::ViSearchHistoryBackward($key, $arg); return }
            'ViTabCompleteNext' { [Microsoft.PowerShell.PSConsoleReadLine]::ViTabCompleteNext($key, $arg); return }
            'ViTabCompletePrevious' { [Microsoft.PowerShell.PSConsoleReadLine]::ViTabCompletePrevious($key, $arg); return }
            default { [Microsoft.PowerShell.PSConsoleReadLine]::TabCompleteNext($key, $arg); return }
        }
    }

    $dxMappingSeeds = @(__DX_MENU_MAPPINGS__)
    $dxExplicitMapped = @{}
    $dxDerivedMapped = @{}

    foreach ($entry in $dxMappingSeeds) {
        $parts = $entry -split '=', 2
        if ($parts.Count -ne 2) { continue }
        $dxExplicitMapped[$parts[0]] = $parts[1]
    }

    foreach ($entry in $dxMappingSeeds) {
        $parts = $entry -split '=', 2
        if ($parts.Count -ne 2) { continue }

        $command = $parts[0]
        $mode = $parts[1]

        try {
            foreach ($alias in Get-Alias -Definition $command -ErrorAction SilentlyContinue) {
                $aliasName = $alias.Name
                if (-not $dxExplicitMapped.ContainsKey($aliasName) -and -not $dxDerivedMapped.ContainsKey($aliasName)) {
                    $dxDerivedMapped[$aliasName] = $mode
                }
            }
        } catch { }
    }

    $Global:__dx_pwsh_menu_mapped = @{}
    foreach ($key in $dxDerivedMapped.Keys) {
        $Global:__dx_pwsh_menu_mapped[$key] = $dxDerivedMapped[$key]
    }
    foreach ($key in $dxExplicitMapped.Keys) {
        $Global:__dx_pwsh_menu_mapped[$key] = $dxExplicitMapped[$key]
    }

    Set-PSReadLineKeyHandler -Key '__DX_PWSH_MENU_KEY__' -BriefDescription 'dx menu' -Description $Global:__dx_pwsh_menu_handler_description -ScriptBlock {
        param($key, $arg)

        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
        $cursorBytes = [System.Text.Encoding]::UTF8.GetByteCount($line.Substring(0, $cursor))

        $promptRow = $null
        try {
            $rawUi = $Host.UI.RawUI
            $cursorY = [int]$rawUi.CursorPosition.Y
            $windowY = [int]$rawUi.WindowPosition.Y
            $relativeY = $cursorY - $windowY
            if ($relativeY -ge 0) { $promptRow = $relativeY }
        } catch {}

        $dxCmds = @(__DX_MENU_ELIGIBLE_COMMANDS__)
        $first = ($line -split '\s+', 2)[0]
        $dxMenuMode = $null
        $dxMapped = $Global:__dx_pwsh_menu_mapped
        if ($dxMapped -and $dxMapped.ContainsKey($first)) {
            $dxMenuMode = $dxMapped[$first]
        }

        if ($env:DX_MENU -eq '0' -or -not (Get-Command dx -ErrorAction SilentlyContinue) -or ($first -notin $dxCmds -and -not $dxMenuMode)) {
            __dx_pwsh_menu_fallback $key $arg
            return
        }

        $json = $null
        try {
            if ($null -ne $promptRow) {
                if ($dxMenuMode) {
                    $json = (dx menu --shell pwsh --mode $dxMenuMode --buffer $line --cursor $cursorBytes --cwd $PWD.Path --session $env:DX_SESSION --prompt-row $promptRow --psreadline-mode)
                } else {
                    $json = (dx menu --shell pwsh --buffer $line --cursor $cursorBytes --cwd $PWD.Path --session $env:DX_SESSION --prompt-row $promptRow --psreadline-mode)
                }
            } else {
                if ($dxMenuMode) {
                    $json = (dx menu --shell pwsh --mode $dxMenuMode --buffer $line --cursor $cursorBytes --cwd $PWD.Path --session $env:DX_SESSION --psreadline-mode)
                } else {
                    $json = (dx menu --shell pwsh --buffer $line --cursor $cursorBytes --cwd $PWD.Path --session $env:DX_SESSION --psreadline-mode)
                }
            }
        } catch { }

        if ($LASTEXITCODE -ne 0 -or -not $json) {
            __dx_pwsh_menu_fallback $key $arg
            return
        }

        $result = $null
        try {
            $result = $json | ConvertFrom-Json
        } catch { }

        if ($result -and $result.action -eq 'cancel') {
            [Microsoft.PowerShell.PSConsoleReadLine]::SetCursorPosition($line.Length)
            [Microsoft.PowerShell.PSConsoleReadLine]::InvokePrompt()
            return
        }

        if (-not $result -or $result.action -ne 'replace') {
            __dx_pwsh_menu_fallback $key $arg
            return
        }

        if (-not $result.terminal -or ($result.terminal -ne 'clean' -and $result.terminal -ne 'dirty')) {
            __dx_pwsh_menu_fallback $key $arg
            return
        }
        if ($result.replaceStart -lt 0 -or $result.replaceEnd -lt $result.replaceStart -or $result.replaceEnd -gt $line.Length) {
            __dx_pwsh_menu_fallback $key $arg
            return
        }

        [Microsoft.PowerShell.PSConsoleReadLine]::Replace($result.replaceStart, $result.replaceEnd - $result.replaceStart, $result.value)
        [Microsoft.PowerShell.PSConsoleReadLine]::SetCursorPosition($result.replaceStart + $result.value.Length)
        if ($result.terminal -eq 'dirty') {
            [Microsoft.PowerShell.PSConsoleReadLine]::InvokePrompt()
        }
    }
}
"#,
        );
    }

    if command_not_found {
        script.push_str(
            r#"
if ($ExecutionContext.InvokeCommand.PSObject.Properties.Name -contains 'CommandNotFoundAction') {
    $script:__dx_command_not_found_handler = [System.EventHandler[System.Management.Automation.CommandLookupEventArgs]]{
        param($sender, $eventArgs)

        $cmd = $eventArgs.CommandName
        if ($env:DX_RESOLVE_GUARD) { return }
        if (-not (__dx_is_path_like $cmd)) { return }
        if (-not (Get-Command dx -ErrorAction SilentlyContinue)) { return }

        $env:DX_RESOLVE_GUARD = '1'
        $resolved = (dx resolve $cmd 2>$null)
        $resolveStatus = $LASTEXITCODE
        Remove-Item Env:DX_RESOLVE_GUARD -ErrorAction SilentlyContinue

        if ($resolveStatus -ne 0 -or -not $resolved) { return }

        __dx_set_location_native @($resolved)
        if ($?) {
            __dx_push_pwd
            $eventArgs.StopSearch = $true
            $eventArgs.CommandScriptBlock = { }
        }
    }

    $ExecutionContext.InvokeCommand.CommandNotFoundAction = $script:__dx_command_not_found_handler
    $script:__dx_installed_command_not_found_action = $true
}
"#,
        );
    }

    script.push_str(
        r#"
$ExecutionContext.SessionState.Module.OnRemove += {
    __dx_restore_aliases
"#,
    );

    if menu {
        script.push_str(
            r#"
    if ($script:__dx_installed_menu_key) {
        switch ($Global:__dx_pwsh_menu_previous_function) {
            'AcceptAndGetNext' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function AcceptAndGetNext; break }
            'AcceptLine' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function AcceptLine; break }
            'AcceptNextSuggestionWord' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function AcceptNextSuggestionWord; break }
            'AcceptSuggestion' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function AcceptSuggestion; break }
            'BeginningOfHistory' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function BeginningOfHistory; break }
            'ClearHistory' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function ClearHistory; break }
            'Complete' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function Complete; break }
            'EndOfHistory' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function EndOfHistory; break }
            'ForwardSearchHistory' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function ForwardSearchHistory; break }
            'HistorySearchBackward' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function HistorySearchBackward; break }
            'HistorySearchForward' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function HistorySearchForward; break }
            'MenuComplete' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function MenuComplete; break }
            'NextHistory' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function NextHistory; break }
            'PossibleCompletions' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function PossibleCompletions; break }
            'PrependAndAccept' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function PrependAndAccept; break }
            'PreviousHistory' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function PreviousHistory; break }
            'ReverseSearchHistory' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function ReverseSearchHistory; break }
            'TabCompleteNext' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function TabCompleteNext; break }
            'TabCompletePrevious' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function TabCompletePrevious; break }
            'ValidateAndAcceptLine' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function ValidateAndAcceptLine; break }
            'ViAcceptLine' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function ViAcceptLine; break }
            'ViAcceptLineOrExit' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function ViAcceptLineOrExit; break }
            'ViSearchHistoryBackward' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function ViSearchHistoryBackward; break }
            'ViTabCompleteNext' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function ViTabCompleteNext; break }
            'ViTabCompletePrevious' { Set-PSReadLineKeyHandler -Key $script:__dx_installed_menu_key -Function ViTabCompletePrevious; break }
            default { Remove-PSReadLineKeyHandler -Chord $script:__dx_installed_menu_key -ErrorAction SilentlyContinue }
        }
    }
"#,
        );
    }

    script.push_str(
        r#"
    if ($script:__dx_has_command_not_found_action -and $script:__dx_installed_command_not_found_action) {
        $ExecutionContext.InvokeCommand.CommandNotFoundAction = $script:__dx_previous_command_not_found_action
    }
}

Export-ModuleMember -Function Set-DxLocation, Step-Up, Undo-Location, Redo-Location, Set-FrecentLocation, Set-RecentLocation
} | Import-Module -Global
"#,
    );

    apply_template_replacements(
        script,
        [
            (
                "__DX_MENU_ELIGIBLE_COMMANDS__",
                pwsh_quoted_words(MENU_ELIGIBLE_COMMANDS),
            ),
            (
                "__DX_MENU_MAPPINGS__",
                render_pwsh_menu_mapping_list(_mappings),
            ),
            ("__DX_PWSH_MENU_KEY__", menu_key.to_string()),
            (
                "__DX_PWSH_COMPLETION_BINDINGS__",
                render_pwsh_completion_bindings(),
            ),
        ],
    )
}
