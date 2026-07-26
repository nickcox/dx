
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
