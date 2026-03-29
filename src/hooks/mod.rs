mod bash;
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

pub fn generate(shell: Shell, command_not_found: bool) -> String {
    match shell {
        Shell::Bash => bash::generate(command_not_found),
        Shell::Zsh => zsh::generate(command_not_found),
        Shell::Fish => fish::generate(command_not_found),
        Shell::Pwsh => pwsh::generate(command_not_found),
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

    #[test]
    fn generate_bash_without_command_not_found_contains_cd_only() {
        let output = generate(Shell::Bash, false);
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
        let output = generate(Shell::Bash, false);
        // back/forward should use __dx_stack_wrapper (dx undo/redo), not __dx_nav_wrapper (dx push)
        assert!(output.contains("back() {\n  __dx_stack_wrapper back"));
        assert!(output.contains("forward() {\n  __dx_stack_wrapper forward"));
        assert!(output.contains("__dx_stack_wrapper()"));
        assert!(output.contains("dx \"$__dx_undo_or_redo\""));
    }

    #[test]
    fn bash_up_seeds_origin_before_navigate() {
        let output = generate(Shell::Bash, false);
        // __dx_nav_wrapper should call __dx_push_pwd before dx navigate
        let nav_wrapper_start = output.find("__dx_nav_wrapper()").unwrap();
        let nav_section = &output[nav_wrapper_start..];
        let push_pos = nav_section.find("__dx_push_pwd").unwrap();
        let navigate_pos = nav_section.find("dx navigate").unwrap();
        assert!(
            push_pos < navigate_pos,
            "push_pwd should come before dx navigate in nav_wrapper"
        );
    }

    #[test]
    fn zsh_back_forward_use_stack_wrapper() {
        let output = generate(Shell::Zsh, false);
        assert!(output.contains("back() {\n  __dx_stack_wrapper back"));
        assert!(output.contains("forward() {\n  __dx_stack_wrapper forward"));
        assert!(output.contains("__dx_stack_wrapper()"));
    }

    #[test]
    fn fish_back_forward_use_stack_wrapper() {
        let output = generate(Shell::Fish, false);
        assert!(output.contains("function back\n  __dx_stack_wrapper back"));
        assert!(output.contains("function forward\n  __dx_stack_wrapper forward"));
        assert!(output.contains("function __dx_stack_wrapper"));
    }

    #[test]
    fn generate_bash_with_command_not_found_contains_handler() {
        let output = generate(Shell::Bash, true);
        assert!(output.contains("cd()"));
        assert!(output.contains("DX_SESSION"));
        assert!(output.contains("command_not_found_handle"));
    }

    #[test]
    fn generate_zsh_uses_handler_suffix() {
        let output = generate(Shell::Zsh, true);
        assert!(output.contains("command_not_found_handler"));
        assert!(output.contains("compdef _dx_complete_dx dx"));
        assert!(output.contains("compdef _dx_complete_ancestors up"));
        assert!(!output.contains("command_not_found_handle()"));
    }

    #[test]
    fn generate_fish_without_command_not_found_excludes_handler() {
        let output = generate(Shell::Fish, false);
        assert!(output.contains("function cd"));
        assert!(output.contains("function up"));
        assert!(output.contains("complete -c dx"));
        assert!(output.contains("DX_SESSION"));
        assert!(!output.contains("fish_command_not_found"));
    }

    #[test]
    fn generate_pwsh_with_command_not_found_includes_guard_and_action() {
        let output = generate(Shell::Pwsh, true);
        assert!(output.contains("Set-Location"));
        assert!(output.contains("function up"));
        assert!(output.contains("Register-ArgumentCompleter -CommandName dx"));
        assert!(output.contains("CommandNotFoundAction"));
        assert!(output.contains("DX_RESOLVE_GUARD"));
    }

    #[test]
    fn generate_pwsh_without_command_not_found_excludes_action() {
        let output = generate(Shell::Pwsh, false);
        assert!(output.contains("Set-Location"));
        assert!(!output.contains("CommandNotFoundAction"));
    }

    #[test]
    fn generate_all_shells_guard_existing_dx_session() {
        let bash = generate(Shell::Bash, false);
        let zsh = generate(Shell::Zsh, false);
        let fish = generate(Shell::Fish, false);
        let pwsh = generate(Shell::Pwsh, false);

        assert!(bash.contains("DX_SESSION:-"));
        assert!(zsh.contains("DX_SESSION:-"));
        assert!(fish.contains("if not set -q DX_SESSION"));
        assert!(pwsh.contains("if (-not $env:DX_SESSION)"));
    }

    #[test]
    fn bash_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Bash, true));
    }

    #[test]
    fn zsh_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Zsh, true));
    }

    #[test]
    fn fish_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Fish, true));
    }

    #[test]
    fn pwsh_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Pwsh, true));
    }
}
