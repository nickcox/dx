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

/// Removes shell quoting from a query token, preserving any trailing `/`.
///
/// The trailing `/` — whether typed by the user or appended by us after a
/// menu selection — is always preserved so that `expand_filesystem_prefix`
/// can enumerate the directory's children on the next Tab press.
///
/// Handles:
/// - Single-quoted: `'foo bar'/` → `foo bar/`, `'it'\''s'/` → `it's/`
/// - Double-quoted: `"foo bar"/` → `foo bar/`
/// - Unquoted: `/usr/local/` → `/usr/local/` (unchanged)
fn unquote_shell_quoted(s: &str) -> String {
    // Single-quoted string (possibly with `'\''` escape sequences for embedded `'`).
    // The trailing `/` sits outside the closing `'`; move it after unquoting.
    //
    // Example: `'foo bar'/`  → `foo bar/`
    // Example: `'it'\''s'/` → `it's/`
    // Example: `'foo bar'`   → `foo bar`
    if s.starts_with('\'') {
        let (s, trailing_slash) = if s.ends_with("'/") {
            (&s[..s.len() - 1], "/")
        } else {
            (s, "")
        };
        let unquoted = s
            .split("'\\''")
            .map(|seg| {
                let inner = seg.strip_prefix('\'').unwrap_or(seg);
                inner.strip_suffix('\'').unwrap_or(inner)
            })
            .collect::<Vec<_>>()
            .join("'");
        return format!("{unquoted}{trailing_slash}");
    }

    // Double-quoted: handle `\"` and `\\` escapes; preserve trailing `/`.
    if s.starts_with('"') {
        let (s, trailing_slash) = if s.ends_with("\"/") {
            (&s[..s.len() - 1], "/")
        } else {
            (s, "")
        };
        let inner = s.strip_prefix('"').unwrap_or(s);
        let inner = inner.strip_suffix('"').unwrap_or(inner);
        let mut result = String::new();
        let mut chars = inner.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\'
                && let Some(&next) = chars.peek()
                && matches!(next, '"' | '\\' | '$' | '`')
            {
                result.push(next);
                chars.next();
                continue;
            }
            result.push(ch);
        }
        return format!("{result}{trailing_slash}");
    }

    // Unquoted — return as-is.
    s.to_string()
}
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
    parse_buffer_with_mode(buffer, cursor, false)
}

/// Parses a command-line buffer with optional shell-mode behavior toggles.
///
/// When `psreadline_mode` is true, POSIX flagged `cd` forms (`-L`, `-P`, `--`)
/// are treated as non-intervention/fallback forms.
pub fn parse_buffer_with_mode(
    buffer: &str,
    cursor: usize,
    psreadline_mode: bool,
) -> Option<ParsedBuffer> {
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

    let query_start = if command == "cd" {
        compute_cd_query_start(buffer, cmd_end_byte, cursor, psreadline_mode)?
    } else {
        // Count whitespace between the command token and the query region.
        let after_cmd = &buffer[cmd_end_byte..cursor];
        let ws_len = after_cmd.len() - after_cmd.trim_start().len();
        cmd_end_byte + ws_len
    };

    let query_text = &buffer[query_start..cursor];
    let query = if query_text.is_empty() {
        None
    } else {
        // Unquote shell-quoted tokens (e.g. `'/Library/Application Support'/` →
        // `/Library/Application Support`) so the resolver receives a raw path.
        // We strip the trailing `/` only when it follows a closing quote — that is
        // the `/` we appended for Tab-ability.  A bare trailing `/` typed by the user
        // (e.g. `cd /` or `cd /usr/`) is meaningful and must be preserved so that
        // `expand_filesystem_prefix` lists the directory's children.
        let unquoted = unquote_shell_quoted(query_text);
        if unquoted.is_empty() {
            None
        } else {
            Some(unquoted)
        }
    };

    Some(ParsedBuffer {
        mode,
        query,
        replace_start: query_start,
        replace_end: cursor,
        needs_space_prefix: false,
    })
}

