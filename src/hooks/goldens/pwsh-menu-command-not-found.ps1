using namespace System.Management.Automation
using namespace System.Management.Automation.Language
if (-not $env:DX_SESSION) {
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

    $undoOrRedo = if ($Mode -eq 'back') { 'undo' } else { 'redo' }
    $origin = $PWD.Path

    $dest = $null
    if ($Selector) {
        $navigateResult = __dx_stack_invoke -CommandArgs @('navigate', $Mode, $Selector)
        $target = $navigateResult.Output
        if ($navigateResult.ExitCode -ne 0 -or -not $target) {
            return
        }

        $previewResult = __dx_stack_invoke -CommandArgs @('stack', $undoOrRedo, '--preview', '--target', $target)
    } else {
        $previewResult = __dx_stack_invoke -CommandArgs @('stack', $undoOrRedo, '--preview')
    }
    $dest = $previewResult.Output

    if ($previewResult.ExitCode -ne 0 -or -not $dest) {
        return
    }

    __dx_set_location_native @($dest)
    if (-not $?) {
        return
    }
    $applyResult = __dx_stack_invoke -CommandArgs @('stack', $undoOrRedo, '--target', $dest)
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



Register-ArgumentCompleter -Native -CommandName 'dx' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'dx'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $i -eq ($commandElements.Count - 1)) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'dx' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('resolve', 'resolve', [CompletionResultType]::ParameterValue, 'resolve')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'init')
            [CompletionResult]::new('complete', 'complete', [CompletionResultType]::ParameterValue, 'complete')
            [CompletionResult]::new('navigate', 'navigate', [CompletionResultType]::ParameterValue, 'navigate')
            [CompletionResult]::new('bookmarks', 'bookmarks', [CompletionResultType]::ParameterValue, 'bookmarks')
            [CompletionResult]::new('stack', 'stack', [CompletionResultType]::ParameterValue, 'stack')
            [CompletionResult]::new('menu', 'menu', [CompletionResultType]::ParameterValue, 'menu')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'dx;resolve' {
            [CompletionResult]::new('--list', '--list', [CompletionResultType]::ParameterName, 'list')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;init' {
            [CompletionResult]::new('--command-not-found', '--command-not-found', [CompletionResultType]::ParameterName, 'command-not-found')
            [CompletionResult]::new('--menu', '--menu', [CompletionResultType]::ParameterName, 'menu')
            [CompletionResult]::new('--native-menu', '--native-menu', [CompletionResultType]::ParameterName, 'native-menu')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;complete' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('paths', 'paths', [CompletionResultType]::ParameterValue, 'paths')
            [CompletionResult]::new('ancestors', 'ancestors', [CompletionResultType]::ParameterValue, 'ancestors')
            [CompletionResult]::new('frecents', 'frecents', [CompletionResultType]::ParameterValue, 'frecents')
            [CompletionResult]::new('recents', 'recents', [CompletionResultType]::ParameterValue, 'recents')
            [CompletionResult]::new('stack', 'stack', [CompletionResultType]::ParameterValue, 'stack')
            [CompletionResult]::new('filesystem', 'filesystem', [CompletionResultType]::ParameterValue, 'filesystem')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'dx;complete;paths' {
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'limit')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;complete;ancestors' {
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'limit')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;complete;frecents' {
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'limit')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;complete;recents' {
            [CompletionResult]::new('--session', '--session', [CompletionResultType]::ParameterName, 'session')
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'limit')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;complete;stack' {
            [CompletionResult]::new('--direction', '--direction', [CompletionResultType]::ParameterName, 'direction')
            [CompletionResult]::new('--session', '--session', [CompletionResultType]::ParameterName, 'session')
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'limit')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;complete;filesystem' {
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'limit')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;complete;help' {
            [CompletionResult]::new('paths', 'paths', [CompletionResultType]::ParameterValue, 'paths')
            [CompletionResult]::new('ancestors', 'ancestors', [CompletionResultType]::ParameterValue, 'ancestors')
            [CompletionResult]::new('frecents', 'frecents', [CompletionResultType]::ParameterValue, 'frecents')
            [CompletionResult]::new('recents', 'recents', [CompletionResultType]::ParameterValue, 'recents')
            [CompletionResult]::new('stack', 'stack', [CompletionResultType]::ParameterValue, 'stack')
            [CompletionResult]::new('filesystem', 'filesystem', [CompletionResultType]::ParameterValue, 'filesystem')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'dx;complete;help;paths' {
            break
        }
        'dx;complete;help;ancestors' {
            break
        }
        'dx;complete;help;frecents' {
            break
        }
        'dx;complete;help;recents' {
            break
        }
        'dx;complete;help;stack' {
            break
        }
        'dx;complete;help;filesystem' {
            break
        }
        'dx;complete;help;help' {
            break
        }
        'dx;navigate' {
            [CompletionResult]::new('--session', '--session', [CompletionResultType]::ParameterName, 'session')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;bookmarks' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Save a bookmark for a directory')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a saved bookmark')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List saved bookmarks (default when no subcommand given)')
            [CompletionResult]::new('prune', 'prune', [CompletionResultType]::ParameterValue, 'Remove bookmarks whose target directory no longer exists')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'dx;bookmarks;add' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;bookmarks;remove' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Output as JSON')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;bookmarks;list' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;bookmarks;prune' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;bookmarks;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Save a bookmark for a directory')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a saved bookmark')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List saved bookmarks (default when no subcommand given)')
            [CompletionResult]::new('prune', 'prune', [CompletionResultType]::ParameterValue, 'Remove bookmarks whose target directory no longer exists')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'dx;bookmarks;help;add' {
            break
        }
        'dx;bookmarks;help;remove' {
            break
        }
        'dx;bookmarks;help;list' {
            break
        }
        'dx;bookmarks;help;prune' {
            break
        }
        'dx;bookmarks;help;help' {
            break
        }
        'dx;stack' {
            [CompletionResult]::new('--direction', '--direction', [CompletionResultType]::ParameterName, 'direction')
            [CompletionResult]::new('--session', '--session', [CompletionResultType]::ParameterName, 'session')
            [CompletionResult]::new('--list', '--list', [CompletionResultType]::ParameterName, 'list')
            [CompletionResult]::new('--clear', '--clear', [CompletionResultType]::ParameterName, 'clear')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'json')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('push', 'push', [CompletionResultType]::ParameterValue, 'push')
            [CompletionResult]::new('undo', 'undo', [CompletionResultType]::ParameterValue, 'undo')
            [CompletionResult]::new('redo', 'redo', [CompletionResultType]::ParameterValue, 'redo')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'dx;stack;push' {
            [CompletionResult]::new('--session', '--session', [CompletionResultType]::ParameterName, 'session')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;stack;undo' {
            [CompletionResult]::new('--session', '--session', [CompletionResultType]::ParameterName, 'session')
            [CompletionResult]::new('--target', '--target', [CompletionResultType]::ParameterName, 'target')
            [CompletionResult]::new('--preview', '--preview', [CompletionResultType]::ParameterName, 'Print the destination without changing session history')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;stack;redo' {
            [CompletionResult]::new('--session', '--session', [CompletionResultType]::ParameterName, 'session')
            [CompletionResult]::new('--target', '--target', [CompletionResultType]::ParameterName, 'target')
            [CompletionResult]::new('--preview', '--preview', [CompletionResultType]::ParameterName, 'Print the destination without changing session history')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;stack;help' {
            [CompletionResult]::new('push', 'push', [CompletionResultType]::ParameterValue, 'push')
            [CompletionResult]::new('undo', 'undo', [CompletionResultType]::ParameterValue, 'undo')
            [CompletionResult]::new('redo', 'redo', [CompletionResultType]::ParameterValue, 'redo')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'dx;stack;help;push' {
            break
        }
        'dx;stack;help;undo' {
            break
        }
        'dx;stack;help;redo' {
            break
        }
        'dx;stack;help;help' {
            break
        }
        'dx;menu' {
            [CompletionResult]::new('--buffer', '--buffer', [CompletionResultType]::ParameterName, 'Full command-line buffer text')
            [CompletionResult]::new('--cursor', '--cursor', [CompletionResultType]::ParameterName, 'Cursor byte position within the buffer')
            [CompletionResult]::new('--cwd', '--cwd', [CompletionResultType]::ParameterName, 'Working directory (defaults to current directory)')
            [CompletionResult]::new('--session', '--session', [CompletionResultType]::ParameterName, 'Session identifier (defaults to DX_SESSION env var)')
            [CompletionResult]::new('--prompt-row', '--prompt-row', [CompletionResultType]::ParameterName, 'Prompt row override for shells that can provide buffer cursor row')
            [CompletionResult]::new('--mode', '--mode', [CompletionResultType]::ParameterName, 'Explicit mapped-command menu mode for init-generated external command hooks')
            [CompletionResult]::new('--shell', '--shell', [CompletionResultType]::ParameterName, 'Shell syntax used for replacement text')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'dx;help' {
            [CompletionResult]::new('resolve', 'resolve', [CompletionResultType]::ParameterValue, 'resolve')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'init')
            [CompletionResult]::new('complete', 'complete', [CompletionResultType]::ParameterValue, 'complete')
            [CompletionResult]::new('navigate', 'navigate', [CompletionResultType]::ParameterValue, 'navigate')
            [CompletionResult]::new('bookmarks', 'bookmarks', [CompletionResultType]::ParameterValue, 'bookmarks')
            [CompletionResult]::new('stack', 'stack', [CompletionResultType]::ParameterValue, 'stack')
            [CompletionResult]::new('menu', 'menu', [CompletionResultType]::ParameterValue, 'menu')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'dx;help;resolve' {
            break
        }
        'dx;help;init' {
            break
        }
        'dx;help;complete' {
            [CompletionResult]::new('paths', 'paths', [CompletionResultType]::ParameterValue, 'paths')
            [CompletionResult]::new('ancestors', 'ancestors', [CompletionResultType]::ParameterValue, 'ancestors')
            [CompletionResult]::new('frecents', 'frecents', [CompletionResultType]::ParameterValue, 'frecents')
            [CompletionResult]::new('recents', 'recents', [CompletionResultType]::ParameterValue, 'recents')
            [CompletionResult]::new('stack', 'stack', [CompletionResultType]::ParameterValue, 'stack')
            [CompletionResult]::new('filesystem', 'filesystem', [CompletionResultType]::ParameterValue, 'filesystem')
            break
        }
        'dx;help;complete;paths' {
            break
        }
        'dx;help;complete;ancestors' {
            break
        }
        'dx;help;complete;frecents' {
            break
        }
        'dx;help;complete;recents' {
            break
        }
        'dx;help;complete;stack' {
            break
        }
        'dx;help;complete;filesystem' {
            break
        }
        'dx;help;navigate' {
            break
        }
        'dx;help;bookmarks' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Save a bookmark for a directory')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a saved bookmark')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List saved bookmarks (default when no subcommand given)')
            [CompletionResult]::new('prune', 'prune', [CompletionResultType]::ParameterValue, 'Remove bookmarks whose target directory no longer exists')
            break
        }
        'dx;help;bookmarks;add' {
            break
        }
        'dx;help;bookmarks;remove' {
            break
        }
        'dx;help;bookmarks;list' {
            break
        }
        'dx;help;bookmarks;prune' {
            break
        }
        'dx;help;stack' {
            [CompletionResult]::new('push', 'push', [CompletionResultType]::ParameterValue, 'push')
            [CompletionResult]::new('undo', 'undo', [CompletionResultType]::ParameterValue, 'undo')
            [CompletionResult]::new('redo', 'redo', [CompletionResultType]::ParameterValue, 'redo')
            break
        }
        'dx;help;stack;push' {
            break
        }
        'dx;help;stack;undo' {
            break
        }
        'dx;help;stack;redo' {
            break
        }
        'dx;help;menu' {
            break
        }
        'dx;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
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

