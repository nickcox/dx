mod bash;
mod common;
mod fish;
mod mappings;
mod pwsh;
mod zsh;

pub use mappings::{
    MenuCommandMapping, MenuCommandMappingError, MenuMappingMode, parse_menu_command_mappings,
};
pub use pwsh::{PwshMenuKeyError, parse_pwsh_menu_key};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Pwsh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitMenuMode {
    Disabled,
    Tui,
    NativePwsh,
}

impl Shell {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "bash" => Some(Self::Bash),
            "zsh" => Some(Self::Zsh),
            "fish" => Some(Self::Fish),
            "pwsh" => Some(Self::Pwsh),
            _ => None,
        }
    }

    pub fn supported_list() -> &'static str {
        "bash, zsh, fish, pwsh"
    }
}

pub fn generate(shell: Shell, command_not_found: bool, menu: bool) -> String {
    generate_with_mappings_and_pwsh_key(shell, command_not_found, menu, &[], "Tab")
}

pub fn generate_with_mappings(
    shell: Shell,
    command_not_found: bool,
    menu: bool,
    mappings: &[MenuCommandMapping],
) -> String {
    generate_with_mappings_and_pwsh_key(shell, command_not_found, menu, mappings, "Tab")
}

pub fn generate_with_mappings_and_pwsh_key(
    shell: Shell,
    command_not_found: bool,
    menu: bool,
    mappings: &[MenuCommandMapping],
    pwsh_menu_key: &str,
) -> String {
    let menu_mode = if menu {
        InitMenuMode::Tui
    } else {
        InitMenuMode::Disabled
    };
    generate_with_menu_mode_and_pwsh_key(
        shell,
        command_not_found,
        menu_mode,
        mappings,
        pwsh_menu_key,
    )
}

