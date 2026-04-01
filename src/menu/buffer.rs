use crate::complete::{CompletionMode, StackDirection};

/// Parsed context extracted from a command-line buffer at a given cursor position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedBuffer {
    /// The completion mode determined by the command token.
    pub mode: CompletionMode,
    /// The query string typed after the command, if any.
    pub query: Option<String>,
    /// Byte offset where the replacement region starts
    /// (first byte after command token and its trailing whitespace).
    pub replace_start: usize,
    /// Byte offset where the replacement region ends (the cursor position).
    pub replace_end: usize,
    /// Whether a space must be prepended to the replacement value.
    /// True when the cursor sits immediately after the command token with
    /// no intervening whitespace (e.g. buffer="cd", cursor=2).
    pub needs_space_prefix: bool,
}

/// Maps a shell command name to its completion mode.
fn command_to_mode(command: &str) -> Option<CompletionMode> {
    match command {
        "cd" => Some(CompletionMode::Paths),
        "up" => Some(CompletionMode::Ancestors),
        "cdf" | "z" => Some(CompletionMode::Frecents),
        "cdr" => Some(CompletionMode::Recents),
        "back" | "cd-" => Some(CompletionMode::Stack(StackDirection::Back)),
        "forward" | "cd+" => Some(CompletionMode::Stack(StackDirection::Forward)),
        _ => None,
    }
}

/// Parses a command-line buffer at the given cursor byte position to extract
/// the completion mode, query string, and replacement byte range.
///
/// Returns `None` if the buffer is empty, the cursor precedes the end of the
/// command token, or the command token is not a recognized navigation command.
pub fn parse_buffer(buffer: &str, cursor: usize) -> Option<ParsedBuffer> {
    let cursor = cursor.min(buffer.len());

    // Work only with the portion of the buffer up to the cursor.
    let visible = &buffer[..cursor];

    // Skip leading whitespace to find the command token.
    let trimmed = visible.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    let leading_ws = visible.len() - trimmed.len();

    // The command token ends at the first whitespace character (or the end of trimmed).
    let cmd_len = trimmed
        .find(|c: char| c.is_whitespace())
        .unwrap_or(trimmed.len());
    let command = &trimmed[..cmd_len];
    let mode = command_to_mode(command)?;

    let cmd_end_byte = leading_ws + cmd_len;

    // If the cursor hasn't moved past the command token, there's no query region yet.
    // We still return a parsed result so the menu can show unfiltered candidates.
    // In this case, needs_space_prefix is true because there's no whitespace separator.
    if cursor <= cmd_end_byte {
        // Cursor is within or at the end of the command token — only valid if it
        // covers the full command token (partial matches should not trigger the menu).
        if cursor < cmd_end_byte {
            return None;
        }
        return Some(ParsedBuffer {
            mode,
            query: None,
            replace_start: cursor,
            replace_end: cursor,
            needs_space_prefix: true,
        });
    }

    // Count whitespace between the command token and the query region.
    let after_cmd = &buffer[cmd_end_byte..cursor];
    let ws_len = after_cmd.len() - after_cmd.trim_start().len();
    let query_start = cmd_end_byte + ws_len;

    let query_text = &buffer[query_start..cursor];
    let query = if query_text.is_empty() {
        None
    } else {
        Some(query_text.to_string())
    };

    Some(ParsedBuffer {
        mode,
        query,
        replace_start: query_start,
        replace_end: cursor,
        needs_space_prefix: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::complete::{CompletionMode, StackDirection};

    #[test]
    fn cd_with_query() {
        let p = parse_buffer("cd foo", 6).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Paths);
        assert_eq!(p.query.as_deref(), Some("foo"));
        assert_eq!(p.replace_start, 3);
        assert_eq!(p.replace_end, 6);
        assert!(!p.needs_space_prefix);
    }

    #[test]
    fn cd_with_trailing_space_no_query() {
        let p = parse_buffer("cd ", 3).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Paths);
        assert_eq!(p.query, None);
        assert_eq!(p.replace_start, 3);
        assert_eq!(p.replace_end, 3);
        assert!(!p.needs_space_prefix);
    }

    #[test]
    fn cd_no_space_needs_prefix() {
        let p = parse_buffer("cd", 2).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Paths);
        assert_eq!(p.query, None);
        assert_eq!(p.replace_start, 2);
        assert_eq!(p.replace_end, 2);
        assert!(p.needs_space_prefix);
    }

    #[test]
    fn up_maps_to_ancestors() {
        let p = parse_buffer("up", 2).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Ancestors);
    }

    #[test]
    fn cdf_maps_to_frecents() {
        let p = parse_buffer("cdf proj", 8).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Frecents);
        assert_eq!(p.query.as_deref(), Some("proj"));
    }

    #[test]
    fn z_maps_to_frecents() {
        let p = parse_buffer("z work", 6).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Frecents);
        assert_eq!(p.query.as_deref(), Some("work"));
    }

    #[test]
    fn cdr_maps_to_recents() {
        let p = parse_buffer("cdr", 3).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Recents);
    }

    #[test]
    fn back_maps_to_stack_back() {
        let p = parse_buffer("back", 4).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Stack(StackDirection::Back));
    }

    #[test]
    fn cd_minus_maps_to_stack_back() {
        let p = parse_buffer("cd- ", 4).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Stack(StackDirection::Back));
    }

    #[test]
    fn forward_maps_to_stack_forward() {
        let p = parse_buffer("forward", 7).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Stack(StackDirection::Forward));
    }

    #[test]
    fn cd_plus_maps_to_stack_forward() {
        let p = parse_buffer("cd+ ", 4).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Stack(StackDirection::Forward));
    }

    #[test]
    fn unrecognized_command_returns_none() {
        assert!(parse_buffer("ls -la", 6).is_none());
    }

    #[test]
    fn empty_buffer_returns_none() {
        assert!(parse_buffer("", 0).is_none());
    }

    #[test]
    fn whitespace_only_buffer_returns_none() {
        assert!(parse_buffer("   ", 3).is_none());
    }

    #[test]
    fn cursor_mid_command_returns_none() {
        assert!(parse_buffer("cd foo", 1).is_none());
    }

    #[test]
    fn leading_whitespace_is_handled() {
        let p = parse_buffer("  cd foo", 8).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Paths);
        assert_eq!(p.query.as_deref(), Some("foo"));
        assert_eq!(p.replace_start, 5);
        assert_eq!(p.replace_end, 8);
    }

    #[test]
    fn multiple_spaces_between_command_and_query() {
        let p = parse_buffer("cd   bar", 8).expect("should parse");
        assert_eq!(p.query.as_deref(), Some("bar"));
        assert_eq!(p.replace_start, 5);
        assert_eq!(p.replace_end, 8);
    }

    #[test]
    fn cursor_beyond_buffer_is_clamped() {
        let p = parse_buffer("cd foo", 100).expect("should parse");
        assert_eq!(p.replace_end, 6);
        assert_eq!(p.query.as_deref(), Some("foo"));
    }
}
