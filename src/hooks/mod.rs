mod bash;
mod common;
mod fish;
mod mappings;
mod pwsh;
mod zsh;

use clap::ValueEnum;

pub use mappings::{MenuCommandMapping, MenuCommandMappingError, parse_menu_command_mappings};
pub use pwsh::{PwshMenuKeyError, parse_pwsh_menu_key};

/// The shells `dx` can generate hooks for. Doubles as the `dx init <SHELL>` and
/// `dx menu --shell` argument type, so the accepted spellings and the hook
/// dispatch table can never drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
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

/// Everything that varies between generated hook scripts.
#[derive(Debug, Clone)]
pub struct HookOptions {
    pub command_not_found: bool,
    pub menu_mode: InitMenuMode,
    pub mappings: Vec<MenuCommandMapping>,
    /// PSReadLine chord that opens the menu. Ignored by every other shell.
    pub pwsh_menu_key: String,
}

impl Default for HookOptions {
    fn default() -> Self {
        Self {
            command_not_found: false,
            menu_mode: InitMenuMode::Disabled,
            mappings: Vec::new(),
            pwsh_menu_key: DEFAULT_PWSH_MENU_KEY.to_string(),
        }
    }
}

pub const DEFAULT_PWSH_MENU_KEY: &str = "Tab";

