//! How each shell spells things: which shells `dx` supports, how a path is
//! quoted for insertion into a command line, and what unit that shell's line
//! editor counts buffer offsets in.

use clap::ValueEnum;

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

/// Quotes `path` using `shell`'s syntax, or `None` when the path holds control
/// characters that no quoting can make safe to inject into a buffer.
pub fn quote_path(shell: Shell, path: &str) -> Option<String> {
    if path.chars().any(char::is_control) {
        return None;
    }

    Some(match shell {
        Shell::Bash | Shell::Zsh => quote_posix(path),
        Shell::Fish => quote_fish(path),
        Shell::Pwsh => quote_pwsh(path),
    })
}

/// Converts a UTF-8 byte offset into the buffer unit the shell's line editor
/// counts in: bytes for Bash, characters for Zsh and Fish, UTF-16 code units
/// for PowerShell.
pub fn offset_from_byte(shell: Shell, buffer: &str, byte_offset: usize) -> Option<usize> {
    if byte_offset > buffer.len() || !buffer.is_char_boundary(byte_offset) {
        return None;
    }

    let prefix = &buffer[..byte_offset];
    Some(match shell {
        Shell::Bash => byte_offset,
        Shell::Zsh | Shell::Fish => prefix.chars().count(),
        Shell::Pwsh => prefix.encode_utf16().count(),
    })
}

/// Whether the string holds characters that require shell quoting.
fn needs_shell_quoting(s: &str) -> bool {
    s.chars().any(|c| {
        c.is_whitespace()
            || matches!(
                c,
                '(' | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '!'
                    | '#'
                    | '$'
                    | '&'
                    | '*'
                    | '?'
                    | ';'
                    | '<'
                    | '>'
                    | '|'
                    | '\\'
                    | '\''
                    | '"'
                    | '`'
                    | '~'
            )
    })
}

fn quote_posix(path: &str) -> String {
    if needs_shell_quoting(path) {
        format!("'{}'", path.replace('\'', "'\\''"))
    } else {
        path.to_string()
    }
}

fn quote_fish(path: &str) -> String {
    if !needs_shell_quoting(path) {
        return path.to_string();
    }

    path.chars().fold(String::new(), |mut escaped, ch| {
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-')) {
            escaped.push('\\');
        }
        escaped.push(ch);
        escaped
    })
}

fn quote_pwsh(path: &str) -> String {
    if needs_shell_quoting(path) {
        format!("'{}'", path.replace('\'', "''"))
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_paths_are_left_unquoted() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Pwsh] {
            assert_eq!(
                quote_path(shell, "src/main.rs").as_deref(),
                Some("src/main.rs")
            );
        }
    }

    /// Three conventions: POSIX ends the quote to emit an escaped one, PowerShell
    /// doubles it, and fish escapes in place without quoting at all.
    #[test]
    fn each_shell_escapes_a_quote_its_own_way() {
        assert_eq!(
            quote_path(Shell::Bash, "it's").as_deref(),
            Some(r"'it'\''s'")
        );
        assert_eq!(
            quote_path(Shell::Zsh, "it's").as_deref(),
            Some(r"'it'\''s'")
        );
        assert_eq!(quote_path(Shell::Pwsh, "it's").as_deref(), Some("'it''s'"));
        assert_eq!(quote_path(Shell::Fish, "it's").as_deref(), Some(r"it\'s"));
    }

    #[test]
    fn a_space_is_quoted_or_escaped_but_never_left_bare() {
        assert_eq!(quote_path(Shell::Bash, "a dir").as_deref(), Some("'a dir'"));
        assert_eq!(quote_path(Shell::Zsh, "a dir").as_deref(), Some("'a dir'"));
        assert_eq!(quote_path(Shell::Pwsh, "a dir").as_deref(), Some("'a dir'"));
        assert_eq!(quote_path(Shell::Fish, "a dir").as_deref(), Some(r"a\ dir"));
    }

    /// Control characters cannot be made safe by quoting, so there is nothing to
    /// insert into the buffer.
    #[test]
    fn control_characters_are_rejected_rather_than_quoted() {
        assert_eq!(quote_path(Shell::Bash, "a\nb"), None);
        assert_eq!(quote_path(Shell::Pwsh, "a\u{7}b"), None);
    }

    /// Each line editor counts its buffer differently, so the same byte offset
    /// maps to three different numbers.
    #[test]
    fn offsets_are_reported_in_each_editors_own_unit() {
        let buffer = "cd é🙂";
        let bytes = buffer.len();
        assert_eq!(offset_from_byte(Shell::Bash, buffer, bytes), Some(bytes));
        assert_eq!(offset_from_byte(Shell::Zsh, buffer, bytes), Some(5));
        assert_eq!(offset_from_byte(Shell::Fish, buffer, bytes), Some(5));
        assert_eq!(offset_from_byte(Shell::Pwsh, buffer, bytes), Some(6));
    }

    #[test]
    fn offsets_off_a_character_boundary_are_rejected() {
        let buffer = "cd é";
        assert_eq!(
            offset_from_byte(Shell::Bash, buffer, buffer.len() - 1),
            None
        );
        assert_eq!(
            offset_from_byte(Shell::Bash, buffer, buffer.len() + 1),
            None
        );
    }
}
