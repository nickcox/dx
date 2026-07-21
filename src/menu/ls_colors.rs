use std::collections::HashMap;
use std::path::Path;

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, PartialEq)]
pub struct LsColorsConfig {
    dir: Option<Style>,
    symlink: Option<Style>,
    executable: Option<Style>,
    socket: Option<Style>,
    pipe: Option<Style>,
    block_dev: Option<Style>,
    char_dev: Option<Style>,
    setuid: Option<Style>,
    setgid: Option<Style>,
    sticky_other_writable: Option<Style>,
    other_writable: Option<Style>,
    sticky: Option<Style>,
    orphan_symlink: Option<Style>,
    missing: Option<Style>,
    extensions: HashMap<String, Style>,
}

impl LsColorsConfig {
    pub fn style_for_path(&self, path: &Path) -> Option<Style> {
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => return self.extension_style(path).or(self.missing),
        };
        let ft = metadata.file_type();

        if ft.is_symlink() {
            if self.orphan_symlink.is_some() && !path.exists() {
                return self.orphan_symlink;
            }
            return self.symlink;
        }

        if ft.is_dir() {
            return self.directory_style(&metadata).or(self.dir);
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;

            if ft.is_socket() {
                return self.socket;
            }
            if ft.is_fifo() {
                return self.pipe;
            }
            if ft.is_block_device() {
                return self.block_dev;
            }
            if ft.is_char_device() {
                return self.char_dev;
            }
        }

        if ft.is_file() {
            return self.file_style(path, &metadata);
        }

        None
    }

    fn file_style(&self, path: &Path, metadata: &std::fs::Metadata) -> Option<Style> {
        #[cfg(unix)]
        {
            let mode = unix_mode(metadata);
            if mode & 0o4000 != 0 && self.setuid.is_some() {
                return self.setuid;
            }
            if mode & 0o2000 != 0 && self.setgid.is_some() {
                return self.setgid;
            }
        }

        if is_executable(metadata) && self.executable.is_some() {
            return self.executable;
        }

        self.extension_style(path)
    }

    fn directory_style(&self, metadata: &std::fs::Metadata) -> Option<Style> {
        #[cfg(unix)]
        {
            let mode = unix_mode(metadata);
            if mode & 0o1002 == 0o1002 && self.sticky_other_writable.is_some() {
                return self.sticky_other_writable;
            }
            if mode & 0o0002 != 0 && self.other_writable.is_some() {
                return self.other_writable;
            }
            if mode & 0o1000 != 0 && self.sticky.is_some() {
                return self.sticky;
            }
        }

        None
    }

    fn extension_style(&self, path: &Path) -> Option<Style> {
        let filename = path.file_name()?.to_string_lossy().to_lowercase();
        self.extensions
            .iter()
            .filter(|(suffix, _)| filename.ends_with(suffix.as_str()))
            .max_by_key(|(suffix, _)| suffix.len())
            .map(|(_, style)| *style)
    }
}

#[cfg(unix)]
fn unix_mode(metadata: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode()
}

#[cfg(unix)]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &std::fs::Metadata) -> bool {
    false
}

pub fn parse_ls_colors(val: &str) -> LsColorsConfig {
    let mut config = LsColorsConfig {
        dir: None,
        symlink: None,
        executable: None,
        socket: None,
        pipe: None,
        block_dev: None,
        char_dev: None,
        setuid: None,
        setgid: None,
        sticky_other_writable: None,
        other_writable: None,
        sticky: None,
        orphan_symlink: None,
        missing: None,
        extensions: HashMap::new(),
    };

    for entry in val.split(':') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let Some((key, value)) = entry.split_once('=') else {
            continue;
        };
        let style = parse_color_value(value);
        match key {
            "di" => config.dir = Some(style),
            "ln" => config.symlink = Some(style),
            "ex" => config.executable = Some(style),
            "so" => config.socket = Some(style),
            "pi" => config.pipe = Some(style),
            "bd" => config.block_dev = Some(style),
            "cd" => config.char_dev = Some(style),
            "su" => config.setuid = Some(style),
            "sg" => config.setgid = Some(style),
            "tw" => config.sticky_other_writable = Some(style),
            "ow" => config.other_writable = Some(style),
            "st" => config.sticky = Some(style),
            "or" => config.orphan_symlink = Some(style),
            "mi" => config.missing = Some(style),
            _ => {
                if let Some(ext) = key.strip_prefix('*') {
                    config.extensions.insert(ext.to_lowercase(), style);
                }
            }
        }
    }

    config
}

