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
        'dx;bookmarks;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Save a bookmark for a directory')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a saved bookmark')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List saved bookmarks (default when no subcommand given)')
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


Register-ArgumentCompleter -CommandName Set-DxLocation,cd,Set-Location -ParameterName Path -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    __dx_emit_native_completion (__dx_complete_json -Mode paths -Word $wordToComplete) -Directory
}

Register-ArgumentCompleter -CommandName Step-Up,up,'..' -ParameterName Selector -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    __dx_emit_native_completion (__dx_complete_json -Mode ancestors -Word $wordToComplete)
}

Register-ArgumentCompleter -CommandName Undo-Location,back,'cd-' -ParameterName Selector -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    __dx_emit_native_completion (__dx_complete_json -Mode stack -Word $wordToComplete -ExtraArgs @('--direction', 'back'))
}

Register-ArgumentCompleter -CommandName Redo-Location,forward,'cd+' -ParameterName Selector -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    __dx_emit_native_completion (__dx_complete_json -Mode stack -Word $wordToComplete -ExtraArgs @('--direction', 'forward'))
}

Register-ArgumentCompleter -CommandName Set-FrecentLocation,cdf,z -ParameterName Query -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    __dx_emit_native_completion (__dx_complete_json -Mode frecents -Word $wordToComplete)
}

Register-ArgumentCompleter -CommandName Set-RecentLocation,cdr -ParameterName Query -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    __dx_emit_native_completion (__dx_complete_json -Mode recents -Word $wordToComplete)
}


function __dx_unquote_completion_word {
    param([string]$Word)

    $value = $Word
    if ($value -and $value[0] -eq "'") {
        $value = $value.Substring(1)
    }
    if ($value -and $value[$value.Length - 1] -eq "'") {
        $value = $value.Substring(0, $value.Length - 1)
    }
    return $value.Replace("''", "'")
}

function __dx_complete_json {
    param(
        [string]$Mode,
        [AllowEmptyString()]
        [string]$Word,
        [string[]]$ExtraArgs
    )

    if (-not (Get-Command dx -CommandType Application -ErrorAction SilentlyContinue)) {
        return @()
    }

    $dxArgs = @('complete', $Mode)
    if ($ExtraArgs) {
        $dxArgs += $ExtraArgs
    }
    $dxArgs += '--json'

    $limit = 1000L
    $configuredLimit = 0L
    if (
        $env:DX_MAX_MENU_RESULTS -and
        [long]::TryParse($env:DX_MAX_MENU_RESULTS.Trim(), [ref]$configuredLimit) -and
        $configuredLimit -ge 1
    ) {
        $limit = $configuredLimit
    }
    $probeLimit = if ($limit -lt [long]::MaxValue) { $limit + 1 } else { $limit }
    $dxArgs += @('--limit', [string]$probeLimit)

    $query = __dx_unquote_completion_word $Word
    if ($Mode -eq 'paths' -and -not $query) {
        $query = './'
    }
    if ($null -ne $query) {
        $dxArgs += @($query)
    }

    try {
        $json = (& dx @dxArgs 2>$null | Out-String)
        if ($LASTEXITCODE -ne 0 -or -not $json) {
            return @()
        }
        $candidates = @($json | ConvertFrom-Json)
        $hasMore = $candidates.Count -gt $limit
        if ($hasMore) {
            $candidates = @($candidates | Select-Object -First $limit)
            foreach ($candidate in $candidates) {
                $candidate | Add-Member -NotePropertyName __dxShowingFirst -NotePropertyValue $limit -Force
            }
        }
        return $candidates
    } catch {
        return @()
    }
}

function __dx_quote_completion_path {
    param([string]$Path, [switch]$Directory)

    $value = $Path
    if (
        $Directory -and
        -not $value.EndsWith([System.IO.Path]::DirectorySeparatorChar) -and
        -not $value.EndsWith([System.IO.Path]::AltDirectorySeparatorChar)
    ) {
        $value += [System.IO.Path]::DirectorySeparatorChar
    }
    # Backslash is the canonical Windows path separator and is neither an escape
    # character nor an operator in PowerShell, so it must not trigger quoting.
    if ($value -notmatch '[\s()\[\]{}!#$&*?;<>|''"`~]') {
        return $value
    }
    return "'" + $value.Replace("'", "''") + "'"
}

function __dx_native_item_max_len {
    $default = 80L
    if (-not $env:DX_MENU_ITEM_MAX_LEN) { return $default }

    $configured = 0L
    if (-not [long]::TryParse($env:DX_MENU_ITEM_MAX_LEN.Trim(), [ref]$configured)) {
        return $default
    }
    if ($configured -le 0) { return $null }
    return [Math]::Min($configured, [int]::MaxValue)
}

function __dx_truncate_native_label {
    param([string]$Label, [Nullable[int]]$MaxLength)

    if ($null -eq $MaxLength) { return $Label }
    $starts = [System.Globalization.StringInfo]::ParseCombiningCharacters($Label)
    if ($starts.Count -le $MaxLength) { return $Label }
    if ($MaxLength -le 1) { return '…' }

    $tailStart = $starts[$starts.Count - ($MaxLength - 1)]
    return '…' + $Label.Substring($tailStart)
}

