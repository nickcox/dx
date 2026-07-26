

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

__dx_register_native_mapped_completions @(__DX_MENU_MAPPINGS__)
