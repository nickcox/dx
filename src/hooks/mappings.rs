use std::collections::HashSet;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuMappingMode {
    Path,
    Directory,
    File,
}

impl MenuMappingMode {
    pub fn as_cli_arg(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Directory => "directory",
            Self::File => "file",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "path" => Some(Self::Path),
            "directory" => Some(Self::Directory),
            "file" => Some(Self::File),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuCommandMapping {
    pub command: String,
    pub mode: MenuMappingMode,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MenuCommandMappingError {
    #[error("invalid mapping entry '{0}' (expected <command>=<mode>)")]
    InvalidEntry(String),
    #[error("mapping command cannot be empty in entry '{0}'")]
    EmptyCommand(String),
    #[error("invalid mapping mode '{mode}' in entry '{entry}' (expected path, directory, or file)")]
    InvalidMode { entry: String, mode: String },
    #[error("duplicate mapping for command '{0}'")]
    DuplicateCommand(String),
}

pub fn parse_menu_command_mappings(
    raw: &str,
) -> Result<Vec<MenuCommandMapping>, MenuCommandMappingError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut seen = HashSet::new();
    let mut mappings = Vec::new();

    for entry in trimmed.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            return Err(MenuCommandMappingError::InvalidEntry(entry.to_string()));
        }

        let Some((command, mode)) = entry.split_once('=') else {
            return Err(MenuCommandMappingError::InvalidEntry(entry.to_string()));
        };

        let command = command.trim();
        if command.is_empty() {
            return Err(MenuCommandMappingError::EmptyCommand(entry.to_string()));
        }

        let Some(mode) = MenuMappingMode::parse(mode) else {
            return Err(MenuCommandMappingError::InvalidMode {
                entry: entry.to_string(),
                mode: mode.trim().to_string(),
            });
        };

        if !seen.insert(command.to_string()) {
            return Err(MenuCommandMappingError::DuplicateCommand(
                command.to_string(),
            ));
        }

        mappings.push(MenuCommandMapping {
            command: command.to_string(),
            mode,
        });
    }

    Ok(mappings)
}

#[cfg(test)]
mod tests {
    use super::{
        MenuCommandMapping, MenuCommandMappingError, MenuMappingMode, parse_menu_command_mappings,
    };

    #[test]
    fn parses_valid_menu_mappings() {
        let parsed = parse_menu_command_mappings("ls=path, open=directory , cat=file")
            .expect("mappings should parse");

        assert_eq!(
            parsed,
            vec![
                MenuCommandMapping {
                    command: "ls".to_string(),
                    mode: MenuMappingMode::Path,
                },
                MenuCommandMapping {
                    command: "open".to_string(),
                    mode: MenuMappingMode::Directory,
                },
                MenuCommandMapping {
                    command: "cat".to_string(),
                    mode: MenuMappingMode::File,
                },
            ]
        );
    }

    #[test]
    fn empty_raw_value_means_no_mappings() {
        assert!(
            parse_menu_command_mappings("   ")
                .expect("empty mappings should be valid")
                .is_empty()
        );
    }

    #[test]
    fn missing_equals_is_invalid() {
        let err = parse_menu_command_mappings("ls=path,badentry").expect_err("must fail");
        assert_eq!(
            err,
            MenuCommandMappingError::InvalidEntry("badentry".to_string())
        );
    }

    #[test]
    fn empty_command_is_invalid() {
        let err = parse_menu_command_mappings("=path").expect_err("must fail");
        assert_eq!(
            err,
            MenuCommandMappingError::EmptyCommand("=path".to_string())
        );
    }

    #[test]
    fn unknown_mode_is_invalid() {
        let err = parse_menu_command_mappings("ls=unknown").expect_err("must fail");
        assert_eq!(
            err,
            MenuCommandMappingError::InvalidMode {
                entry: "ls=unknown".to_string(),
                mode: "unknown".to_string(),
            }
        );
    }

    #[test]
    fn duplicate_command_is_invalid() {
        let err = parse_menu_command_mappings("ls=path,ls=file").expect_err("must fail");
        assert_eq!(
            err,
            MenuCommandMappingError::DuplicateCommand("ls".to_string())
        );
    }
}