fn parse_color_value(val: &str) -> Style {
    let mut style = Style::default();
    let parts: Vec<&str> = val.split(';').collect();
    let mut i = 0;
    while i < parts.len() {
        match parts[i] {
            "38" => {
                i += 1;
                if i < parts.len() && parts[i] == "5" {
                    i += 1;
                    if i < parts.len()
                        && let Ok(n) = parts[i].parse::<u8>()
                    {
                        style = style.fg(Color::Indexed(n));
                    }
                } else if i < parts.len() && parts[i] == "2" && i + 3 < parts.len() {
                    let r = parts[i + 1].parse::<u8>().ok();
                    let g = parts[i + 2].parse::<u8>().ok();
                    let b = parts[i + 3].parse::<u8>().ok();
                    if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                        style = style.fg(Color::Rgb(r, g, b));
                    }
                    i += 3;
                }
            }
            "48" => {
                i += 1;
                if i < parts.len() && parts[i] == "5" {
                    i += 1;
                    if i < parts.len()
                        && let Ok(n) = parts[i].parse::<u8>()
                    {
                        style = style.bg(Color::Indexed(n));
                    }
                } else if i < parts.len() && parts[i] == "2" && i + 3 < parts.len() {
                    let r = parts[i + 1].parse::<u8>().ok();
                    let g = parts[i + 2].parse::<u8>().ok();
                    let b = parts[i + 3].parse::<u8>().ok();
                    if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                        style = style.bg(Color::Rgb(r, g, b));
                    }
                    i += 3;
                }
            }
            n => {
                if let Ok(num) = n.parse::<u8>() {
                    match num {
                        0 => {}
                        1 => style = style.add_modifier(Modifier::BOLD),
                        2 => style = style.add_modifier(Modifier::DIM),
                        3 => style = style.add_modifier(Modifier::ITALIC),
                        4 => style = style.add_modifier(Modifier::UNDERLINED),
                        5 => style = style.add_modifier(Modifier::SLOW_BLINK),
                        6 => style = style.add_modifier(Modifier::RAPID_BLINK),
                        7 => style = style.add_modifier(Modifier::REVERSED),
                        8 => style = style.add_modifier(Modifier::HIDDEN),
                        9 => style = style.add_modifier(Modifier::CROSSED_OUT),
                        22 => style = style.remove_modifier(Modifier::BOLD),
                        23 => style = style.remove_modifier(Modifier::ITALIC),
                        24 => style = style.remove_modifier(Modifier::UNDERLINED),
                        25 => {
                            style = style.remove_modifier(Modifier::SLOW_BLINK);
                            style = style.remove_modifier(Modifier::RAPID_BLINK);
                        }
                        27 => style = style.remove_modifier(Modifier::REVERSED),
                        28 => style = style.remove_modifier(Modifier::HIDDEN),
                        29 => style = style.remove_modifier(Modifier::CROSSED_OUT),
                        30..=37 => style = style.fg(standard_color(num - 30)),
                        // Extended-color introducers ("38"/"48") are consumed by the
                        // dedicated string arms above, so they never reach this numeric
                        // match; ignore them defensively rather than panicking.
                        38 | 48 => {}
                        39 => style = style.fg(Color::Reset),
                        40..=47 => style = style.bg(standard_color(num - 40)),
                        49 => style = style.bg(Color::Reset),
                        90..=97 => style = style.fg(bright_color(num - 90)),
                        100..=107 => style = style.bg(bright_color(num - 100)),
                        _ => {}
                    }
                }
            }
        }
        i += 1;
    }
    style
}

fn standard_color(n: u8) -> Color {
    match n {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::Gray,
        _ => Color::Reset,
    }
}

