pub fn generate(command_not_found: bool, menu: bool) -> String {
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
    return $Cmd -match '(/|^\.|^~|^\.{3,}$)'
}

function __dx_push_pwd {
    if (Get-Command dx -ErrorAction SilentlyContinue) {
        dx push $PWD.Path *> $null
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

        $dest = (dx $undoOrRedo --target $target)
    } else {
        $dest = (dx $undoOrRedo)
    }

    if ($LASTEXITCODE -ne 0 -or -not $dest) {
        return
    }

    __dx_set_location_native @($dest)
}

function __dx_set_location_native {
    param([string[]]$Args)
    Set-Location @Args
}

function cd {
    param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Args)

    $Global:__dx_oldpwd = $PWD.Path

    if (-not $Args -or $Args.Count -eq 0) {
        __dx_set_location_native @("~")
        if ($?) { __dx_push_pwd }
        return
    }

    if ($Args.Count -eq 1 -and $Args[0] -eq '-') {
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

Register-ArgumentCompleter -CommandName dx -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $elements = @($commandAst.CommandElements | ForEach-Object { $_.Extent.Text })
    if ($elements.Count -le 1) {
        __dx_emit_completion @('resolve', 'complete', 'init', 'mark', 'unmark', 'bookmarks', 'push', 'pop', 'undo', 'redo', 'navigate')
        return
    }

    $sub = $elements[1]
    switch ($sub) {
        'resolve' {
            __dx_emit_completion (__dx_complete_mode -Mode paths -Word $wordToComplete)
            break
        }
        'complete' {
            if ($elements.Count -le 3) {
                __dx_emit_completion @('paths', 'ancestors', 'frecents', 'recents', 'stack')
            }
            break
        }
        default {
            break
        }
    }
}

Register-ArgumentCompleter -CommandName cd,Set-Location -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    __dx_emit_completion (__dx_complete_mode -Mode paths -Word $wordToComplete)
}

Register-ArgumentCompleter -CommandName up -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    __dx_emit_completion (__dx_complete_mode -Mode ancestors -Word $wordToComplete)
}

Register-ArgumentCompleter -CommandName cdf,z -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    __dx_emit_completion (__dx_complete_mode -Mode frecents -Word $wordToComplete)
}

Register-ArgumentCompleter -CommandName cdr -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    __dx_emit_completion (__dx_complete_mode -Mode recents -Word $wordToComplete)
}

Register-ArgumentCompleter -CommandName back,cd- -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    __dx_emit_completion (__dx_complete_mode -Mode stack -Word $wordToComplete -ExtraArgs @('--direction', 'back'))
}

Register-ArgumentCompleter -CommandName forward,cd+ -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    __dx_emit_completion (__dx_complete_mode -Mode stack -Word $wordToComplete -ExtraArgs @('--direction', 'forward'))
}
"#,
    );

    if menu {
        script.push_str(
            r#"
if (Get-Module -Name PSReadLine -ErrorAction SilentlyContinue) {
    Set-PSReadLineKeyHandler -Key Tab -ScriptBlock {
        param($key, $arg)

        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

        $dxCmds = @('cd', 'up', 'cdf', 'z', 'cdr', 'back', 'forward', 'cd-', 'cd+')
        $first = ($line -split '\s+', 2)[0]

        if ($env:DX_MENU -eq '0' -or -not (Get-Command dx -ErrorAction SilentlyContinue) -or $first -notin $dxCmds) {
            [Microsoft.PowerShell.PSConsoleReadLine]::TabCompleteNext($key, $arg)
            return
        }

        $json = $null
        try {
            $json = (dx menu --buffer $line --cursor $cursor --cwd $PWD.Path --session $env:DX_SESSION 2>$null)
        } catch { }

        if ($LASTEXITCODE -ne 0 -or -not $json) {
            [Microsoft.PowerShell.PSConsoleReadLine]::TabCompleteNext($key, $arg)
            return
        }

        $result = $null
        try {
            $result = $json | ConvertFrom-Json
        } catch { }

        if (-not $result -or $result.action -ne 'replace') {
            [Microsoft.PowerShell.PSConsoleReadLine]::TabCompleteNext($key, $arg)
            return
        }

        [Microsoft.PowerShell.PSConsoleReadLine]::Replace($result.replaceStart, $result.replaceEnd - $result.replaceStart, $result.value)
        [Microsoft.PowerShell.PSConsoleReadLine]::SetCursorPosition($result.replaceStart + $result.value.Length)
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

    script
}