function __dx_emit_native_completion {
    param([object[]]$Candidates, [switch]$Directory)

    $itemMaxLength = __dx_native_item_max_len
    foreach ($candidate in $Candidates) {
        if ($null -eq $candidate -or -not $candidate.path) { continue }
        $tooltip = [string]$candidate.path
        $showingFirst = $candidate.PSObject.Properties['__dxShowingFirst']
        if ($showingFirst) {
            $tooltip += " | showing first $($showingFirst.Value)"
        }
        [System.Management.Automation.CompletionResult]::new(
            (__dx_quote_completion_path ([string]$candidate.path) -Directory:$Directory),
            (__dx_truncate_native_label ([string]$candidate.label) $itemMaxLength),
            'ParameterValue',
            $tooltip
        )
    }
}

function __dx_resolve_completion_command {
    param([string]$CommandName)

    $command = Get-Command $CommandName -ErrorAction SilentlyContinue | Select-Object -First 1
    $seen = @{}
    while ($command -is [System.Management.Automation.AliasInfo] -and -not $seen.ContainsKey($command.Name)) {
        $seen[$command.Name] = $true
        $command = Get-Command $command.Definition -ErrorAction SilentlyContinue | Select-Object -First 1
    }
    return $command
}

function __dx_completion_parameter {
    param([System.Management.Automation.CommandInfo]$Command)

    if (-not $Command -or -not $Command.Parameters) { return $null }
    if ($Command.Parameters.ContainsKey('Path')) { return 'Path' }
    if ($Command.Parameters.ContainsKey('LiteralPath')) { return 'LiteralPath' }

    $positional = foreach ($parameter in $Command.Parameters.Values) {
        if ($parameter.ParameterType -ne [string] -and $parameter.ParameterType -ne [string[]]) {
            continue
        }
        foreach ($attribute in $parameter.Attributes) {
            if ($attribute -is [System.Management.Automation.ParameterAttribute] -and $attribute.Position -ge 0) {
                [PSCustomObject]@{ Name = $parameter.Name; Position = $attribute.Position }
            }
        }
    }
    return ($positional | Sort-Object Position, Name | Select-Object -First 1 -ExpandProperty Name)
}

function __dx_register_native_mapped_completion {
    param([string]$CommandName, [string]$Kind)

    $command = __dx_resolve_completion_command $CommandName
    if (
        -not $command -or
        $command.CommandType -in @(
            [System.Management.Automation.CommandTypes]::Application,
            [System.Management.Automation.CommandTypes]::ExternalScript
        )
    ) {
        $completer = switch ($Kind) {
            'path' {
                { param($wordToComplete, $commandAst, $cursorPosition); __dx_emit_native_completion (__dx_complete_json -Mode filesystem -Word $wordToComplete -ExtraArgs @('path')) }
            }
            'directory' {
                { param($wordToComplete, $commandAst, $cursorPosition); __dx_emit_native_completion (__dx_complete_json -Mode filesystem -Word $wordToComplete -ExtraArgs @('directory')) -Directory }
            }
            'file' {
                { param($wordToComplete, $commandAst, $cursorPosition); __dx_emit_native_completion (__dx_complete_json -Mode filesystem -Word $wordToComplete -ExtraArgs @('file')) }
            }
        }
        Register-ArgumentCompleter -Native -CommandName $CommandName -ScriptBlock $completer
        return
    }

    $parameter = __dx_completion_parameter $command
    if ($parameter) {
        $completer = switch ($Kind) {
            'path' {
                { param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters); __dx_emit_native_completion (__dx_complete_json -Mode filesystem -Word $wordToComplete -ExtraArgs @('path')) }
            }
            'directory' {
                { param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters); __dx_emit_native_completion (__dx_complete_json -Mode filesystem -Word $wordToComplete -ExtraArgs @('directory')) -Directory }
            }
            'file' {
                { param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters); __dx_emit_native_completion (__dx_complete_json -Mode filesystem -Word $wordToComplete -ExtraArgs @('file')) }
            }
        }
        Register-ArgumentCompleter -CommandName $CommandName -ParameterName $parameter -ScriptBlock $completer
    } else {
        [Console]::Error.WriteLine("dx init: warning: no path-like parameter found for native menu mapping '$CommandName'")
    }
}

function __dx_register_native_mapped_completions {
    param([string[]]$Mappings)

    $explicit = @{}
    $derived = @{}
    foreach ($entry in $Mappings) {
        $parts = $entry -split '=', 2
        if ($parts.Count -ne 2) { continue }
        $explicit[$parts[0]] = $parts[1]
    }

    foreach ($entry in $Mappings) {
        $parts = $entry -split '=', 2
        if ($parts.Count -ne 2) { continue }
        try {
            foreach ($alias in Get-Alias -Definition $parts[0] -ErrorAction SilentlyContinue) {
                if (-not $explicit.ContainsKey($alias.Name) -and -not $derived.ContainsKey($alias.Name)) {
                    $derived[$alias.Name] = $parts[1]
                }
            }
        } catch { }
    }

    foreach ($command in $derived.Keys) {
        __dx_register_native_mapped_completion -CommandName $command -Kind $derived[$command]
    }
    foreach ($command in $explicit.Keys) {
        __dx_register_native_mapped_completion -CommandName $command -Kind $explicit[$command]
    }
}

__dx_register_native_mapped_completions @()

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

    if ($script:__dx_has_command_not_found_action -and $script:__dx_installed_command_not_found_action) {
        $ExecutionContext.InvokeCommand.CommandNotFoundAction = $script:__dx_previous_command_not_found_action
    }
}

Export-ModuleMember -Function Set-DxLocation, Step-Up, Undo-Location, Redo-Location, Set-FrecentLocation, Set-RecentLocation
} | Import-Module -Global
