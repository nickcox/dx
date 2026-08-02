//! Fish hook generation.

use super::MenuCommandMapping;
use super::common::{
    apply_template_replacements, fish_case_words, menu_eligible_commands,
    render_fish_completion_bindings, render_fish_frecency_wrappers, render_fish_menu_mapping_cases,
};

pub fn generate(
    command_not_found: bool,
    menu: bool,
    mappings: &[MenuCommandMapping],
    clap_completion: &str,
    frecency: bool,
) -> String {
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
            ("__DX_CLAP_COMPLETION__", clap_completion.to_string()),
            (
                "__DX_FISH_FRECENCY_WRAPPERS__",
                render_fish_frecency_wrappers(frecency),
            ),
            (
                "__DX_FISH_COMPLETION_BINDINGS__",
                render_fish_completion_bindings(frecency),
            ),
            (
                "__DX_FISH_MENU_CASE_WORDS__",
                fish_case_words(&menu_eligible_commands(frecency)),
            ),
            (
                "__DX_FISH_MENU_MAPPING_CASES__",
                render_fish_menu_mapping_cases(mappings),
            ),
        ],
    )
}