fn compute_cd_query_start(
    buffer: &str,
    cmd_end_byte: usize,
    cursor: usize,
    psreadline_mode: bool,
) -> Option<usize> {
    let after_cmd = &buffer[cmd_end_byte..cursor];
    let ws_len = after_cmd.len() - after_cmd.trim_start().len();
    let first_start = cmd_end_byte + ws_len;
    if first_start >= cursor {
        return Some(first_start);
    }

    let first_slice = &buffer[first_start..cursor];
    let first_rel_end = first_slice
        .find(|c: char| c.is_whitespace())
        .unwrap_or(first_slice.len());
    let first_end = first_start + first_rel_end;
    let first_token = &buffer[first_start..first_end];

    match first_token {
        "-" => None,
        "-L" | "-P" | "--" => {
            if psreadline_mode {
                return None;
            }

            // Intentional fallback/non-intervention: when the cursor is still at the
            // end of an approved flag token (`cd -P`, `cd -L`, `cd --`) and no
            // separator/path token exists yet, return `None` so shell-native handling
            // remains in control until the user moves into the path argument position.
            if first_end >= cursor {
                return None;
            }

            let post_flag = &buffer[first_end..cursor];
            let post_flag_ws = post_flag.len() - post_flag.trim_start().len();
            Some(first_end + post_flag_ws)
        }
        token if token.starts_with('-') => None,
        _ => Some(first_start),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::complete::{CompletionMode, StackDirection};

    fn parse_psreadline(buffer: &str) -> Option<ParsedBuffer> {
        parse_buffer_with_mode(buffer, buffer.len(), true)
    }

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

    // --- unquote_shell_quoted tests ---

    #[test]
    fn unquote_simple_path_with_slash() {
        // Bare trailing slash preserved
        assert_eq!(unquote_shell_quoted("Downloads/"), "Downloads/");
    }

    #[test]
    fn unquote_bare_slash_preserved() {
        assert_eq!(unquote_shell_quoted("/"), "/");
    }

    #[test]
    fn unquote_absolute_with_trailing_slash_preserved() {
        assert_eq!(unquote_shell_quoted("/usr/local/"), "/usr/local/");
    }

    #[test]
    fn unquote_single_quoted_path_with_appended_slash() {
        // Trailing `/` moved outside the unquoted result — preserved for drill-in
        assert_eq!(
            unquote_shell_quoted("'/Library/Application Support'/"),
            "/Library/Application Support/"
        );
    }

    #[test]
    fn unquote_single_quoted_path_without_slash() {
        assert_eq!(
            unquote_shell_quoted("'/Library/Application Support'"),
            "/Library/Application Support"
        );
    }

    #[test]
    fn unquote_single_quoted_with_embedded_single_quote() {
        assert_eq!(unquote_shell_quoted("'it'\\''s here'/"), "it's here/");
    }

    #[test]
    fn unquote_double_quoted_path() {
        assert_eq!(unquote_shell_quoted("\"foo bar\"/"), "foo bar/");
    }

    #[test]
    fn parse_buffer_handles_quoted_token_for_drill_in() {
        // After selecting '/Library/Application Support'/ the buffer is:
        // `cd '/Library/Application Support'/` — query must include the trailing slash
        // so that expand_filesystem_prefix lists the directory's children.
        let buf = "cd '/Library/Application Support'/";
        let p = parse_buffer(buf, buf.len()).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Paths);
        assert_eq!(p.query.as_deref(), Some("/Library/Application Support/"));
        assert_eq!(p.replace_start, 3);
        assert_eq!(p.replace_end, buf.len());
    }

    #[test]
    fn parse_buffer_handles_simple_slash_suffix_for_drill_in() {
        // `cd Downloads/` — slash preserved for child enumeration
        let buf = "cd Downloads/";
        let p = parse_buffer(buf, buf.len()).expect("should parse");
        assert_eq!(p.query.as_deref(), Some("Downloads/"));
    }

    #[test]
    fn parse_buffer_bare_slash_query() {
        // `cd /` — slash preserved so resolver lists root children
        let buf = "cd /";
        let p = parse_buffer(buf, buf.len()).expect("should parse");
        assert_eq!(p.query.as_deref(), Some("/"));
    }

    #[test]
    fn cd_flagged_forms_isolate_path_token() {
        let p = parse_buffer("cd -P foo", "cd -P foo".len()).expect("should parse");
        assert_eq!(p.mode, CompletionMode::Paths);
        assert_eq!(p.query.as_deref(), Some("foo"));
        assert_eq!(p.replace_start, 6);
        assert_eq!(p.replace_end, 9);

        let p = parse_buffer("cd -L /tmp/work", "cd -L /tmp/work".len()).expect("should parse");
        assert_eq!(p.query.as_deref(), Some("/tmp/work"));
        assert_eq!(p.replace_start, 6);

        let p = parse_buffer("cd -- bar", "cd -- bar".len()).expect("should parse");
        assert_eq!(p.query.as_deref(), Some("bar"));
        assert_eq!(p.replace_start, 6);

        let p = parse_buffer("cd -P ", "cd -P ".len()).expect("should parse");
        assert_eq!(p.query, None);
        assert_eq!(p.replace_start, "cd -P ".len());
        assert_eq!(p.replace_end, "cd -P ".len());
        assert!(!p.needs_space_prefix);

        let p = parse_buffer("cd -- ", "cd -- ".len()).expect("should parse");
        assert_eq!(p.query, None);
        assert_eq!(p.replace_start, "cd -- ".len());
        assert_eq!(p.replace_end, "cd -- ".len());

        let buf = "cd -P '/tmp/with space'/";
        let p = parse_buffer(buf, buf.len()).expect("should parse");
        assert_eq!(p.query.as_deref(), Some("/tmp/with space/"));
        assert_eq!(p.replace_start, 6);

        let buf = "cd -L '/tmp/with space'/";
        let p = parse_buffer(buf, buf.len()).expect("should parse");
        assert_eq!(p.query.as_deref(), Some("/tmp/with space/"));
        assert_eq!(p.replace_start, 6);
    }

    #[test]
    fn cd_approved_flags_without_trailing_space_fall_back() {
        // Intentional non-intervention before the path token starts.
        assert!(parse_buffer("cd -P", "cd -P".len()).is_none());
        assert!(parse_buffer("cd -L", "cd -L".len()).is_none());
        assert!(parse_buffer("cd --", "cd --".len()).is_none());
    }

    #[test]
    fn cd_unsupported_and_lone_dash_forms_fall_back() {
        assert!(parse_buffer("cd -", "cd -".len()).is_none());
        assert!(parse_buffer("cd -Q foo", "cd -Q foo".len()).is_none());
        assert!(parse_buffer("cd -LP foo", "cd -LP foo".len()).is_none());
        assert!(parse_buffer("cd -abc foo", "cd -abc foo".len()).is_none());

        assert!(parse_psreadline("cd -P foo").is_none());
        assert!(parse_psreadline("cd -L foo").is_none());
        assert!(parse_psreadline("cd -- foo").is_none());

        // Unflagged `cd <path>` remains supported in psreadline mode.
        let p = parse_psreadline("cd foo").expect("should parse");
        assert_eq!(p.query.as_deref(), Some("foo"));
    }
}
