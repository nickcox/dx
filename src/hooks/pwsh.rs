pub fn generate(command_not_found: bool) -> String {
    let mut script = String::from(
        r#"if (-not $env:DX_SESSION) {
    $env:DX_SESSION = [string]$PID
}

if (-not (Get-Variable -Name __dx_oldpwd -Scope Global -ErrorAction SilentlyContinue)) {
    $Global:__dx_oldpwd = $PWD.Path
}

function __dx_is_path_like {
    param([string]$Cmd)
    return $Cmd -match '(/|^\.|^~|^\.{3,}$)'
}

function __dx_push_pwd {
    if (Get-Command dx -ErrorAction SilentlyContinue) {
        dx push $PWD.Path *> $null
    }
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
"#,
    );

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