if (Get-Module -Name PSReadLine -ErrorAction SilentlyContinue) {
    $Global:__dx_pwsh_menu_handler_description = 'dx menu handler'
    $dxNewMenuKey = 'Tab'
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

    function global:__dx_pwsh_capture_redraw_context {
        param(
            [string]$Line,
            [int]$Cursor,
            $RawUi
        )

        try {
            $cursorPosition = $RawUi.CursorPosition
            $windowPosition = $RawUi.WindowPosition
            $windowSize = $RawUi.WindowSize
            $bufferSize = $RawUi.BufferSize
            $width = [int]$windowSize.Width
            if ($width -le 0 -or $Cursor -lt 0 -or $Cursor -gt $Line.Length) { return $null }

            $prefix = $Line.Substring(0, $Cursor)
            $segments = [regex]::Split($prefix, "\r?\n")
            $options = Get-PSReadLineOption
            $promptText = $null
            if ($options.PromptText) {
                $promptText = [string]@($options.PromptText)[0]
            }

            if ($promptText) {
                $initialX = [int]($RawUi.LengthInBufferCells($promptText) % $width)
            } elseif ($segments.Count -eq 1) {
                $prefixCells = [int]$RawUi.LengthInBufferCells($prefix)
                $initialX = (([int]$cursorPosition.X - ($prefixCells % $width)) + $width) % $width
            } else {
                return $null
            }

            $continuationCells = [int]$RawUi.LengthInBufferCells([string]$options.ContinuationPrompt)
            $rowOffset = 0
            for ($i = 0; $i -lt $segments.Count; $i++) {
                $startX = if ($i -eq 0) { $initialX } else { $continuationCells % $width }
                $cells = [int]$RawUi.LengthInBufferCells([string]$segments[$i])
                $rowOffset += [Math]::Floor(($startX + $cells) / $width)
                if ($i -lt $segments.Count - 1) { $rowOffset += 1 }
            }

            $extraPromptLines = [Math]::Max([int]$options.ExtraPromptLineCount, 0)
            $promptTopY = [int]$cursorPosition.Y - [int]$rowOffset - $extraPromptLines
            $relativeCursorY = [int]$cursorPosition.Y - [int]$windowPosition.Y
            if ($promptTopY -lt 0 -or $relativeCursorY -lt 0) { return $null }

            return [PSCustomObject]@{
                CursorY = [int]$cursorPosition.Y
                RelativeCursorY = $relativeCursorY
                PromptTopY = $promptTopY
                WindowY = [int]$windowPosition.Y
                WindowHeight = [int]$windowSize.Height
                BufferHeight = [int]$bufferSize.Height
            }
        } catch {
            return $null
        }
    }

    function global:__dx_pwsh_resolve_redraw_y {
        param($Result, $Context)

        if ($null -eq $Result -or $null -eq $Context) { return $null }
        try {
            $redrawNumber = [double]$Result.redrawRow
            $scrollNumber = [double]$Result.scrollRows
            if (
                [double]::IsNaN($redrawNumber) -or [double]::IsInfinity($redrawNumber) -or
                [double]::IsNaN($scrollNumber) -or [double]::IsInfinity($scrollNumber) -or
                $redrawNumber -ne [Math]::Floor($redrawNumber) -or
                $scrollNumber -ne [Math]::Floor($scrollNumber)
            ) { return $null }

            $redrawRow = [int]$redrawNumber
            $scrollRows = [int]$scrollNumber
            if ($redrawRow -lt 0 -or $scrollRows -lt 0 -or $redrawRow -ge $Context.WindowHeight) {
                return $null
            }

            $expectedRedrawRow = [Math]::Max($Context.RelativeCursorY - $scrollRows, 0)
            if ($redrawRow -ne $expectedRedrawRow) { return $null }

            $targetY = $Context.PromptTopY - $scrollRows
            if ($targetY -lt $Context.WindowY -or $targetY -ge $Context.BufferHeight) { return $null }
            return [int]$targetY
        } catch {
            return $null
        }
    }

    function global:__dx_pwsh_invoke_prompt_at {
        param([int]$RedrawY)

        [Console]::SetCursorPosition(0, $RedrawY)
        [Console]::Write("`e[0J")
        [Microsoft.PowerShell.PSConsoleReadLine]::InvokePrompt($null, $RedrawY)
    }

    $dxMappingSeeds = @()
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

    Set-PSReadLineKeyHandler -Key 'Tab' -BriefDescription 'dx menu' -Description $Global:__dx_pwsh_menu_handler_description -ScriptBlock {
        param($key, $arg)

        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
        $cursorBytes = [System.Text.Encoding]::UTF8.GetByteCount($line.Substring(0, $cursor))

        $promptRow = $null
        $redrawContext = $null
        try {
            $rawUi = $Host.UI.RawUI
            $redrawContext = __dx_pwsh_capture_redraw_context -Line $line -Cursor $cursor -RawUi $rawUi
            if ($null -ne $redrawContext) { $promptRow = $redrawContext.RelativeCursorY }
        } catch {}

        $dxCmds = @('cd', 'up', 'cdf', 'z', 'cdr', 'back', 'forward', 'cd-', 'cd+')
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
                    $json = (dx menu --shell pwsh --mode $dxMenuMode --buffer $line --cursor $cursorBytes --cwd $PWD.Path --session $env:DX_SESSION --prompt-row $promptRow)
                } else {
                    $json = (dx menu --shell pwsh --buffer $line --cursor $cursorBytes --cwd $PWD.Path --session $env:DX_SESSION --prompt-row $promptRow)
                }
            } else {
                if ($dxMenuMode) {
                    $json = (dx menu --shell pwsh --mode $dxMenuMode --buffer $line --cursor $cursorBytes --cwd $PWD.Path --session $env:DX_SESSION)
                } else {
                    $json = (dx menu --shell pwsh --buffer $line --cursor $cursorBytes --cwd $PWD.Path --session $env:DX_SESSION)
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

        $redrawY = $null
        if ($result -and $result.terminal -eq 'dirty') {
            $redrawY = __dx_pwsh_resolve_redraw_y -Result $result -Context $redrawContext
        }

        if ($result -and $result.action -eq 'cancel') {
            [Microsoft.PowerShell.PSConsoleReadLine]::SetCursorPosition($cursor)
            if ($null -ne $redrawY) {
                try {
                    __dx_pwsh_invoke_prompt_at -RedrawY ([int]$redrawY)
                } catch {
                    [Microsoft.PowerShell.PSConsoleReadLine]::InvokePrompt()
                }
            } else {
                [Microsoft.PowerShell.PSConsoleReadLine]::InvokePrompt()
            }
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
        if ($result.terminal -eq 'dirty' -and $null -eq $redrawY) {
            __dx_pwsh_menu_fallback $key $arg
            return
        }
        if ($result.replaceStart -lt 0 -or $result.replaceEnd -lt $result.replaceStart -or $result.replaceEnd -gt $line.Length) {
            __dx_pwsh_menu_fallback $key $arg
            return
        }

        # Re-anchor before editing the buffer. Replace renders immediately, and
        # until the explicit-Y InvokePrompt moves the cached PSReadLine origin
        # to the post-scroll row, that render lands scrollRows too low: a slow
        # terminal paints the stale row, and a replacement wrapping past the
        # last row scrolls the console out from under the redraw target that
        # was computed before dx ran.
        $dxRedrawn = $false
        if ($result.terminal -eq 'dirty') {
            try {
                __dx_pwsh_invoke_prompt_at -RedrawY ([int]$redrawY)
                $dxRedrawn = $true
            } catch { }
        }

        [Microsoft.PowerShell.PSConsoleReadLine]::Replace($result.replaceStart, $result.replaceEnd - $result.replaceStart, $result.value)
        [Microsoft.PowerShell.PSConsoleReadLine]::SetCursorPosition($result.replaceStart + $result.value.Length)
        if ($result.terminal -eq 'dirty' -and -not $dxRedrawn) {
            [Microsoft.PowerShell.PSConsoleReadLine]::InvokePrompt()
        }
    }
}

if ($ExecutionContext.InvokeCommand.PSObject.Properties.Name -contains 'CommandNotFoundAction') {
    $script:__dx_command_not_found_handler = [System.EventHandler[System.Management.Automation.CommandLookupEventArgs]]{
        param($dxSender, $dxEventArgs)

        $cmd = $dxEventArgs.CommandName
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
            $dxEventArgs.StopSearch = $true
            $dxEventArgs.CommandScriptBlock = { }
        }
    }

    $ExecutionContext.InvokeCommand.CommandNotFoundAction = $script:__dx_command_not_found_handler
    $script:__dx_installed_command_not_found_action = $true
}

$ExecutionContext.SessionState.Module.OnRemove += {
    __dx_restore_aliases

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
    Remove-Item -LiteralPath Function:global:__dx_pwsh_menu_fallback -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath Function:global:__dx_pwsh_capture_redraw_context -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath Function:global:__dx_pwsh_resolve_redraw_y -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath Function:global:__dx_pwsh_invoke_prompt_at -Force -ErrorAction SilentlyContinue

    if ($script:__dx_has_command_not_found_action -and $script:__dx_installed_command_not_found_action) {
        $ExecutionContext.InvokeCommand.CommandNotFoundAction = $script:__dx_previous_command_not_found_action
    }
}

Export-ModuleMember -Function Set-DxLocation, Step-Up, Undo-Location, Redo-Location, Set-FrecentLocation, Set-RecentLocation
} | Import-Module -Global