pub fn generate(shell: Shell, options: &HookOptions) -> String {
    let menu = options.menu_mode == InitMenuMode::Tui;
    match shell {
        Shell::Bash => bash::generate(options.command_not_found, menu, &options.mappings),
        Shell::Zsh => zsh::generate(options.command_not_found, menu, &options.mappings),
        Shell::Fish => fish::generate(options.command_not_found, menu, &options.mappings),
        Shell::Pwsh => pwsh::generate(
            options.command_not_found,
            options.menu_mode,
            &options.mappings,
            &options.pwsh_menu_key,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        HookOptions, InitMenuMode, MenuCommandMapping, Shell, generate,
        parse_menu_command_mappings, parse_pwsh_menu_key,
    };

    /// The tests enumerate the (command_not_found x menu) matrix, so a helper
    /// keeps that shape readable without reviving the old boolean-pair API.
    fn options(command_not_found: bool, menu: bool) -> HookOptions {
        HookOptions {
            command_not_found,
            menu_mode: if menu {
                InitMenuMode::Tui
            } else {
                InitMenuMode::Disabled
            },
            ..HookOptions::default()
        }
    }

    fn menu_options(mappings: Vec<MenuCommandMapping>, pwsh_menu_key: &str) -> HookOptions {
        HookOptions {
            menu_mode: InitMenuMode::Tui,
            mappings,
            pwsh_menu_key: pwsh_menu_key.to_string(),
            ..HookOptions::default()
        }
    }

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
    fn bash_up_seeds_origin_before_navigate() {
        let output = generate(Shell::Bash, &options(false, false));
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
        let bash = generate(Shell::Bash, &options(false, false));
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

        let zsh = generate(Shell::Zsh, &options(false, false));
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

        let fish = generate(Shell::Fish, &options(false, false));
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
        let output = generate(Shell::Pwsh, &options(false, false));

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
        let bash = generate(Shell::Bash, &options(false, false));
        let bash_stack = section_between(&bash, "__dx_stack_wrapper()", "\n__dx_jump_mode()");
        assert!(!bash_stack.contains("__dx_push_pwd"));
        assert!(bash_stack.contains("--preview"));
        assert!(bash_stack.contains("--target \"$__dx_dest\" >/dev/null"));

        let zsh = generate(Shell::Zsh, &options(false, false));
        let zsh_stack = section_between(&zsh, "__dx_stack_wrapper()", "\n__dx_jump_mode()");
        assert!(!zsh_stack.contains("__dx_push_pwd"));
        assert!(zsh_stack.contains("--preview"));
        assert!(zsh_stack.contains("--target \"$__dx_dest\" >/dev/null"));

        let fish = generate(Shell::Fish, &options(false, false));
        let fish_stack = section_between(
            &fish,
            "function __dx_stack_wrapper",
            "\nfunction __dx_jump_mode",
        );
        assert!(!fish_stack.contains("__dx_push_pwd"));
        assert!(fish_stack.contains("--preview"));
        assert!(fish_stack.contains("--target \"$dest\" >/dev/null"));

        let pwsh = generate(Shell::Pwsh, &options(false, false));
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
    fn pwsh_menu_mapping_precedence_prefers_explicit_over_derived() {
        let mappings =
            parse_menu_command_mappings("Get-ChildItem=path,gci=file").expect("valid mappings");
        let pwsh = generate(Shell::Pwsh, &menu_options(mappings.clone(), "Tab"));

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
    fn pwsh_menu_key_is_ignored_by_non_pwsh_shells() {
        let bash = generate(Shell::Bash, &menu_options(Vec::new(), "F12"));
        assert!(!bash.contains("F12"));
    }

    #[test]
    fn bash_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Bash, &options(true, false)));
    }

    #[test]
    fn zsh_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Zsh, &options(true, false)));
    }

    #[test]
    fn fish_script_has_balanced_braces_and_quotes() {
        assert_balanced_braces(&generate(Shell::Fish, &options(true, false)));
    }

    #[test]
    fn pwsh_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Pwsh, &options(true, false)));
    }

    #[test]
    fn bash_menu_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Bash, &options(true, true)));
    }

    #[test]
    fn zsh_menu_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Zsh, &options(true, true)));
    }

    #[test]
    fn fish_menu_script_has_balanced_braces_and_quotes() {
        assert_balanced_braces(&generate(Shell::Fish, &options(true, true)));
    }

    #[test]
    fn pwsh_menu_script_has_balanced_braces_and_quotes() {
        assert_balanced_delimiters(&generate(Shell::Pwsh, &options(true, true)));
    }

    #[test]
    fn generated_scripts_do_not_leak_internal_placeholder_tokens() {
        let shells = [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Pwsh];
        for shell in shells {
            for command_not_found in [false, true] {
                for menu in [false, true] {
                    let script = generate(shell, &options(command_not_found, menu));
                    assert_no_unresolved_internal_placeholders(&script);
                }
            }
        }
    }

    #[test]
    fn menu_enabled_scripts_keep_cross_shell_menu_invocation_marker() {
        let shells = [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Pwsh];
        for shell in shells {
            let script = generate(shell, &options(false, true));
            assert!(
                script.contains("dx menu --shell"),
                "missing menu invocation marker for {shell:?}"
            );
            assert_no_unresolved_internal_placeholders(&script);
        }
    }

    /// The blocks each `HookOptions` combination is expected to select.
    ///
    /// Each marker is a line unique to one optional template, so asserting on
    /// its presence checks block selection directly — cheaply, and for every
    /// combination, without storing a golden per combination.
    const BLOCK_MARKERS: &[(Shell, &str, &str)] = &[
        (Shell::Bash, "menu", "__dx_try_menu() {"),
        (
            Shell::Bash,
            "command-not-found",
            "command_not_found_handle() {",
        ),
        (Shell::Zsh, "menu", "__dx_menu_widget() {"),
        (
            Shell::Zsh,
            "command-not-found",
            "command_not_found_handler() {",
        ),
        (Shell::Fish, "menu", "function __dx_menu_complete"),
        (
            Shell::Fish,
            "command-not-found",
            "function fish_command_not_found",
        ),
        (
            Shell::Pwsh,
            "menu",
            "function global:__dx_pwsh_menu_fallback",
        ),
        (
            Shell::Pwsh,
            "command-not-found",
            "if ($ExecutionContext.InvokeCommand.PSObject.Properties.Name -contains 'CommandNotFoundAction') {",
        ),
    ];

    #[test]
    fn options_select_the_expected_template_blocks() {
        for &(shell, block, marker) in BLOCK_MARKERS {
            for command_not_found in [false, true] {
                for menu in [false, true] {
                    let script = generate(shell, &options(command_not_found, menu));
                    let expected = match block {
                        "menu" => menu,
                        "command-not-found" => command_not_found,
                        other => panic!("unhandled block {other}"),
                    };
                    assert_eq!(
                        script.contains(marker),
                        expected,
                        "{shell:?} {block} block: expected present={expected} \
                         for command_not_found={command_not_found} menu={menu}"
                    );
                }
            }
        }
    }

    #[test]
    fn native_menu_replaces_the_psreadline_menu_block() {
        let native = generate(
            Shell::Pwsh,
            &HookOptions {
                menu_mode: InitMenuMode::NativePwsh,
                ..HookOptions::default()
            },
        );

        assert!(native.contains("function __dx_unquote_completion_word"));
        assert!(!native.contains("function global:__dx_pwsh_menu_fallback"));
    }

    /// Full-text goldens for the maximal option set of each shell.
    ///
    /// The templates are themselves checked in, so a golden per combination
    /// would repeat one template edit as an identical diff four or five times.
    /// Pinning the maximal script keeps the bytes that ship under review, while
    /// `options_select_the_expected_template_blocks` covers the smaller
    /// combinations.
    fn golden_cases() -> Vec<(&'static str, String)> {
        let maximal = options(true, true);
        vec![
            (
                "bash-menu-command-not-found.sh",
                generate(Shell::Bash, &maximal),
            ),
            (
                "zsh-menu-command-not-found.zsh",
                generate(Shell::Zsh, &maximal),
            ),
            (
                "fish-menu-command-not-found.fish",
                generate(Shell::Fish, &maximal),
            ),
            (
                "pwsh-menu-command-not-found.ps1",
                generate(Shell::Pwsh, &maximal),
            ),
            (
                "pwsh-native-menu-command-not-found.ps1",
                generate(
                    Shell::Pwsh,
                    &HookOptions {
                        command_not_found: true,
                        menu_mode: InitMenuMode::NativePwsh,
                        ..HookOptions::default()
                    },
                ),
            ),
        ]
    }

    /// Compares one generated script against its checked-in golden.
    ///
    /// Mismatches are collected rather than panicking, so a single run reports
    /// every difference instead of stopping at the first.
    fn check_golden(name: &str, actual: &str, mismatches: &mut Vec<String>) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/hooks/goldens")
            .join(name);

        if std::env::var_os("DX_UPDATE_GOLDENS").is_some() {
            std::fs::write(&path, actual).expect("write golden script");
            return;
        }

        match std::fs::read_to_string(&path) {
            Ok(expected) if expected == actual => {}
            Ok(_) => mismatches.push(name.to_owned()),
            Err(error) => mismatches.push(format!("{name} ({error})")),
        }
    }

    #[test]
    fn generated_scripts_match_goldens() {
        let mut mismatches = Vec::new();
        for (name, script) in golden_cases() {
            check_golden(name, &script, &mut mismatches);
        }

        assert!(
            mismatches.is_empty(),
            "generated scripts differ from src/hooks/goldens: {}\n\
             regenerate with `DX_UPDATE_GOLDENS=1 cargo test` and review with `git diff`",
            mismatches.join(", "),
        );
    }
}
