use super::common::{
    apply_template_replacements, pwsh_quoted_words, render_pwsh_completion_bindings,
    render_pwsh_menu_mapping_list, MENU_ELIGIBLE_COMMANDS,
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

pub fn generate(command_not_found: bool, menu: bool) -> String {
    generate_with_mappings_and_menu_key(command_not_found, menu, &[], "Tab")
}

pub fn generate_with_mappings(
    command_not_found: bool,
    menu: bool,
    _mappings: &[MenuCommandMapping],
) -> String {
    generate_with_mappings_and_menu_key(command_not_found, menu, _mappings, "Tab")
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

if (-not (Get-Variable -Name __dx_oldpwd -Scope Global -ErrorAction SilentlyContinue)) {
    $Global:__dx_oldpwd = $PWD.Path
}

Remove-Item Alias:cd -ErrorAction SilentlyContinue

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

function cd {
    param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Args)

    $Global:__dx_oldpwd = $PWD.Path

    if (-not $Args -or $Args.Count -eq 0) {
        __dx_push_pwd
        __dx_set_location_native @("~")
        if ($?) { __dx_push_pwd }
        return
    }

    if ($Args.Count -eq 1 -and $Args[0] -eq '-') {
        __dx_push_pwd
        __dx_set_location_native @($Global:__dx_oldpwd)
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

function up {
    param([string]$Selector)
    __dx_nav_wrapper -Mode up -Selector $Selector
}

function back {
    param([string]$Selector)
    __dx_stack_wrapper -Mode back -Selector $Selector
}

function forward {
    param([string]$Selector)
    __dx_stack_wrapper -Mode forward -Selector $Selector
}

Set-Alias -Name 'cd-' -Value back -Scope Global
Set-Alias -Name 'cd+' -Value forward -Scope Global

function cdf {
    param([string]$Query)
    $target = __dx_complete_first (__dx_complete_mode -Mode frecents -Word $Query)
    if ($target) {
        __dx_set_location_native @($target)
        if ($?) { __dx_push_pwd }
    }
}

Set-Alias -Name z -Value cdf -Scope Global

function cdr {
    param([string]$Query)
    $target = __dx_complete_first (__dx_complete_mode -Mode recents -Word $Query)
    if ($target) {
        __dx_set_location_native @($target)
        if ($?) { __dx_push_pwd }
    }
}

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

    if ($Global:__dx_pwsh_menu_key -and $Global:__dx_pwsh_menu_key -ne $dxNewMenuKey) {
        try {
            $oldHandler = Get-PSReadLineKeyHandler -Chord $Global:__dx_pwsh_menu_key -ErrorAction SilentlyContinue
            if ($oldHandler -and $oldHandler.Description -eq $Global:__dx_pwsh_menu_handler_description) {
                switch ($Global:__dx_pwsh_menu_previous_function) {
                    'AcceptAndGetNext' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function AcceptAndGetNext; break }
                    'AcceptLine' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function AcceptLine; break }
                    'AcceptNextSuggestionWord' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function AcceptNextSuggestionWord; break }
                    'AcceptSuggestion' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function AcceptSuggestion; break }
                    'BeginningOfHistory' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function BeginningOfHistory; break }
                    'ClearHistory' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function ClearHistory; break }
                    'Complete' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function Complete; break }
                    'EndOfHistory' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function EndOfHistory; break }
                    'ForwardSearchHistory' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function ForwardSearchHistory; break }
                    'HistorySearchBackward' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function HistorySearchBackward; break }
                    'HistorySearchForward' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function HistorySearchForward; break }
                    'MenuComplete' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function MenuComplete; break }
                    'NextHistory' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function NextHistory; break }
                    'PossibleCompletions' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function PossibleCompletions; break }
                    'PrependAndAccept' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function PrependAndAccept; break }
                    'PreviousHistory' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function PreviousHistory; break }
                    'ReverseSearchHistory' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function ReverseSearchHistory; break }
                    'TabCompleteNext' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function TabCompleteNext; break }
                    'TabCompletePrevious' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function TabCompletePrevious; break }
                    'ValidateAndAcceptLine' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function ValidateAndAcceptLine; break }
                    'ViAcceptLine' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function ViAcceptLine; break }
                    'ViAcceptLineOrExit' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function ViAcceptLineOrExit; break }
                    'ViSearchHistoryBackward' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function ViSearchHistoryBackward; break }
                    'ViTabCompleteNext' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function ViTabCompleteNext; break }
                    'ViTabCompletePrevious' { Set-PSReadLineKeyHandler -Key $Global:__dx_pwsh_menu_key -Function ViTabCompletePrevious; break }
                    default { Remove-PSReadLineKeyHandler -Chord $Global:__dx_pwsh_menu_key -ErrorAction SilentlyContinue }
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

    Set-PSReadLineKeyHandler -Key '__DX_PWSH_MENU_KEY__' -BriefDescription 'dx menu' -Description $Global:__dx_pwsh_menu_handler_description -ScriptBlock {
        param($key, $arg)

        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

        $promptRow = $null
        try {
            $rawUi = $Host.UI.RawUI
            $cursorY = [int]$rawUi.CursorPosition.Y
            $windowY = [int]$rawUi.WindowPosition.Y
            $relativeY = $cursorY - $windowY
            if ($relativeY -ge 0) { $promptRow = $relativeY }
        } catch {}

        $dxCmds = @(__DX_MENU_ELIGIBLE_COMMANDS__)
        $dxMapped = @(__DX_MENU_MAPPINGS__)
        $first = ($line -split '\s+', 2)[0]
        $dxMenuMode = $null

        foreach ($entry in $dxMapped) {
            $parts = $entry -split '=', 2
            if ($parts.Count -eq 2 -and $parts[0] -eq $first) {
                $dxMenuMode = $parts[1]
                break
            }
        }

        if ($env:DX_MENU -eq '0' -or -not (Get-Command dx -ErrorAction SilentlyContinue) -or ($first -notin $dxCmds -and -not $dxMenuMode)) {
            __dx_pwsh_menu_fallback $key $arg
            return
        }

        $json = $null
        try {
            if ($null -ne $promptRow) {
                if ($dxMenuMode) {
                    $json = (dx menu --mode $dxMenuMode --buffer $line --cursor $cursor --cwd $PWD.Path --session $env:DX_SESSION --prompt-row $promptRow --psreadline-mode)
                } else {
                    $json = (dx menu --buffer $line --cursor $cursor --cwd $PWD.Path --session $env:DX_SESSION --prompt-row $promptRow --psreadline-mode)
                }
            } else {
                if ($dxMenuMode) {
                    $json = (dx menu --mode $dxMenuMode --buffer $line --cursor $cursor --cwd $PWD.Path --session $env:DX_SESSION --psreadline-mode)
                } else {
                    $json = (dx menu --buffer $line --cursor $cursor --cwd $PWD.Path --session $env:DX_SESSION --psreadline-mode)
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

        [Microsoft.PowerShell.PSConsoleReadLine]::Replace($result.replaceStart, $result.replaceEnd - $result.replaceStart, $result.value)
        [Microsoft.PowerShell.PSConsoleReadLine]::SetCursorPosition($result.replaceStart + $result.value.Length)
        [Microsoft.PowerShell.PSConsoleReadLine]::InvokePrompt()
    }
}
"#,
        );
    }

    if command_not_found {
        script.push_str(
            r#"
if ($ExecutionContext.InvokeCommand.PSObject.Properties.Name -contains 'CommandNotFoundAction') {
    $Global:__dx_command_not_found_handler = [System.EventHandler[System.Management.Automation.CommandLookupEventArgs]]{
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

    $ExecutionContext.InvokeCommand.CommandNotFoundAction = $Global:__dx_command_not_found_handler
}
"#,
        );
    }

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
