//! Fish hook generation.

use super::MenuCommandMapping;
use super::common::{
    MENU_ELIGIBLE_COMMANDS, apply_template_replacements, fish_case_words,
    render_fish_completion_bindings, render_fish_menu_mapping_cases,
};
use crate::{cli, hooks::Shell};

pub fn generate(command_not_found: bool, menu: bool, mappings: &[MenuCommandMapping]) -> String {
    let mut script = String::from(include_str!("templates/fish/base.fish"));

    if menu {
        script.push_str(include_str!("templates/fish/menu.fish"));
    }

    if command_not_found {
        script.push_str(include_str!("templates/fish/command-not-found.fish"));
    }

    apply_template_replacements(
        script,
        [
            (
                "__DX_CLAP_COMPLETION__",
                cli::completion_script(Shell::Fish),
            ),
            (
                "__DX_FISH_COMPLETION_BINDINGS__",
                render_fish_completion_bindings(),
            ),
            (
                "__DX_FISH_MENU_CASE_WORDS__",
                fish_case_words(MENU_ELIGIBLE_COMMANDS),
            ),
            (
                "__DX_FISH_MENU_MAPPING_CASES__",
                render_fish_menu_mapping_cases(mappings),
            ),
        ],
    )
}