pub fn generate_with_menu_mode_and_pwsh_key(
    shell: Shell,
    command_not_found: bool,
    menu_mode: InitMenuMode,
    mappings: &[MenuCommandMapping],
    pwsh_menu_key: &str,
) -> String {
    let menu = menu_mode == InitMenuMode::Tui;
    match shell {
        Shell::Bash => bash::generate_with_mappings(command_not_found, menu, mappings),
        Shell::Zsh => zsh::generate_with_mappings(command_not_found, menu, mappings),
        Shell::Fish => fish::generate_with_mappings(command_not_found, menu, mappings),
        Shell::Pwsh => pwsh::generate_with_mappings_and_menu_key(
            command_not_found,
            menu_mode,
            mappings,
            pwsh_menu_key,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Shell, generate, generate_with_mappings, generate_with_mappings_and_pwsh_key,
        parse_menu_command_mappings, parse_pwsh_menu_key,
    };

    fn count_unescaped(script: &str, needle: char) -> usize {
        let mut escaped = false;
        let mut count = 0;
        for ch in script.chars() {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == needle {
                count += 1;
            }
        }
        count
    }

    fn assert_balanced_delimiters(script: &str) {
        assert_balanced_braces(script);

        let double_quotes = count_unescaped(script, '"');
        assert_eq!(double_quotes % 2, 0, "unbalanced double quotes");

        let single_quotes = count_unescaped(script, '\'');
        assert_eq!(single_quotes % 2, 0, "unbalanced single quotes");
    }

    fn assert_balanced_braces(script: &str) {
        let mut braces = 0_i32;
        for ch in script.chars() {
            if ch == '{' {
                braces += 1;
            }
            if ch == '}' {
                braces -= 1;
            }
            assert!(braces >= 0, "unbalanced braces in generated script");
        }
        assert_eq!(braces, 0, "unbalanced braces in generated script");
    }

    fn section_between<'a>(script: &'a str, start: &str, end: &str) -> &'a str {
        let start_idx = script.find(start).expect("missing start marker");
        let rest = &script[start_idx..];
        let end_rel = rest.find(end).expect("missing end marker");
        &rest[..end_rel]
    }

    fn assert_contains_in_order(section: &str, markers: &[&str]) {
        let mut cursor = 0;
        for marker in markers {
            let found = section[cursor..]
                .find(marker)
                .unwrap_or_else(|| panic!("missing marker in section: {marker}"));
            cursor += found + marker.len();
        }
    }

    #[test]
    fn pwsh_menu_key_parser_defaults_and_trims() {
        assert_eq!(parse_pwsh_menu_key("Tab").expect("valid key"), "Tab");
        assert_eq!(parse_pwsh_menu_key("  F12  ").expect("valid key"), "F12");
        assert_eq!(parse_pwsh_menu_key("").expect("empty defaults"), "Tab");
        assert_eq!(parse_pwsh_menu_key("   ").expect("empty defaults"), "Tab");
    }

    #[test]
    fn pwsh_menu_key_parser_rejects_unsafe_characters() {
        assert!(parse_pwsh_menu_key("Bad'Key").is_err());
        assert!(parse_pwsh_menu_key("Bad\"Key").is_err());
        assert!(parse_pwsh_menu_key("Bad\nKey").is_err());
        assert!(parse_pwsh_menu_key("Bad\rKey").is_err());
    }

    fn assert_no_unresolved_internal_placeholders(script: &str) {
        for token in script.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
            if token.starts_with("__DX_") {
                assert!(
                    !token.ends_with("__") || token == "__DX_",
                    "found unresolved internal placeholder token: {token}"
                );
            }
        }
    }

    #[test]
    fn generate_bash_without_command_not_found_contains_cd_only() {
        let output = generate(Shell::Bash, false, false);
        assert!(output.contains("cd()"));
        assert!(output.contains("up()"));
        assert!(output.contains("back()"));
        assert!(output.contains("forward()"));
        assert!(output.contains("cdf()"));
        assert!(output.contains("cdr()"));
        assert!(output.contains("complete -F _dx -o bashdefault -o default dx"));
        assert!(output.contains("DX_SESSION"));
        assert!(!output.contains("command_not_found_handle"));
    }

    #[test]
    fn bash_back_forward_use_stack_wrapper_not_nav_wrapper() {
        let output = generate(Shell::Bash, false, false);
        // back/forward should use __dx_stack_wrapper (dx stack undo/redo), not __dx_nav_wrapper (dx stack push)
        assert!(output.contains("back() {\n  __dx_stack_wrapper back"));
        assert!(output.contains("forward() {\n  __dx_stack_wrapper forward"));
        assert!(output.contains("__dx_stack_wrapper()"));
        assert!(output.contains("__dx_stack_run stack \"$__dx_undo_or_redo\""));
    }

    #[test]
    fn bash_up_seeds_origin_before_navigate() {
        let output = generate(Shell::Bash, false, false);
        // __dx_nav_wrapper should call __dx_push_pwd before dx navigate
        let nav_wrapper_start = output
            .find("__dx_nav_wrapper()")
            .expect("expected bash nav wrapper marker to exist");
        let nav_section = &output[nav_wrapper_start..];
        let push_pos = nav_section
            .find("__dx_push_pwd")
            .expect("expected __dx_push_pwd call in bash nav wrapper section");
        let navigate_pos = nav_section
            .find("dx navigate")
            .expect("expected dx navigate call in bash nav wrapper section");
        assert!(
            push_pos < navigate_pos,
            "push_pwd should come before dx navigate in nav_wrapper"
        );
    }

    #[test]
    fn posix_jump_wrappers_seed_origin_before_cd_and_record_destination() {
        let bash = generate(Shell::Bash, false, false);
        let bash_jump = section_between(&bash, "__dx_jump_mode()", "\ncd() {");
        assert_eq!(bash_jump.matches("__dx_push_pwd").count(), 2);
        assert_contains_in_order(
            bash_jump,
            &[
                "[[ $__dx_status -eq 0 ]] || return \"$__dx_status\"",
                "[[ -n \"$__dx_target\" ]] || return 1",
                "__dx_push_pwd",
                "__dx_cd_native \"$__dx_target\" || return $?",
                "__dx_push_pwd",
            ],
        );

        let zsh = generate(Shell::Zsh, false, false);
        let zsh_jump = section_between(&zsh, "__dx_jump_mode()", "\ncd() {");
        assert_eq!(zsh_jump.matches("__dx_push_pwd").count(), 2);
        assert_contains_in_order(
            zsh_jump,
            &[
                "[[ $__dx_status -eq 0 ]] || return $__dx_status",
                "[[ -n \"$__dx_target\" ]] || return 1",
                "__dx_push_pwd",
                "builtin cd \"$__dx_target\" || return $?",
                "__dx_push_pwd",
            ],
        );

        let fish = generate(Shell::Fish, false, false);
        let fish_jump = section_between(&fish, "function __dx_jump_mode", "\nfunction cd");
        assert_eq!(fish_jump.matches("__dx_push_pwd").count(), 2);
        assert_contains_in_order(
            fish_jump,
            &[
                "if test $dx_status -ne 0",
                "return $dx_status",
                "test -n \"$target\"; or return 1",
                "__dx_push_pwd",
                "__dx_cd_native \"$target\"",
                "set dx_status $status",
                "if test $dx_status -ne 0",
                "__dx_push_pwd",
            ],
        );
    }

    #[test]
    fn pwsh_jump_wrappers_seed_origin_before_set_location_and_record_destination() {
        let output = generate(Shell::Pwsh, false, false);

        let cdf = section_between(
            &output,
            "function Set-FrecentLocation {",
            "\n__dx_set_alias cdf",
        );
        assert_eq!(cdf.matches("__dx_push_pwd").count(), 2);
        assert_contains_in_order(
            cdf,
            &[
                "$target = __dx_complete_first",
                "if ($target) {",
                "__dx_push_pwd",
                "__dx_set_location_native @($target)",
                "if ($?) { __dx_push_pwd }",
            ],
        );

        let cdr = section_between(
            &output,
            "function Set-RecentLocation {",
            "\n__dx_set_alias cdr",
        );
        assert_eq!(cdr.matches("__dx_push_pwd").count(), 2);
        assert_contains_in_order(
            cdr,
            &[
                "$target = __dx_complete_first",
                "if ($target) {",
                "__dx_push_pwd",
                "__dx_set_location_native @($target)",
                "if ($?) { __dx_push_pwd }",
            ],
        );
    }

    #[test]
    fn stack_traversal_wrappers_do_not_seed_new_origin() {
        let bash = generate(Shell::Bash, false, false);
        let bash_stack = section_between(&bash, "__dx_stack_wrapper()", "\n__dx_jump_mode()");
        assert!(!bash_stack.contains("__dx_push_pwd"));
        assert!(bash_stack.contains("--preview"));
        assert!(bash_stack.contains("--target \"$__dx_dest\" >/dev/null"));

        let zsh = generate(Shell::Zsh, false, false);
        let zsh_stack = section_between(&zsh, "__dx_stack_wrapper()", "\n__dx_jump_mode()");
        assert!(!zsh_stack.contains("__dx_push_pwd"));
        assert!(zsh_stack.contains("--preview"));
        assert!(zsh_stack.contains("--target \"$__dx_dest\" >/dev/null"));

        let fish = generate(Shell::Fish, false, false);
        let fish_stack = section_between(
            &fish,
            "function __dx_stack_wrapper",
            "\nfunction __dx_jump_mode",
        );
        assert!(!fish_stack.contains("__dx_push_pwd"));
        assert!(fish_stack.contains("--preview"));
        assert!(fish_stack.contains("--target \"$dest\" >/dev/null"));

        let pwsh = generate(Shell::Pwsh, false, false);
        let pwsh_stack = section_between(
            &pwsh,
            "function __dx_stack_wrapper",
            "\nfunction __dx_set_location_native",
        );
        assert!(!pwsh_stack.contains("__dx_push_pwd"));
        assert!(pwsh_stack.contains("--preview"));
        assert!(
            pwsh_stack.contains(
                "__dx_stack_invoke -CommandArgs @('stack', $undoOrRedo, '--target', $dest)"
            )
        );
    }

    #[test]
    fn zsh_back_forward_use_stack_wrapper() {
        let output = generate(Shell::Zsh, false, false);
        assert!(output.contains("back() {\n  __dx_stack_wrapper back"));
        assert!(output.contains("forward() {\n  __dx_stack_wrapper forward"));
        assert!(output.contains("__dx_stack_wrapper()"));
    }

    #[test]
    fn fish_back_forward_use_stack_wrapper() {
        let output = generate(Shell::Fish, false, false);
        assert!(output.contains("function back\n  __dx_stack_wrapper back"));
        assert!(output.contains("function forward\n  __dx_stack_wrapper forward"));
        assert!(output.contains("function __dx_stack_wrapper"));
    }

    #[test]
    fn generate_bash_with_command_not_found_contains_handler() {
        let output = generate(Shell::Bash, true, false);
        assert!(output.contains("cd()"));
        assert!(output.contains("DX_SESSION"));
        assert!(output.contains("command_not_found_handle"));
    }

    #[test]
    fn generate_zsh_uses_handler_suffix() {
        let output = generate(Shell::Zsh, true, false);
        assert!(output.contains("command_not_found_handler"));
        assert!(output.contains("#compdef dx"));
        assert!(output.contains("compdef _dx_complete_ancestors up"));
        assert!(!output.contains("command_not_found_handle()"));
    }

    #[test]
    fn generate_fish_without_command_not_found_excludes_handler() {
        let output = generate(Shell::Fish, false, false);
        assert!(output.contains("function cd"));
        assert!(output.contains("function up"));
        assert!(output.contains("complete -c dx"));
        assert!(output.contains("DX_SESSION"));
        assert!(!output.contains("fish_command_not_found"));
    }

    #[test]
    fn generate_pwsh_with_command_not_found_includes_guard_and_action() {
        let output = generate(Shell::Pwsh, true, false);
        assert!(output.contains("Set-Location"));
        assert!(output.contains("function Step-Up"));
        assert!(output.contains("__dx_set_alias up Step-Up"));
        assert!(output.contains("Register-ArgumentCompleter -Native -CommandName 'dx'"));
        assert!(output.contains("CommandNotFoundAction"));
        assert!(output.contains("DX_RESOLVE_GUARD"));
    }

    #[test]
    fn generate_pwsh_without_command_not_found_excludes_action() {
        let output = generate(Shell::Pwsh, false, false);
        assert!(output.contains("Set-Location"));
        assert!(!output.contains(
            "[System.EventHandler[System.Management.Automation.CommandLookupEventArgs]]"
        ));
        assert!(!output.contains("$script:__dx_installed_command_not_found_action = $true"));
    }

    #[test]
    fn pwsh_imports_in_memory_module_with_cleanup() {
        let output = generate(Shell::Pwsh, false, false);
        assert!(
            output.contains("Get-Module -Name dx | Remove-Module -ErrorAction SilentlyContinue")
        );
        assert!(
            output.contains("$Global:__dx_previous_aliases_for_cleanup = $__dx_previous_aliases")
        );
        assert!(output.contains("function global:__dx_restore_aliases {"));
        assert!(output.contains("New-Module -Name dx -ScriptBlock {"));
        assert!(output.contains("$ExecutionContext.SessionState.Module.OnRemove += {"));
        assert!(!output.contains("$MyInvocation.MyCommand.ScriptBlock.Module.OnRemove"));
        assert!(output.contains("Export-ModuleMember -Function Set-DxLocation, Step-Up, Undo-Location, Redo-Location, Set-FrecentLocation, Set-RecentLocation"));
        assert!(output.contains("foreach ($__dx_alias_name in @('cd', 'up', '..', 'back', 'forward', 'cd-', 'cd+', 'cdf', 'cdr', 'z'))"));
        assert!(output.contains("Remove-Item -LiteralPath \"Alias:\\$__dx_alias_name\" -Force -ErrorAction SilentlyContinue"));
        assert!(output.contains("__dx_restore_aliases"));
        assert!(output.contains("} | Import-Module -Global"));
    }

    #[test]
    fn pwsh_aliases_cd_to_named_location_wrapper() {
        let output = generate(Shell::Pwsh, false, false);
        assert!(output.contains("function Set-DxLocation"));
        assert!(output.contains("__dx_set_alias cd Set-DxLocation"));
        assert!(output.contains(
            "Set-Alias -Name $Name -Value $Value -Scope Global -Option $existing.Options -Force"
        ));
        assert!(!output.contains("function cd {"));
        assert!(!output.contains("Remove-Item Alias:cd -ErrorAction SilentlyContinue"));
    }

    #[test]
    fn pwsh_location_wrapper_uses_native_parameter_binding() {
        let output = generate(Shell::Pwsh, false, false);
        assert!(output.contains("[CmdletBinding(DefaultParameterSetName = 'Path')]"));
        assert!(output.contains("ValueFromPipeline, ValueFromPipelineByPropertyName"));
        assert!(output.contains("[Alias('PSPath', 'LP')]"));
        assert!(output.contains("[switch]$PassThru"));
        assert!(output.contains("[string]$StackName"));
        assert!(output.contains("'Microsoft.PowerShell.Management\\Set-Location'"));
        assert!(output.contains("GetSteppablePipeline($MyInvocation.CommandOrigin)"));
        assert!(output.contains("$steppablePipeline.Process($_)"));
        assert!(output.contains("$steppablePipeline.End()"));
        assert!(output.contains("function __dx_is_resolvable_path"));
        assert!(output.contains("$PSBoundParameters.ContainsKey('Path')"));
        assert!(output.contains("$Path -in @('-', '+')"));
        assert!(output.contains("__dx_push_path $startLocation.Path"));
        assert!(!output.contains("__dx_oldpwd"));
        assert!(!output.contains("ValueFromRemainingArguments"));
    }

    #[test]
    fn pwsh_uses_idiomatic_primary_navigation_function_names() {
        let output = generate(Shell::Pwsh, false, false);
        assert!(output.contains("function Step-Up"));
        assert!(output.contains("function Undo-Location"));
        assert!(output.contains("function Redo-Location"));
        assert!(output.contains("function Set-FrecentLocation"));
        assert!(output.contains("function Set-RecentLocation"));
        assert!(!output.contains("function up {"));
        assert!(!output.contains("function back {"));
        assert!(!output.contains("function forward {"));
        assert!(!output.contains("function cdf {"));
        assert!(!output.contains("function cdr {"));
    }

    #[test]
    fn pwsh_installs_short_navigation_aliases() {
        let output = generate(Shell::Pwsh, false, false);
        assert!(output.contains("__dx_set_alias up Step-Up"));
        assert!(output.contains("__dx_set_alias '..' Step-Up"));
        assert!(output.contains("__dx_set_alias back Undo-Location"));
        assert!(output.contains("__dx_set_alias 'cd-' Undo-Location"));
        assert!(output.contains("__dx_set_alias forward Redo-Location"));
        assert!(output.contains("__dx_set_alias 'cd+' Redo-Location"));
        assert!(output.contains("__dx_set_alias cdf Set-FrecentLocation"));
        assert!(output.contains("__dx_set_alias z Set-FrecentLocation"));
        assert!(output.contains("__dx_set_alias cdr Set-RecentLocation"));
    }

    #[test]
    fn pwsh_back_forward_use_stack_wrapper_and_undo_redo() {
        let output = generate(Shell::Pwsh, false, false);
        assert!(output.contains("function __dx_stack_wrapper"));
        assert!(output.contains("$undoOrRedo = if ($Mode -eq 'back') { 'undo' } else { 'redo' }"));
        assert!(output.contains("__dx_stack_wrapper -Mode back -Selector $Selector"));
        assert!(output.contains("__dx_stack_wrapper -Mode forward -Selector $Selector"));
    }

    #[test]
    fn generate_all_shells_guard_existing_dx_session() {
        let bash = generate(Shell::Bash, false, false);
        let zsh = generate(Shell::Zsh, false, false);
        let fish = generate(Shell::Fish, false, false);
        let pwsh = generate(Shell::Pwsh, false, false);

        assert!(bash.contains("DX_SESSION:-"));
        assert!(zsh.contains("DX_SESSION:-"));
        assert!(fish.contains("if not set -q DX_SESSION"));
        assert!(pwsh.contains("if (-not $env:DX_SESSION)"));
    }

    #[test]
    fn all_shells_freeze_wrapper_and_completion_routing_contracts() {
        let bash = generate(Shell::Bash, false, false);
        assert!(bash.contains("cd()"));
        assert!(bash.contains("up()"));
        assert!(bash.contains("cdf()"));
        assert!(bash.contains("z()"));
        assert!(bash.contains("cdr()"));
        assert!(bash.contains("back()"));
        assert!(bash.contains("cd-()"));
        assert!(bash.contains("forward()"));
        assert!(bash.contains("cd+()"));
        assert!(bash.contains("__dx_resolved=\"$(dx resolve \"$__dx_path_arg\" 2>/dev/null)\""));
        assert!(bash.contains("dx complete paths \"$cur\" 2>/dev/null"));
        assert!(bash.contains("_dx()"));
        assert!(!bash.contains("_dx_complete_dx"));
        assert!(bash.contains("--command-not-found"));
        assert!(bash.contains("bookmarks"));
        assert!(bash.contains("filesystem"));
        assert!(bash.contains("complete -o default -F _dx_complete_paths cd"));
        assert!(bash.contains("complete -F _dx_complete_stack_back cd-"));
        assert!(bash.contains("complete -F _dx_complete_stack_forward cd+"));

        let zsh = generate(Shell::Zsh, false, false);
        assert!(zsh.contains("cd()"));
        assert!(zsh.contains("up()"));
        assert!(zsh.contains("cdf()"));
        assert!(zsh.contains("z()"));
        assert!(zsh.contains("cdr()"));
        assert!(zsh.contains("back()"));
        assert!(zsh.contains("cd-()"));
        assert!(zsh.contains("forward()"));
        assert!(zsh.contains("cd+()"));
        assert!(zsh.contains("__dx_resolved=\"$(dx resolve \"$__dx_path_arg\" 2>/dev/null)\""));
        assert!(zsh.contains("dx complete paths \"$cur\" 2>/dev/null"));
        assert!(zsh.contains("#compdef dx"));
        assert!(!zsh.contains("compdef _dx_complete_dx dx"));
        assert!(zsh.contains("--command-not-found"));
        assert!(zsh.contains("bookmarks"));
        assert!(zsh.contains("filesystem"));
        assert!(zsh.contains("compdef _dx_complete_paths cd"));
        assert!(zsh.contains("compdef _dx_complete_stack_back back 'cd-'"));
        assert!(zsh.contains("compdef _dx_complete_stack_forward forward 'cd+'"));

        let fish = generate(Shell::Fish, false, false);
        assert!(fish.contains("function cd"));
        assert!(fish.contains("function up"));
        assert!(fish.contains("function cdf"));
        assert!(fish.contains("function z"));
        assert!(fish.contains("function cdr"));
        assert!(fish.contains("function back"));
        assert!(fish.contains("function cd-"));
        assert!(fish.contains("function forward"));
        assert!(fish.contains("function cd+"));
        assert!(fish.contains("set -l __dx_resolved (dx resolve \"$__dx_path_arg\" 2>/dev/null)"));
        assert!(fish.contains("dx complete paths (commandline -ct) 2>/dev/null"));
        assert!(fish.contains("complete -c dx"));
        assert!(fish.contains("-l command-not-found"));
        assert!(
            fish.contains("complete -c cd -a '(dx complete paths (commandline -ct) 2>/dev/null)'")
        );
        assert!(fish.contains("complete -c back -a '(dx complete stack --direction back (commandline -ct) 2>/dev/null)'"));
        assert!(fish.contains("complete -c cd+ -a '(dx complete stack --direction forward (commandline -ct) 2>/dev/null)'"));

        let pwsh = generate(Shell::Pwsh, false, false);
        assert!(pwsh.contains("function Set-DxLocation"));
        assert!(pwsh.contains("__dx_set_alias cd Set-DxLocation"));
        assert!(pwsh.contains("function Step-Up"));
        assert!(pwsh.contains("__dx_set_alias up Step-Up"));
        assert!(pwsh.contains("__dx_set_alias '..' Step-Up"));
        assert!(pwsh.contains("function Set-FrecentLocation"));
        assert!(pwsh.contains("__dx_set_alias cdf Set-FrecentLocation"));
        assert!(pwsh.contains("__dx_set_alias z Set-FrecentLocation"));
        assert!(pwsh.contains("function Set-RecentLocation"));
        assert!(pwsh.contains("__dx_set_alias cdr Set-RecentLocation"));
        assert!(pwsh.contains("function Undo-Location"));
        assert!(pwsh.contains("__dx_set_alias back Undo-Location"));
        assert!(pwsh.contains("__dx_set_alias 'cd-' Undo-Location"));
        assert!(pwsh.contains("function Redo-Location"));
        assert!(pwsh.contains("__dx_set_alias forward Redo-Location"));
        assert!(pwsh.contains("__dx_set_alias 'cd+' Redo-Location"));
        assert!(pwsh.contains("$resolved = (dx resolve $Path 2>$null)"));
        assert!(pwsh.contains("__dx_complete_mode -Mode paths -Word $wordToComplete"));
        assert!(pwsh.contains("Register-ArgumentCompleter -Native -CommandName 'dx'"));
        assert!(pwsh.contains("--command-not-found"));
        assert!(pwsh.contains("bookmarks"));
        assert!(
            pwsh.contains("Register-ArgumentCompleter -CommandName cd,Set-Location -ScriptBlock")
        );
        assert!(pwsh.contains("Register-ArgumentCompleter -CommandName back,cd- -ScriptBlock"));
        assert!(pwsh.contains("Register-ArgumentCompleter -CommandName forward,cd+ -ScriptBlock"));
    }

    #[test]
    fn all_shells_freeze_menu_fallback_contract_markers() {
        let bash = generate(Shell::Bash, false, true);
        assert!(!bash.contains(";;&"), "menu mappings must support Bash 3.2");
        assert!(bash.contains("__dx_json=\"$(dx menu --shell bash --buffer \"$COMP_LINE\" --cursor \"$COMP_POINT\" --cwd \"$PWD\" --session \"${DX_SESSION:-}\" </dev/tty 2>/dev/tty)\" || return 1"));
        assert!(bash.contains("[[ \"$__dx_action\" == \"cancel\" ]] && return 0"));
        assert!(bash.contains("[[ \"$__dx_action\" == \"replace\" ]] || return 1"));
        assert!(bash.contains("(( __dx_re >= __dx_rs )) || return 1"));
        assert!(bash.contains(
            "__dx_terminal=\"$(__dx_json_extract_string terminal \"$__dx_json\")\" || return 1"
        ));
        assert!(bash.contains(
            "[[ \"$__dx_terminal\" == \"clean\" || \"$__dx_terminal\" == \"dirty\" ]] || return 1"
        ));
        assert!(bash.contains("__dx_menu_terminal=\"$__dx_terminal\""));
        assert!(bash.contains(
            "[[ \"$__dx_menu_terminal\" == \"dirty\" && -t 1 ]] && printf '\\r' >/dev/tty"
        ));
        assert!(bash.contains(
            "if __dx_try_menu; then\n    [[ \"$__dx_menu_terminal\" == \"dirty\" && -t 1 ]] && printf '\\r' >/dev/tty\n    return 0\n  fi"
        ));
        assert!(bash.contains("case \"$__dx_cmd\" in"));

        let zsh = generate(Shell::Zsh, false, true);
        assert!(
            zsh.contains("if [[ $__dx_exit -ne 0 ]]; then\n    zle expand-or-complete\n    return")
        );
        assert!(zsh.contains("if [[ \"$__dx_action\" == \"cancel\" ]]; then"));
        assert!(zsh.contains("CURSOR=${#BUFFER}"));
        assert!(zsh.contains("zle reset-prompt"));
        assert!(zsh.contains(
            "[[ \"$__dx_action\" == \"replace\" ]] || { zle expand-or-complete; return }"
        ));
        assert!(zsh.contains("(( __dx_re >= __dx_rs )) || { zle expand-or-complete; return }"));
        assert!(zsh.contains("(( __dx_re <= ${#BUFFER} )) || { zle expand-or-complete; return }"));
        assert!(zsh.contains("(( __dx_closed )) || { zle expand-or-complete; return }"));
        assert!(zsh.contains("[[ -n \"$__dx_value\" ]] || { zle expand-or-complete; return }"));
        assert!(zsh.contains("local __dx_terminal_marker=\"\\\"terminal\\\":\\\"\""));
        assert!(zsh.contains(
            "[[ \"$__dx_terminal\" == \"clean\" || \"$__dx_terminal\" == \"dirty\" ]] || { zle expand-or-complete; return }"
        ));
        assert!(zsh.contains("[[ \"$__dx_terminal\" == \"dirty\" ]] && zle reset-prompt"));

        let fish = generate(Shell::Fish, false, true);
        assert!(fish.contains("set -l json (dx menu --shell fish --buffer \"$buf\" --cursor $cur --cwd \"$PWD\" --session \"$DX_SESSION\" </dev/tty 2>/dev/tty)"));
        assert!(fish.contains("if test $status -ne 0\n    commandline -f complete\n    return"));
        assert!(fish.contains("if test \"$action\" = \"cancel\""));
        assert!(fish.contains("commandline -C (string length -- \"$buf\")"));
        assert!(fish.contains("if test \"$action\" != \"replace\""));
        assert!(fish.contains("if test (count $value_match) -lt 2"));
        assert!(fish.contains("set -l terminal (string replace -r '.*\\\"terminal\\\":\\\"([^\\\"[:space:]]+)\\\".*' '$1' -- \"$json\")"));
        assert!(fish.contains("if test \"$terminal\" != \"clean\" -a \"$terminal\" != \"dirty\""));
        assert!(fish.contains("if test $re -lt $rs"));
        assert!(fish.contains("if test $rs -gt $buflen; or test $re -gt $buflen"));
        assert!(fish.contains("if test \"$terminal\" = \"dirty\"\n    commandline -f repaint"));

        let pwsh = generate(Shell::Pwsh, false, true);
        assert!(pwsh.contains("$dxNewMenuKey = 'Tab'"));
        assert!(pwsh.contains("$Global:__dx_pwsh_menu_key = $dxNewMenuKey"));
        assert!(pwsh.contains("Get-PSReadLineKeyHandler -Chord $Global:__dx_pwsh_menu_key"));
        assert!(pwsh.contains("$Global:__dx_pwsh_menu_handler_description = 'dx menu handler'"));
        assert!(pwsh.contains("function global:__dx_pwsh_menu_fallback"));
        assert!(pwsh.contains(
            "$previousHandler.Description -eq $Global:__dx_pwsh_menu_handler_description"
        ));
        assert!(pwsh.contains("$dxPreviousMenuKeyVariable = Get-Variable -Name __dx_pwsh_menu_key -Scope Global -ErrorAction SilentlyContinue"));
        assert!(pwsh.contains("Remove-PSReadLineKeyHandler -Chord $dxPreviousMenuKey"));
        assert!(pwsh.contains("Set-PSReadLineKeyHandler -Key 'Tab'"));
        assert!(pwsh.contains(
            "-BriefDescription 'dx menu' -Description $Global:__dx_pwsh_menu_handler_description"
        ));
        assert!(pwsh.contains("'MenuComplete' { [Microsoft.PowerShell.PSConsoleReadLine]::MenuComplete($key, $arg); return }"));
        assert!(pwsh.contains("$Global:__dx_pwsh_menu_previous_function -eq 'CustomAction'"));
        assert!(pwsh.contains("dx init: warning: PSReadLine key '$Global:__dx_pwsh_menu_key' was bound to a CustomAction"));
        assert!(pwsh.contains("Set-PSReadLineKeyHandler -Key 'Tab'"));
        assert!(pwsh.contains("-ScriptBlock"));
        assert!(pwsh.contains("$dxMappingSeeds = @()"));
        assert!(pwsh.contains("$Global:__dx_pwsh_menu_mapped = @{}"));
        assert!(pwsh.contains("$dxMapped = $Global:__dx_pwsh_menu_mapped"));
        assert!(pwsh.contains("if ($env:DX_MENU -eq '0' -or -not (Get-Command dx -ErrorAction SilentlyContinue) -or ($first -notin $dxCmds -and -not $dxMenuMode))"));
        assert!(pwsh.contains("if ($LASTEXITCODE -ne 0 -or -not $json)"));
        assert!(pwsh.contains("$result = $json | ConvertFrom-Json"));
        assert!(pwsh.contains("function global:__dx_pwsh_capture_redraw_context"));
        assert!(pwsh.contains("function global:__dx_pwsh_resolve_redraw_y"));
        assert!(pwsh.contains("function global:__dx_pwsh_invoke_prompt_at"));
        assert!(pwsh.contains("[Console]::Write(\"`e[0J\")"));
        assert!(pwsh.contains(
            "$expectedRedrawRow = [Math]::Max($Context.RelativeCursorY - $scrollRows, 0)"
        ));
        assert!(pwsh.contains("if ($result -and $result.action -eq 'cancel')"));
        assert!(
            pwsh.contains("[Microsoft.PowerShell.PSConsoleReadLine]::SetCursorPosition($cursor)")
        );
        assert!(pwsh.contains("if (-not $result -or $result.action -ne 'replace')"));
        assert!(pwsh.contains("if (-not $result.terminal -or ($result.terminal -ne 'clean' -and $result.terminal -ne 'dirty'))"));
        assert!(pwsh.contains("if ($result.terminal -eq 'dirty')"));
        assert!(pwsh.contains("if ($result.terminal -eq 'dirty' -and $null -eq $redrawY)"));
        assert!(pwsh.contains("__dx_pwsh_invoke_prompt_at -RedrawY ([int]$redrawY)"));
        assert!(pwsh.contains("$result.replaceEnd -gt $line.Length"));
        assert!(pwsh.contains("PSConsoleReadLine]::InvokePrompt()"));
        assert!(pwsh.contains("__dx_pwsh_menu_fallback $key $arg"));
        assert!(pwsh.contains("default { [Microsoft.PowerShell.PSConsoleReadLine]::TabCompleteNext($key, $arg); return }"));
    }

    #[test]
    fn pwsh_menu_mappings_expand_aliases_at_hook_load() {
        let mappings = parse_menu_command_mappings("Get-ChildItem=path").expect("valid mapping");
        let pwsh = generate_with_mappings(Shell::Pwsh, false, true, &mappings);

        assert!(pwsh.contains("$dxMappingSeeds = @('Get-ChildItem=path')"));
        assert!(pwsh.contains("$dxExplicitMapped = @{}"));
        assert!(pwsh.contains("$dxDerivedMapped = @{}"));
        assert!(pwsh.contains("$dxExplicitMapped[$parts[0]] = $parts[1]"));
        assert!(pwsh.contains(
            "foreach ($alias in Get-Alias -Definition $command -ErrorAction SilentlyContinue)"
        ));
        assert!(pwsh.contains("$dxDerivedMapped[$aliasName] = $mode"));
        assert!(pwsh.contains("$Global:__dx_pwsh_menu_mapped = @{}"));
        assert!(pwsh.contains("$dxMapped = $Global:__dx_pwsh_menu_mapped"));
        assert!(pwsh.contains("if ($dxMapped -and $dxMapped.ContainsKey($first))"));
        assert!(pwsh.contains("$dxMenuMode = $dxMapped[$first]"));
        assert!(!pwsh.contains("foreach ($entry in $dxMapped)"));
    }

    #[test]
    fn pwsh_menu_mapping_precedence_prefers_explicit_over_derived() {
        let mappings =
            parse_menu_command_mappings("Get-ChildItem=path,gci=file").expect("valid mappings");
        let pwsh = generate_with_mappings(Shell::Pwsh, false, true, &mappings);

        assert!(pwsh.contains("$dxMappingSeeds = @('Get-ChildItem=path', 'gci=file')"));
        assert!(pwsh.contains("if (-not $dxExplicitMapped.ContainsKey($aliasName) -and -not $dxDerivedMapped.ContainsKey($aliasName))"));
        let derived_pos = pwsh
            .find("foreach ($key in $dxDerivedMapped.Keys)")
            .expect("derived mappings should be copied");
        let explicit_pos = pwsh
            .find("foreach ($key in $dxExplicitMapped.Keys)")
            .expect("explicit mappings should be copied");
        assert!(
            derived_pos < explicit_pos,
            "explicit mappings should be copied last so they win"
        );
    }

    #[test]
    fn pwsh_menu_key_can_be_customized_in_generated_output() {
        let pwsh = generate_with_mappings_and_pwsh_key(Shell::Pwsh, false, true, &[], "F12");
        assert!(pwsh.contains("$dxNewMenuKey = 'F12'"));
        assert!(pwsh.contains("$Global:__dx_pwsh_menu_key = $dxNewMenuKey"));
        assert!(pwsh.contains("Set-PSReadLineKeyHandler -Key 'F12'"));
        assert!(pwsh.contains("-ScriptBlock"));
    }

    #[test]
    fn pwsh_menu_key_is_ignored_by_non_pwsh_shells() {
        let bash = generate_with_mappings_and_pwsh_key(Shell::Bash, false, true, &[], "F12");
        assert!(!bash.contains("F12"));
    }

    #[test]
    fn all_shells_freeze_command_not_found_guard_contract_markers() {
        let bash = generate(Shell::Bash, true, false);
        assert!(bash.contains("command_not_found_handle()"));
        assert!(bash.contains("if [[ -n \"${DX_RESOLVE_GUARD:-}\" ]]; then"));
        assert!(bash.contains("if ! __dx_is_path_like \"$__dx_cmd\"; then"));
        assert!(bash.contains("\"$__dx_cmd\" == *-*"));
        assert!(bash.contains("\"$__dx_cmd\" == *..*"));
        assert!(bash.contains(
            "__dx_resolved=\"$(DX_RESOLVE_GUARD=1 dx resolve \"$__dx_cmd\" 2>/dev/null)\""
        ));

        let zsh = generate(Shell::Zsh, true, false);
        assert!(zsh.contains("command_not_found_handler()"));
        assert!(zsh.contains("if [[ -n \"${DX_RESOLVE_GUARD:-}\" ]]; then"));
        assert!(zsh.contains("if ! __dx_is_path_like \"$__dx_cmd\"; then"));
        assert!(zsh.contains("\"$__dx_cmd\" == *-*"));
        assert!(zsh.contains("\"$__dx_cmd\" == *..*"));
        assert!(zsh.contains(
            "__dx_resolved=\"$(DX_RESOLVE_GUARD=1 dx resolve \"$__dx_cmd\" 2>/dev/null)\""
        ));

        let fish = generate(Shell::Fish, true, false);
        assert!(fish.contains("function fish_command_not_found --argument __dx_cmd"));
        assert!(fish.contains("if set -q DX_RESOLVE_GUARD"));
        assert!(fish.contains("if not __dx_is_path_like \"$__dx_cmd\""));
        assert!(fish.contains(".*-|.*_|.*\\.\\..*"));
        assert!(fish.contains("set -lx DX_RESOLVE_GUARD 1"));
        assert!(fish.contains("set -l __dx_resolved (dx resolve \"$__dx_cmd\" 2>/dev/null)"));
        assert!(fish.contains("set -e DX_RESOLVE_GUARD"));

        let pwsh = generate(Shell::Pwsh, true, false);
        assert!(pwsh.contains("CommandNotFoundAction"));
        assert!(pwsh.contains("if ($env:DX_RESOLVE_GUARD) { return }"));
        assert!(pwsh.contains("if (-not (__dx_is_path_like $cmd)) { return }"));
        assert!(pwsh.contains("-match '(/|^\\.|^~|^\\.{3,}$|-|_|\\.\\.)'"));
        assert!(pwsh.contains("$env:DX_RESOLVE_GUARD = '1'"));
        assert!(pwsh.contains("$resolved = (dx resolve $cmd 2>$null)"));
        assert!(pwsh.contains("Remove-Item Env:DX_RESOLVE_GUARD -ErrorAction SilentlyContinue"));
    }

    #[test]
    fn bash_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Bash, true, false));
    }

    #[test]
    fn zsh_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Zsh, true, false));
    }

    #[test]
    fn fish_script_has_balanced_braces_and_quotes() {
        assert_balanced_braces(&generate(Shell::Fish, true, false));
    }

    #[test]
    fn pwsh_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Pwsh, true, false));
    }

    #[test]
    fn bash_menu_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Bash, true, true));
    }

    #[test]
    fn zsh_menu_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Zsh, true, true));
    }

    #[test]
    fn fish_menu_script_has_balanced_braces_and_quotes() {
        assert_balanced_braces(&generate(Shell::Fish, true, true));
    }

    #[test]
    fn pwsh_menu_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Pwsh, true, true));
    }

    #[test]
    fn generated_scripts_do_not_leak_internal_placeholder_tokens() {
        let shells = [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Pwsh];
        for shell in shells {
            for command_not_found in [false, true] {
                for menu in [false, true] {
                    let script = generate(shell, command_not_found, menu);
                    assert_no_unresolved_internal_placeholders(&script);
                }
            }
        }
    }

    #[test]
    fn menu_enabled_scripts_keep_cross_shell_menu_invocation_marker() {
        let shells = [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Pwsh];
        for shell in shells {
            let script = generate(shell, false, true);
            assert!(
                script.contains("dx menu --shell"),
                "missing menu invocation marker for {shell:?}"
            );
            assert_no_unresolved_internal_placeholders(&script);
        }
    }

    #[test]
    fn zsh_menu_parses_terminal_and_conditionally_resets_prompt() {
        let script = generate(Shell::Zsh, false, true);
        assert!(script.contains(r#"__dx_terminal_marker="\"terminal\":\"""#));
        assert!(script.contains("__dx_terminal"));
        assert!(
            script.contains(r#"[[ "$__dx_terminal" == "clean" || "$__dx_terminal" == "dirty" ]]"#)
        );
        assert!(script.contains(r#"[[ "$__dx_terminal" == "dirty" ]] && zle reset-prompt"#));
    }

    #[test]
    fn fish_menu_parses_terminal_and_conditionally_repaints() {
        let script = generate(Shell::Fish, false, true);
        assert!(script.contains(r#"\"terminal\":\""#));
        assert!(script.contains("terminal"));
        assert!(script.contains(r#"test "$terminal" != "clean" -a "$terminal" != "dirty""#));
        assert!(script.contains(r#"if test "$terminal" = "dirty""#));
        assert!(
            script.contains("commandline -f repaint"),
            "repaint should still be present"
        );
    }

    #[test]
    fn pwsh_menu_checks_terminal_field_and_conditionally_invokes_prompt() {
        let script = generate(Shell::Pwsh, false, true);
        assert!(script.contains(r#"$result.terminal"#));
        assert!(script.contains(r#"-ne 'clean' -and $result.terminal -ne 'dirty'"#));
        assert!(script.contains(r#"if ($result.terminal -eq 'dirty')"#));
        assert!(script.contains("InvokePrompt()"));
    }
}
