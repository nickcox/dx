//! Zsh hook generation.

use super::MenuCommandMapping;
use super::common::{
    apply_template_replacements, render_posix_menu_eligible_case_pattern,
    render_posix_wrapper_declarations, render_zsh_completion_bindings,
    render_zsh_completion_functions, render_zsh_menu_mapping_case,
};

pub fn generate(
    command_not_found: bool,
    menu: bool,
    mappings: &[MenuCommandMapping],
    clap_completion: &str,
) -> String {
    let mut script = String::from(include_str!("templates/zsh/base.zsh"));

    if menu {
        script.push_str(include_str!("templates/zsh/menu.zsh"));
    }

    if command_not_found {
        script.push_str(include_str!("templates/zsh/command-not-found.zsh"));
    }

    apply_template_replacements(
        script,
        [
            ("__DX_CLAP_COMPLETION__", clap_completion.to_string()),
            (
                "__DX_ZSH_COMPLETION_BINDINGS__",
                render_zsh_completion_bindings(),
            ),
            (
                "__DX_ZSH_COMPLETION_FUNCTIONS__",
                render_zsh_completion_functions(),
            ),
            (
                "__DX_POSIX_WRAPPER_DECLARATIONS__",
                render_posix_wrapper_declarations(),
            ),
            (
                "__DX_ZSH_MENU_CASE__",
                render_posix_menu_eligible_case_pattern(),
            ),
            (
                "__DX_ZSH_MENU_MAPPING_CASE__",
                render_zsh_menu_mapping_case(mappings),
            ),
        ],
    )
}
