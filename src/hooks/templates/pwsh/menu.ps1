
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
        $redrawContext = $null
        try {
            $rawUi = $Host.UI.RawUI
            $redrawContext = __dx_pwsh_capture_redraw_context -Line $line -Cursor $cursor -RawUi $rawUi
            if ($null -ne $redrawContext) { $promptRow = $redrawContext.RelativeCursorY }
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
