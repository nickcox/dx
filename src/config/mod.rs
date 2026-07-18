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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppConfig {
    pub search_roots: Vec<PathBuf>,
    pub resolve: ResolveOptions,
}

#[derive(Debug, Error)]
pub enum ConfigError {
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
}

#[derive(Debug, Deserialize, Default)]
struct ResolveConfig {
    case_sensitive: Option<bool>,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Self::default();
        let path = config_path();

        if let Some(path) = path
            && path.exists()
        {
            let raw = fs::read_to_string(&path).map_err(|source| ConfigError::Read {
                path: path.display().to_string(),
                source,
            })?;
            let parsed = parse_toml(&raw, &path)?;
            config = merge_toml(config, parsed);
        }

        Ok(merge_environment(config))
    }
}

pub fn config_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("DX_CONFIG") {
        return Some(PathBuf::from(path));
    }

    dirs::config_dir().map(|dir| dir.join("dx").join("config.toml"))
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

    base
}

fn merge_environment(mut base: AppConfig) -> AppConfig {
    if let Ok(raw) = env::var("DX_SEARCH_ROOTS") {
        let roots = split_paths(&raw)
            .into_iter()
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        if !roots.is_empty() {
            base.search_roots = roots;
        }
    }

    if let Ok(raw) = env::var("DX_CASE_SENSITIVE") {
        base.resolve.case_sensitive = parse_bool(&raw, base.resolve.case_sensitive);
    }

    base
}

fn parse_bool(input: &str, default: bool) -> bool {
    match input.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => default,
    }
}

fn split_paths(raw: &str) -> Vec<String> {
    raw.split(':').map(ToString::to_string).collect()
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
    fn split_paths_supports_multiple_values() {
        let roots = split_paths("/a:/b:/c");
        assert_eq!(roots, vec!["/a", "/b", "/c"]);
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
        process.set("DX_SEARCH_ROOTS", "/tmp/r2:/tmp/r3");
        process.set("DX_CASE_SENSITIVE", "false");

        let loaded = AppConfig::load().expect("load config");
        assert_eq!(
            loaded.search_roots,
            vec![PathBuf::from("/tmp/r2"), PathBuf::from("/tmp/r3")]
        );
        assert!(!loaded.resolve.case_sensitive);
    }
}
