mod bash;
mod common;
mod fish;
mod pwsh;
mod zsh;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Pwsh,
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
    match shell {
        Shell::Bash => bash::generate(command_not_found, menu),
        Shell::Zsh => zsh::generate(command_not_found, menu),
        Shell::Fish => fish::generate(command_not_found, menu),
        Shell::Pwsh => pwsh::generate(command_not_found, menu),
    }
}

#[cfg(test)]
mod tests {
    use super::{generate, Shell};

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

        let double_quotes = count_unescaped(script, '"');
        assert_eq!(double_quotes % 2, 0, "unbalanced double quotes");

        let single_quotes = count_unescaped(script, '\'');
        assert_eq!(single_quotes % 2, 0, "unbalanced single quotes");
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
        assert!(output.contains("_dx_complete_dx dx"));
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
        assert!(output.contains("dx stack \"$__dx_undo_or_redo\""));
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
        assert!(output.contains("compdef _dx_complete_dx dx"));
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
        assert!(output.contains("function up"));
        assert!(output.contains("Register-ArgumentCompleter -CommandName dx"));
        assert!(output.contains("CommandNotFoundAction"));
        assert!(output.contains("DX_RESOLVE_GUARD"));
    }

    #[test]
    fn generate_pwsh_without_command_not_found_excludes_action() {
        let output = generate(Shell::Pwsh, false, false);
        assert!(output.contains("Set-Location"));
        assert!(!output.contains("CommandNotFoundAction"));
    }

    #[test]
    fn pwsh_removes_cd_alias_before_wrapper_definition() {
        let output = generate(Shell::Pwsh, false, false);
        assert!(output.contains("Remove-Item Alias:cd -ErrorAction SilentlyContinue"));
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
        assert!(bash.contains("resolve)\n      _dx_complete_paths"));
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
        assert!(zsh.contains("resolve)\n      _dx_complete_paths"));
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
        assert!(
            fish.contains("complete -c cd -a '(dx complete paths (commandline -ct) 2>/dev/null)'")
        );
        assert!(fish.contains("complete -c back -a '(dx complete stack --direction back (commandline -ct) 2>/dev/null)'"));
        assert!(fish.contains("complete -c cd+ -a '(dx complete stack --direction forward (commandline -ct) 2>/dev/null)'"));

        let pwsh = generate(Shell::Pwsh, false, false);
        assert!(pwsh.contains("function cd"));
        assert!(pwsh.contains("function up"));
        assert!(pwsh.contains("function cdf"));
        assert!(pwsh.contains("Set-Alias -Name z -Value cdf -Scope Global"));
        assert!(pwsh.contains("function cdr"));
        assert!(pwsh.contains("function back"));
        assert!(pwsh.contains("Set-Alias -Name 'cd-' -Value back -Scope Global"));
        assert!(pwsh.contains("function forward"));
        assert!(pwsh.contains("Set-Alias -Name 'cd+' -Value forward -Scope Global"));
        assert!(pwsh.contains("$resolved = (dx resolve $pathArg 2>$null)"));
        assert!(pwsh.contains("__dx_complete_mode -Mode paths -Word $wordToComplete"));
        assert!(pwsh.contains("'resolve' {\n            __dx_emit_completion (__dx_complete_mode -Mode paths -Word $wordToComplete)"));
        assert!(
            pwsh.contains("Register-ArgumentCompleter -CommandName cd,Set-Location -ScriptBlock")
        );
        assert!(pwsh.contains("Register-ArgumentCompleter -CommandName back,cd- -ScriptBlock"));
        assert!(pwsh.contains("Register-ArgumentCompleter -CommandName forward,cd+ -ScriptBlock"));
    }

    #[test]
    fn all_shells_freeze_menu_fallback_contract_markers() {
        let bash = generate(Shell::Bash, false, true);
        assert!(bash.contains("__dx_json=\"$(dx menu --buffer \"$COMP_LINE\" --cursor \"$COMP_POINT\" --cwd \"$PWD\" --session \"${DX_SESSION:-}\" </dev/tty 2>/dev/tty)\" || return 1"));
        assert!(bash.contains("[[ \"$__dx_action\" == \"replace\" ]] || return 1"));
        assert!(bash.contains("(( __dx_re >= __dx_rs )) || return 1"));
        assert!(bash.contains("if __dx_try_menu; then\n    return 0\n  fi"));
        assert!(bash.contains("case \"$__dx_cmd\" in"));

        let zsh = generate(Shell::Zsh, false, true);
        assert!(
            zsh.contains("if [[ $__dx_exit -ne 0 ]]; then\n    zle expand-or-complete\n    return")
        );
        assert!(zsh.contains(
            "[[ \"$__dx_action\" == \"replace\" ]] || { zle expand-or-complete; return }"
        ));
        assert!(zsh.contains("(( __dx_re >= __dx_rs )) || { zle expand-or-complete; return }"));
        assert!(zsh.contains("(( __dx_closed )) || { zle expand-or-complete; return }"));
        assert!(zsh.contains("[[ -n \"$__dx_value\" ]] || { zle expand-or-complete; return }"));

        let fish = generate(Shell::Fish, false, true);
        assert!(fish.contains("set -l json (dx menu --buffer \"$buf\" --cursor $cur --cwd \"$PWD\" --session \"$DX_SESSION\" </dev/tty 2>/dev/tty)"));
        assert!(fish.contains("if test $status -ne 0\n    commandline -f complete\n    return"));
        assert!(fish.contains("if not string match -q '*\"action\":\"replace\"*' -- \"$json\""));
        assert!(fish.contains("if test \"$action\" != \"replace\""));
        assert!(fish.contains("if test (count $value_match) -lt 2"));
        assert!(fish.contains("if test $re -lt $rs"));
        assert!(fish.contains("if test $rs -gt $buflen; or test $re -gt $buflen"));

        let pwsh = generate(Shell::Pwsh, false, true);
        assert!(pwsh.contains("if ($env:DX_MENU -eq '0' -or -not (Get-Command dx -ErrorAction SilentlyContinue) -or $first -notin $dxCmds)"));
        assert!(pwsh.contains("if ($LASTEXITCODE -ne 0 -or -not $json)"));
        assert!(pwsh.contains("$result = $json | ConvertFrom-Json"));
        assert!(pwsh.contains("if (-not $result -or $result.action -ne 'replace')"));
        assert!(
            pwsh.contains("[Microsoft.PowerShell.PSConsoleReadLine]::TabCompleteNext($key, $arg)")
        );
    }

    #[test]
    fn all_shells_freeze_command_not_found_guard_contract_markers() {
        let bash = generate(Shell::Bash, true, false);
        assert!(bash.contains("command_not_found_handle()"));
        assert!(bash.contains("if [[ -n \"${DX_RESOLVE_GUARD:-}\" ]]; then"));
        assert!(bash.contains("if ! __dx_is_path_like \"$__dx_cmd\"; then"));
        assert!(bash.contains(
            "__dx_resolved=\"$(DX_RESOLVE_GUARD=1 dx resolve \"$__dx_cmd\" 2>/dev/null)\""
        ));

        let zsh = generate(Shell::Zsh, true, false);
        assert!(zsh.contains("command_not_found_handler()"));
        assert!(zsh.contains("if [[ -n \"${DX_RESOLVE_GUARD:-}\" ]]; then"));
        assert!(zsh.contains("if ! __dx_is_path_like \"$__dx_cmd\"; then"));
        assert!(zsh.contains(
            "__dx_resolved=\"$(DX_RESOLVE_GUARD=1 dx resolve \"$__dx_cmd\" 2>/dev/null)\""
        ));

        let fish = generate(Shell::Fish, true, false);
        assert!(fish.contains("function fish_command_not_found --argument __dx_cmd"));
        assert!(fish.contains("if set -q DX_RESOLVE_GUARD"));
        assert!(fish.contains("if not __dx_is_path_like \"$__dx_cmd\""));
        assert!(fish.contains("set -lx DX_RESOLVE_GUARD 1"));
        assert!(fish.contains("set -l __dx_resolved (dx resolve \"$__dx_cmd\" 2>/dev/null)"));
        assert!(fish.contains("set -e DX_RESOLVE_GUARD"));

        let pwsh = generate(Shell::Pwsh, true, false);
        assert!(pwsh.contains("CommandNotFoundAction"));
        assert!(pwsh.contains("if ($env:DX_RESOLVE_GUARD) { return }"));
        assert!(pwsh.contains("if (-not (__dx_is_path_like $cmd)) { return }"));
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
        assert_balanced_delimiters(&generate(Shell::Fish, true, false));
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
        assert_balanced_delimiters(&generate(Shell::Fish, true, true));
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
                script.contains("dx menu --buffer"),
                "missing menu invocation marker for {shell:?}"
            );
            assert_no_unresolved_internal_placeholders(&script);
        }
    }
}