fn bright_color(n: u8) -> Color {
    match n {
        0 => Color::DarkGray,
        1 => Color::LightRed,
        2 => Color::LightGreen,
        3 => Color::LightYellow,
        4 => Color::LightBlue,
        5 => Color::LightMagenta,
        6 => Color::LightCyan,
        7 => Color::White,
        _ => Color::Reset,
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support;

    use super::*;

    #[test]
    fn parse_empty_string() {
        let config = parse_ls_colors("");
        assert!(config.dir.is_none());
        assert!(config.extensions.is_empty());
    }

    #[test]
    fn parse_directory_blue() {
        let config = parse_ls_colors("di=01;34");
        let style = config.dir.expect("dir style");
        assert_eq!(style.fg, Some(Color::Blue));
        assert!(style.add_modifier & Modifier::BOLD != Modifier::empty());
    }

    #[test]
    fn parse_executable_red() {
        let config = parse_ls_colors("ex=01;32");
        let style = config.executable.expect("ex style");
        assert_eq!(style.fg, Some(Color::Green));
    }

    #[test]
    fn parse_extension_entry() {
        let config = parse_ls_colors("*.rs=01;31");
        let style = config.extensions.get(".rs").expect("rs extension");
        assert_eq!(style.fg, Some(Color::Red));
    }

    #[test]
    fn parse_multiple_entries() {
        let config = parse_ls_colors("di=01;34:ex=01;32:*.rs=01;31");
        assert!(config.dir.is_some());
        assert!(config.executable.is_some());
        assert!(config.extensions.contains_key(".rs"));
    }

    #[test]
    fn parse_ignore_invalid_entries() {
        let config = parse_ls_colors("di=01;34:=nope:nope=:di=01;32");
        assert_eq!(config.dir.unwrap().fg, Some(Color::Green));
    }

    #[test]
    fn parse_256_color() {
        let config = parse_ls_colors("di=38;5;82");
        let style = config.dir.expect("dir style with 256-color fg");
        assert_eq!(style.fg, Some(Color::Indexed(82)));
    }

    #[test]
    fn parse_true_color() {
        let config = parse_ls_colors("di=38;2;255;100;50");
        let style = config.dir.expect("dir style with truecolor fg");
        assert_eq!(style.fg, Some(Color::Rgb(255, 100, 50)));
    }

    #[test]
    fn parse_background_color() {
        let config = parse_ls_colors("di=37;41");
        let style = config.dir.expect("dir style with bg");
        assert_eq!(style.fg, Some(Color::Gray));
        assert_eq!(style.bg, Some(Color::Red));
    }

    #[test]
    fn parse_bright_colors() {
        let config = parse_ls_colors("di=01;93");
        let style = config.dir.expect("dir style with bright fg");
        assert_eq!(style.fg, Some(Color::LightYellow));
    }

    #[test]
    fn style_for_path_extension_match() {
        let temp = test_support::temp_dir("ls-colors-extension-match");
        let config = parse_ls_colors("*.rs=01;31:di=01;34");
        let path = temp.path().join("main.rs");
        let style = config.style_for_path(&path);
        assert_eq!(style.and_then(|s| s.fg), Some(Color::Red));
    }

    #[test]
    fn style_for_path_directory() {
        let temp = test_support::temp_dir("ls-colors-directory");
        let config = parse_ls_colors("di=01;34:ex=01;32");
        let style = config.style_for_path(temp.path());
        assert_eq!(style.and_then(|s| s.fg), Some(Color::Blue));
    }

    #[test]
    fn style_for_path_nonexistent_file() {
        let temp = test_support::temp_dir("ls-colors-nonexistent-file");
        let config = parse_ls_colors("di=01;34:mi=01;31");
        let path = temp.path().join("missing");
        let style = config.style_for_path(&path);
        assert_eq!(style.and_then(|s| s.fg), Some(Color::Red));
    }

    #[test]
    fn style_for_path_nonexistent_file_can_still_match_extension() {
        let temp = test_support::temp_dir("ls-colors-nonexistent-extension");
        let config = parse_ls_colors("*.rs=01;31");
        let path = temp.path().join("missing.rs");
        let style = config.style_for_path(&path);
        assert_eq!(style.and_then(|s| s.fg), Some(Color::Red));
    }

    #[test]
    fn style_for_path_longest_suffix_wins() {
        let temp = test_support::temp_dir("ls-colors-longest-suffix");
        let config = parse_ls_colors("*.rs=01;31:*.test.rs=01;32");
        let path = temp.path().join("main.test.rs");
        let style = config.style_for_path(&path);
        assert_eq!(style.and_then(|s| s.fg), Some(Color::Green));
    }

    #[test]
    fn parse_color_value_standard_fg() {
        let style = parse_color_value("31");
        assert_eq!(style.fg, Some(Color::Red));
        assert_eq!(style.bg, None);
    }

    #[test]
    fn parse_color_value_bold_fg() {
        let style = parse_color_value("01;33");
        assert!(style.add_modifier & Modifier::BOLD != Modifier::empty());
        assert_eq!(style.fg, Some(Color::Yellow));
    }

    #[test]
    fn parse_color_value_reset_style() {
        let style = parse_color_value("0");
        assert_eq!(style, Style::default());
    }
}
