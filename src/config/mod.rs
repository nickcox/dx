use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveOptions {
    pub case_sensitive: bool,
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            case_sensitive: true,
        }
    }
}

/// Interactive menu presentation. `item_max_len` of `None` disables truncation,
/// which `0` or a negative value selects from either source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSettings {
    pub item_max_len: Option<usize>,
    pub border: bool,
    pub max_rows: u16,
    pub max_results: usize,
    pub ls_colors: bool,
}

impl Default for MenuSettings {
    fn default() -> Self {
        Self {
            item_max_len: Some(80),
            border: false,
            max_rows: 20,
            max_results: 1000,
            ls_colors: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppConfig {
    pub search_roots: Vec<PathBuf>,
    pub resolve: ResolveOptions,
    pub menu: MenuSettings,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("explicit config file does not exist: {0}")]
    MissingExplicit(PathBuf),
    #[error("failed to read config file {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse config file {path}: {source}")]
    Parse {
        path: String,
        source: toml::de::Error,
    },
}

#[derive(Debug, Deserialize, Default)]
struct TomlConfig {
    #[serde(default)]
    search_roots: Vec<String>,
    #[serde(default)]
    resolve: ResolveConfig,
    #[serde(default)]
    menu: MenuConfig,
}

#[derive(Debug, Deserialize, Default)]
struct ResolveConfig {
    case_sensitive: Option<bool>,
}

/// Numerics arrive as `i64` so an out-of-range or nonsensical value falls back
/// to the default exactly as it does from the environment, rather than the file
/// and the env var disagreeing about the same input.
#[derive(Debug, Deserialize, Default)]
struct MenuConfig {
    item_max_len: Option<i64>,
    border: Option<bool>,
    max_rows: Option<i64>,
    max_results: Option<i64>,
    ls_colors: Option<bool>,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Self::default();
        if let Some((path, explicit)) = config_path_with_source() {
            match fs::read_to_string(&path) {
                Ok(raw) => {
                    let parsed = parse_toml(&raw, &path)?;
                    config = merge_toml(config, parsed);
                }
                Err(source) if source.kind() == std::io::ErrorKind::NotFound && explicit => {
                    return Err(ConfigError::MissingExplicit(path));
                }
                Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
                Err(source) => {
                    return Err(ConfigError::Read {
                        path: path.display().to_string(),
                        source,
                    });
                }
            }
        }

        Ok(merge_environment(config))
    }
}

pub fn config_path() -> Option<PathBuf> {
    config_path_with_source().map(|(path, _)| path)
}

fn config_path_with_source() -> Option<(PathBuf, bool)> {
    if let Some(path) = env::var_os("DX_CONFIG").filter(|value| !value.is_empty()) {
        return Some((PathBuf::from(path), true));
    }
    dirs::config_dir().map(|dir| (dir.join("dx").join("config.toml"), false))
}

fn parse_toml(raw: &str, path: &Path) -> Result<TomlConfig, ConfigError> {
    toml::from_str::<TomlConfig>(raw).map_err(|source| ConfigError::Parse {
        path: path.display().to_string(),
        source,
    })
}

fn merge_toml(mut base: AppConfig, parsed: TomlConfig) -> AppConfig {
    if !parsed.search_roots.is_empty() {
        base.search_roots = parsed
            .search_roots
            .into_iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
    }

    if let Some(case_sensitive) = parsed.resolve.case_sensitive {
        base.resolve.case_sensitive = case_sensitive;
    }

    if let Some(value) = parsed.menu.item_max_len {
        base.menu.item_max_len = menu_item_max_len(value);
    }
    if let Some(border) = parsed.menu.border {
        base.menu.border = border;
    }
    if let Some(value) = parsed.menu.max_rows {
        base.menu.max_rows = menu_max_rows(value, base.menu.max_rows);
    }
    if let Some(value) = parsed.menu.max_results {
        base.menu.max_results = menu_max_results(value, base.menu.max_results);
    }
    if let Some(ls_colors) = parsed.menu.ls_colors {
        base.menu.ls_colors = ls_colors;
    }

    base
}

/// `0` or negative disables truncation entirely.
fn menu_item_max_len(value: i64) -> Option<usize> {
    usize::try_from(value).ok().filter(|value| *value > 0)
}

fn menu_max_rows(value: i64, default: u16) -> u16 {
    u16::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn menu_max_results(value: i64, default: usize) -> usize {
    usize::try_from(value)
        .ok()
        .filter(|value| *value >= 1)
        .unwrap_or(default)
}

fn merge_environment(mut base: AppConfig) -> AppConfig {
    if let Some(raw) = env::var_os("DX_SEARCH_ROOTS") {
        let roots = env::split_paths(&raw)
            .filter(|path| !path.as_os_str().is_empty())
            .collect::<Vec<_>>();
        if !roots.is_empty() {
            base.search_roots = roots;
        }
    }

    if let Ok(raw) = env::var("DX_CASE_SENSITIVE") {
        base.resolve.case_sensitive = parse_bool(&raw, base.resolve.case_sensitive);
    }

    // An unparseable numeric now falls through to the config file rather than
    // jumping straight to the built-in default, which is the documented
    // precedence finally applying to these settings.
    if let Some(value) = env_i64("DX_MENU_ITEM_MAX_LEN") {
        base.menu.item_max_len = menu_item_max_len(value);
    }
    if let Some(value) = env_i64("DX_MENU_MAX_ROWS") {
        base.menu.max_rows = menu_max_rows(value, base.menu.max_rows);
    }
    if let Some(value) = env_i64("DX_MAX_MENU_RESULTS") {
        base.menu.max_results = menu_max_results(value, base.menu.max_results);
    }

    // These two keep their original truthiness rules; unifying them on
    // `parse_bool` is a separate, user-visible change.
    if let Ok(raw) = env::var("DX_MENU_BORDER") {
        base.menu.border = matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        );
    }
    if let Ok(raw) = env::var("DX_MENU_LS_COLORS") {
        base.menu.ls_colors = raw.trim() == "1";
    }

    base
}

fn env_i64(name: &str) -> Option<i64> {
    let raw = env::var(name).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<i64>().ok()
}

fn parse_bool(input: &str, default: bool) -> bool {
    match input.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::test_support::{self, ScopedProcess, TempDir};

    use super::*;

    fn make_temp_dir(label: &str) -> TempDir {
        test_support::temp_dir(&format!("config-{label}"))
    }

    #[test]
    fn parses_toml_config() {
        let raw = r#"
search_roots = ["/tmp/work", "/tmp/play"]

[resolve]
case_sensitive = false
"#;
        let parsed = parse_toml(raw, Path::new("/tmp/test.toml")).expect("parse should succeed");
        let config = merge_toml(AppConfig::default(), parsed);

        assert_eq!(config.search_roots.len(), 2);
        assert!(!config.resolve.case_sensitive);
    }

    #[test]
    fn defaults_remain_when_toml_fields_missing() {
        let raw = r#"search_roots = []"#;
        let parsed = parse_toml(raw, Path::new("/tmp/test.toml")).expect("parse should succeed");
        let config = merge_toml(AppConfig::default(), parsed);

        assert!(config.search_roots.is_empty());
        assert!(config.resolve.case_sensitive);
    }

    #[test]
    fn environment_path_lists_use_platform_separator() {
        let raw = env::join_paths([Path::new("/a"), Path::new("/b"), Path::new("/c")])
            .expect("join paths");
        let roots = env::split_paths(&raw).collect::<Vec<_>>();
        assert_eq!(
            roots,
            vec![
                PathBuf::from("/a"),
                PathBuf::from("/b"),
                PathBuf::from("/c")
            ]
        );
    }

    /// Writes `body` to an explicit config file and loads it with `env` applied,
    /// so each case exercises the real precedence chain rather than a helper.
    fn load_with(body: &str, env: &[(&str, &str)]) -> AppConfig {
        let temp = make_temp_dir("menu-settings");
        let file = temp.path().join("dx.toml");
        fs::write(&file, body).expect("write config file");

        let mut process = ScopedProcess::new();
        process.set("DX_CONFIG", &file);
        for name in [
            "DX_MENU_ITEM_MAX_LEN",
            "DX_MENU_BORDER",
            "DX_MENU_MAX_ROWS",
            "DX_MAX_MENU_RESULTS",
            "DX_MENU_LS_COLORS",
            "DX_SEARCH_ROOTS",
            "DX_CASE_SENSITIVE",
        ] {
            process.remove(name);
        }
        for (name, value) in env {
            process.set(name, value);
        }

        AppConfig::load().expect("load config")
    }

    #[test]
    fn menu_settings_default_when_nothing_configures_them() {
        let menu = load_with("", &[]).menu;
        assert_eq!(menu, MenuSettings::default());
        assert_eq!(menu.item_max_len, Some(80));
        assert_eq!(menu.max_rows, 20);
        assert_eq!(menu.max_results, 1000);
        assert!(!menu.border);
        assert!(!menu.ls_colors);
    }

    #[test]
    fn menu_settings_come_from_the_config_file() {
        let menu = load_with(
            "[menu]\nitem_max_len = 40\nborder = true\nmax_rows = 8\nmax_results = 25\nls_colors = true\n",
            &[],
        )
        .menu;

        assert_eq!(menu.item_max_len, Some(40));
        assert_eq!(menu.max_rows, 8);
        assert_eq!(menu.max_results, 25);
        assert!(menu.border);
        assert!(menu.ls_colors);
    }

    #[test]
    fn environment_overrides_the_config_file_for_every_menu_setting() {
        let file = "[menu]\nitem_max_len = 40\nborder = true\nmax_rows = 8\nmax_results = 25\nls_colors = true\n";
        let menu = load_with(
            file,
            &[
                ("DX_MENU_ITEM_MAX_LEN", "12"),
                ("DX_MENU_BORDER", "off"),
                ("DX_MENU_MAX_ROWS", "5"),
                ("DX_MAX_MENU_RESULTS", "7"),
                ("DX_MENU_LS_COLORS", "0"),
            ],
        )
        .menu;

        assert_eq!(menu.item_max_len, Some(12));
        assert_eq!(menu.max_rows, 5);
        assert_eq!(menu.max_results, 7);
        assert!(!menu.border);
        assert!(!menu.ls_colors);
    }

    #[test]
    fn environment_only_still_works_without_a_config_file() {
        let menu = load_with(
            "",
            &[
                ("DX_MENU_ITEM_MAX_LEN", "12"),
                ("DX_MENU_BORDER", "yes"),
                ("DX_MENU_MAX_ROWS", "5"),
                ("DX_MAX_MENU_RESULTS", "7"),
                ("DX_MENU_LS_COLORS", "1"),
            ],
        )
        .menu;

        assert_eq!(menu.item_max_len, Some(12));
        assert_eq!(menu.max_rows, 5);
        assert_eq!(menu.max_results, 7);
        assert!(menu.border);
        assert!(menu.ls_colors);
    }

    #[test]
    fn zero_or_negative_item_max_len_disables_truncation_from_either_source() {
        assert_eq!(
            load_with("[menu]\nitem_max_len = 0\n", &[])
                .menu
                .item_max_len,
            None
        );
        assert_eq!(
            load_with("[menu]\nitem_max_len = -3\n", &[])
                .menu
                .item_max_len,
            None
        );
        assert_eq!(
            load_with("", &[("DX_MENU_ITEM_MAX_LEN", "0")])
                .menu
                .item_max_len,
            None
        );
    }

    #[test]
    fn nonsense_numerics_fall_through_rather_than_taking_effect() {
        // An unparseable env value leaves the config-file value in place, which
        // is the documented precedence; previously it jumped to the default.
        let file = "[menu]\nmax_rows = 8\nmax_results = 25\nitem_max_len = 40\n";
        let menu = load_with(
            file,
            &[
                ("DX_MENU_MAX_ROWS", "not-a-number"),
                ("DX_MAX_MENU_RESULTS", ""),
                ("DX_MENU_ITEM_MAX_LEN", "   "),
            ],
        )
        .menu;

        assert_eq!(menu.max_rows, 8);
        assert_eq!(menu.max_results, 25);
        assert_eq!(menu.item_max_len, Some(40));

        // Out-of-range or non-positive values are rejected in favour of the default.
        assert_eq!(load_with("[menu]\nmax_rows = 0\n", &[]).menu.max_rows, 20);
        assert_eq!(
            load_with("[menu]\nmax_rows = 99999999\n", &[])
                .menu
                .max_rows,
            20
        );
        assert_eq!(
            load_with("[menu]\nmax_results = 0\n", &[]).menu.max_results,
            1000
        );
    }

    #[test]
    fn border_and_ls_colors_keep_their_existing_truthiness() {
        // Unified with `parse_bool` in a follow-up; pinned here so that change
        // is visible as a diff rather than a silent shift.
        assert!(load_with("", &[("DX_MENU_BORDER", "true")]).menu.border);
        assert!(load_with("", &[("DX_MENU_BORDER", "on")]).menu.border);
        assert!(!load_with("", &[("DX_MENU_BORDER", "banana")]).menu.border);

        assert!(load_with("", &[("DX_MENU_LS_COLORS", "1")]).menu.ls_colors);
        assert!(
            !load_with("", &[("DX_MENU_LS_COLORS", "true")])
                .menu
                .ls_colors
        );
    }

    #[test]
    fn parse_bool_accepts_common_variants() {
        assert!(parse_bool("true", false));
        assert!(parse_bool("YES", false));
        assert!(!parse_bool("off", true));
        assert!(!parse_bool("0", true));
        assert!(parse_bool("invalid", true));
    }

    #[test]
    fn loads_from_toml_file_path() {
        let mut process = ScopedProcess::new();
        let temp = make_temp_dir("load-file");
        let file = temp.path().join("dx.toml");
        fs::write(
            &file,
            "search_roots=[\"/tmp/r1\"]\n[resolve]\ncase_sensitive=false\n",
        )
        .expect("write config file");

        process.set("DX_CONFIG", &file);
        process.remove("DX_SEARCH_ROOTS");
        process.remove("DX_CASE_SENSITIVE");

        let loaded = AppConfig::load().expect("load config");
        assert_eq!(loaded.search_roots, vec![PathBuf::from("/tmp/r1")]);
        assert!(!loaded.resolve.case_sensitive);
    }

    #[test]
    fn environment_overrides_toml_values() {
        let mut process = ScopedProcess::new();
        let temp = make_temp_dir("load-env");
        let file = temp.path().join("dx.toml");
        fs::write(
            &file,
            "search_roots=[\"/tmp/r1\"]\n[resolve]\ncase_sensitive=true\n",
        )
        .expect("write config file");

        process.set("DX_CONFIG", &file);
        let roots = [temp.path().join("r2"), temp.path().join("r3")];
        process.set(
            "DX_SEARCH_ROOTS",
            env::join_paths(&roots).expect("join search roots"),
        );
        process.set("DX_CASE_SENSITIVE", "false");

        let loaded = AppConfig::load().expect("load config");
        assert_eq!(loaded.search_roots, roots);
        assert!(!loaded.resolve.case_sensitive);
    }

    #[test]
    fn explicit_missing_config_is_an_error() {
        let mut process = ScopedProcess::new();
        let temp = make_temp_dir("missing-explicit");
        process.set("DX_CONFIG", temp.path().join("missing.toml"));

        assert!(matches!(
            AppConfig::load(),
            Err(ConfigError::MissingExplicit(_))
        ));
    }

    #[test]
    fn empty_explicit_config_uses_default_location() {
        let mut process = ScopedProcess::new();
        process.set("DX_CONFIG", "");

        assert_eq!(
            config_path(),
            dirs::config_dir().map(|dir| dir.join("dx/config.toml"))
        );
    }
}
