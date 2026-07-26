//! Bash hook generation.

use super::MenuCommandMapping;
use super::common::{
    apply_template_replacements, render_bash_completion_bindings, render_bash_completion_functions,
    render_bash_menu_fallback_case, render_bash_menu_mapping_case,
    render_posix_wrapper_declarations,
};
use crate::{cli, hooks::Shell};

pub fn generate(command_not_found: bool, menu: bool, mappings: &[MenuCommandMapping]) -> String {
    let mut script = String::from(include_str!("templates/bash/base.sh"));

    if menu {
        script.push_str(include_str!("templates/bash/menu.sh"));
    }

    if command_not_found {
        script.push_str(include_str!("templates/bash/command-not-found.sh"));
    }

    apply_template_replacements(
        script,
        [
            (
                "__DX_CLAP_COMPLETION__",
                cli::completion_script(Shell::Bash),
            ),
            (
                "__DX_BASH_COMPLETION_BINDINGS__",
                render_bash_completion_bindings(),
            ),
            (
                "__DX_BASH_COMPLETION_FUNCTIONS__",
                render_bash_completion_functions(),
            ),
            (
                "__DX_POSIX_WRAPPER_DECLARATIONS__",
                render_posix_wrapper_declarations(),
            ),
            (
                "__DX_BASH_MENU_FALLBACK_CASE__",
                render_bash_menu_fallback_case(),
            ),
            (
                "__DX_BASH_MENU_MAPPING_CASE__",
                render_bash_menu_mapping_case(mappings),
            ),
            (
                "__DX_BASH_MAPPED_MENU_BINDINGS__",
                mappings
                    .iter()
                    .map(|mapping| format!("complete -F _dx_menu_wrapper {}", mapping.command()))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        ],
    )
}
