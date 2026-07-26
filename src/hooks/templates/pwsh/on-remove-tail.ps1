
    if ($script:__dx_has_command_not_found_action -and $script:__dx_installed_command_not_found_action) {
        $ExecutionContext.InvokeCommand.CommandNotFoundAction = $script:__dx_previous_command_not_found_action
    }
}

Export-ModuleMember -Function Set-DxLocation, Step-Up, Undo-Location, Redo-Location, Set-FrecentLocation, Set-RecentLocation
} | Import-Module -Global
