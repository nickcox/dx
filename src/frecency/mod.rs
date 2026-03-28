use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

pub trait FrecencyProvider {
    fn query(&self, filter: &str) -> Vec<PathBuf>;
    fn is_available(&self) -> bool;
}

#[derive(Debug)]
pub struct ZoxideProvider {
    binary: String,
    available: OnceLock<bool>,
}

impl ZoxideProvider {
    pub fn new() -> Self {
        Self {
            binary: "zoxide".to_string(),
            available: OnceLock::new(),
        }
    }

    #[cfg(test)]
    fn with_binary(binary: &str) -> Self {
        Self {
            binary: binary.to_string(),
            available: OnceLock::new(),
        }
    }

    fn detect_availability(&self) -> bool {
        Command::new(&self.binary)
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
}

impl Default for ZoxideProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FrecencyProvider for ZoxideProvider {
    fn query(&self, filter: &str) -> Vec<PathBuf> {
        if !self.is_available() {
            return Vec::new();
        }

        let mut command = Command::new(&self.binary);
        command.arg("query").arg("--list");

        let trimmed = filter.trim();
        if !trimmed.is_empty() {
            command.arg(trimmed);
        }

        let output = match command.output() {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };

        if !output.status.success() {
            return Vec::new();
        }

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(PathBuf::from)
            .collect()
    }

    fn is_available(&self) -> bool {
        *self.available.get_or_init(|| self.detect_availability())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{FrecencyProvider, ZoxideProvider};

    #[derive(Debug)]
    struct MockProvider {
        available: bool,
        values: Vec<PathBuf>,
    }

    impl FrecencyProvider for MockProvider {
        fn query(&self, _filter: &str) -> Vec<PathBuf> {
            self.values.clone()
        }

        fn is_available(&self) -> bool {
            self.available
        }
    }

    #[test]
    fn unavailable_zoxide_provider_reports_false() {
        let provider = ZoxideProvider::with_binary("dx-zoxide-missing-for-test");
        assert!(!provider.is_available());
    }

    #[test]
    fn unavailable_zoxide_provider_returns_empty_query_results() {
        let provider = ZoxideProvider::with_binary("dx-zoxide-missing-for-test");
        assert!(provider.query("proj").is_empty());
    }

    #[test]
    fn frecency_provider_trait_contract_can_be_mocked() {
        let provider = MockProvider {
            available: true,
            values: vec![PathBuf::from("/a"), PathBuf::from("/b")],
        };

        assert!(provider.is_available());
        assert_eq!(
            provider.query("anything"),
            vec![PathBuf::from("/a"), PathBuf::from("/b")]
        );
    }
}
