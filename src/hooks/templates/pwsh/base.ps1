if (-not $env:DX_SESSION) {
    $env:DX_SESSION = [string]$PID
}

Get-Module -Name dx | Remove-Module -ErrorAction SilentlyContinue

$__dx_previous_aliases = @{}
foreach ($__dx_alias_name in @(__DX_PWSH_MANAGED_ALIASES__)) {
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
    foreach ($__dx_alias_name in @(__DX_PWSH_MANAGED_ALIASES__)) {
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
    __dx_push_path $PWD.Path
}

function __dx_push_path {
    param([string]$Path)

    if ($Path) {
        __dx_stack_invoke -CommandArgs @('stack', 'push', $Path) *> $null
    }
}

function __dx_stack_invoke {
    param([string[]]$CommandArgs)

    $dxCommand = Get-Command dx -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $dxCommand) {
        return [PSCustomObject]@{ Output = @(); ExitCode = 127 }
    }

    $fallback = $HOME
    if (-not $fallback -and $env:USERPROFILE) {
        $fallback = $env:USERPROFILE
    }
    if (-not $fallback) {
        $fallback = [System.IO.Path]::GetTempPath()
    }
    if (-not $fallback) {
        return [PSCustomObject]@{ Output = @(); ExitCode = 1 }
    }

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $dxCommand.Source
    $startInfo.WorkingDirectory = $fallback
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    foreach ($argument in $CommandArgs) {
        [void]$startInfo.ArgumentList.Add($argument)
    }

    try {
        $process = [System.Diagnostics.Process]::Start($startInfo)
        $outputText = $process.StandardOutput.ReadToEnd()
        $process.StandardError.ReadToEnd() > $null
        $process.WaitForExit()
    } catch {
        return [PSCustomObject]@{ Output = @(); ExitCode = 1 }
    }

    $output = @()
    $trimmedOutput = $outputText.TrimEnd([char[]]"`r`n")
    if ($trimmedOutput) {
        $output = @($trimmedOutput -split "`r?`n")
    }

    return [PSCustomObject]@{ Output = $output; ExitCode = $process.ExitCode }
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

    $dxArgs = @("complete", $Mode)
    if ($ExtraArgs) {
        $dxArgs += $ExtraArgs
    }
    if ($Word) {
        $dxArgs += @($Word)
    }

    $output = (& dx @dxArgs 2>$null)
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

    $origin = $PWD.Path

    $dest = $null
    if ($Selector) {
        $navigateResult = __dx_stack_invoke -CommandArgs @('navigate', $Mode, $Selector)
        $target = $navigateResult.Output
        if ($navigateResult.ExitCode -ne 0 -or -not $target) {
            return
        }

        $previewResult = __dx_stack_invoke -CommandArgs @('stack', $Mode, '--preview', '--target', $target)
    } else {
        $previewResult = __dx_stack_invoke -CommandArgs @('stack', $Mode, '--preview')
    }
    $dest = $previewResult.Output

    if ($previewResult.ExitCode -ne 0 -or -not $dest) {
        return
    }

    __dx_set_location_native @($dest)
    if (-not $?) {
        return
    }
    $applyResult = __dx_stack_invoke -CommandArgs @('stack', $Mode, '--target', $dest)
    if ($applyResult.ExitCode -ne 0) {
        __dx_set_location_native @($origin)
    }
}

function __dx_set_location_native {
    param([string[]]$PathArgs)
    Microsoft.PowerShell.Management\Set-Location @PathArgs
}

function __dx_is_filesystem_location {
    param([System.Management.Automation.PathInfo]$Location)
    return $Location -and $Location.Provider.Name -eq 'FileSystem'
}

function __dx_is_resolvable_path {
    param([string]$Path)

    if (-not $Path -or $Path -in @('-', '+')) {
        return $false
    }

    if ($Path -match '[*?\[\]]' -or $Path -match '^[^\\/:]+::' -or $Path -match '^[A-Za-z][A-Za-z0-9_-]+:') {
        return $false
    }

    return $true
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
    [CmdletBinding(DefaultParameterSetName = 'Path')]
    param(
        [Parameter(ParameterSetName = 'Path', Position = 0, ValueFromPipeline, ValueFromPipelineByPropertyName)]
        [string]$Path,

        [Parameter(ParameterSetName = 'LiteralPath', Mandatory, ValueFromPipelineByPropertyName)]
        [Alias('PSPath', 'LP')]
        [string]$LiteralPath,

        [switch]$PassThru,

        [Parameter(ParameterSetName = 'Stack', ValueFromPipelineByPropertyName)]
        [string]$StackName
    )

    begin {
        $startLocation = Get-Location
        $nativeParameters = @{} + $PSBoundParameters

        if (
            $PSCmdlet.ParameterSetName -eq 'Path' -and
            $PSBoundParameters.ContainsKey('Path') -and
            -not $MyInvocation.ExpectingInput -and
            (__dx_is_filesystem_location $startLocation) -and
            (__dx_is_resolvable_path $Path) -and
            (Get-Command dx -ErrorAction SilentlyContinue)
        ) {
            try {
                $resolved = (dx resolve $Path 2>$null)
            } catch {
                $resolved = $null
            }
            if ($LASTEXITCODE -eq 0 -and $resolved) {
                $nativeParameters['Path'] = $resolved
            }
        }

        $nativeCommand = $ExecutionContext.InvokeCommand.GetCommand(
            'Microsoft.PowerShell.Management\Set-Location',
            [System.Management.Automation.CommandTypes]::Cmdlet
        )
        $scriptCommand = { & $nativeCommand @nativeParameters }
        $steppablePipeline = $scriptCommand.GetSteppablePipeline($MyInvocation.CommandOrigin)
        $steppablePipeline.Begin($PSCmdlet)
    }

    process {
        $steppablePipeline.Process($_)
    }

    end {
        $steppablePipeline.End()

        $endLocation = Get-Location
        if (
            (__dx_is_filesystem_location $endLocation) -and
            ($endLocation.Path -ne $startLocation.Path) -and
            (Get-Command dx -ErrorAction SilentlyContinue)
        ) {
            if (__dx_is_filesystem_location $startLocation) {
                __dx_push_path $startLocation.Path
            }
            __dx_push_path $endLocation.Path
        }
    }
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

__DX_PWSH_FRECENCY_WRAPPERS__

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
