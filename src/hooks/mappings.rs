use std::collections::HashSet;
use std::fmt;

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
    command: MenuCommandName,
    mode: MenuMappingMode,
}

impl MenuCommandMapping {
    pub fn command(&self) -> &str {
        self.command.as_str()
    }

    pub fn mode(&self) -> MenuMappingMode {
        self.mode
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MenuCommandName(String);

impl MenuCommandName {
    fn parse(raw: &str) -> Option<Self> {
        let mut bytes = raw.bytes();
        let first = bytes.next()?;
        if !first.is_ascii_alphanumeric() && first != b'_' {
            return None;
        }

        bytes
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
            .then(|| Self(raw.to_string()))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MenuCommandName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MenuCommandMappingError {
    #[error("invalid mapping entry '{0}' (expected <command>=<mode>)")]
    InvalidEntry(String),
    #[error("mapping command cannot be empty in entry '{0}'")]
    EmptyCommand(String),
    #[error(
        "invalid mapping command '{command}' in entry '{entry}' (expected an ASCII command name starting with a letter, digit, or '_', followed only by letters, digits, '_', '-', or '.')"
    )]
    InvalidCommand { entry: String, command: String },
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

        let Some(command) = MenuCommandName::parse(command) else {
            return Err(MenuCommandMappingError::InvalidCommand {
                entry: entry.to_string(),
                command: command.to_string(),
            });
        };

        let Some(mode) = MenuMappingMode::parse(mode) else {
            return Err(MenuCommandMappingError::InvalidMode {
                entry: entry.to_string(),
                mode: mode.trim().to_string(),
            });
        };

        if !seen.insert(command.clone()) {
            return Err(MenuCommandMappingError::DuplicateCommand(
                command.to_string(),
            ));
        }

        mappings.push(MenuCommandMapping { command, mode });
    }

    Ok(mappings)
}

#[cfg(test)]
mod tests {
    use super::{MenuCommandMappingError, MenuMappingMode, parse_menu_command_mappings};

    #[test]
    fn parses_valid_menu_mappings() {
        let parsed = parse_menu_command_mappings("ls=path, open=directory , cat=file")
            .expect("mappings should parse");

        let values = parsed
            .iter()
            .map(|mapping| (mapping.command(), mapping.mode()))
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            vec![
                ("ls", MenuMappingMode::Path),
                ("open", MenuMappingMode::Directory),
                ("cat", MenuMappingMode::File),
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
    fn rejects_commands_that_are_unsafe_in_generated_shell_code() {
        let unsafe_commands = [
            "-option",
            "two words",
            "line\nbreak",
            "close)",
            "semi;colon",
            "dollar$var",
            "sub$(command)",
            "back`tick",
            "single'quote",
            "double\"quote",
            "back\\slash",
            "colon:name",
            "nonascii-é",
        ];

        for command in unsafe_commands {
            let entry = format!("{command}=path");
            let err = parse_menu_command_mappings(&entry).expect_err("command must fail");
            assert_eq!(
                err,
                MenuCommandMappingError::InvalidCommand {
                    entry: entry.clone(),
                    command: command.to_string(),
                },
                "unexpected result for {entry:?}"
            );
        }
    }

    #[test]
    fn accepts_conservative_cross_shell_command_names() {
        let parsed = parse_menu_command_mappings(
            "ls=path,Get-ChildItem=directory,git.status=file,_private=path,7z=path",
        )
        .expect("safe commands should parse");

        assert_eq!(
            parsed
                .iter()
                .map(|mapping| mapping.command())
                .collect::<Vec<_>>(),
            ["ls", "Get-ChildItem", "git.status", "_private", "7z"]
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
