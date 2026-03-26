use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub wine: WineSettings,
    #[serde(default)]
    pub app: AppSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineSettings {
    pub default: String,
    pub download_urls: Vec<String>,
    #[serde(default = "default_dxvk_url")]
    pub dxvk_download_url: String,
}

fn default_dxvk_url() -> String {
    "https://github.com/doitsujin/dxvk/releases/download/v2.5.3/dxvk-2.5.3.tar.gz".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub first_run_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineConfig {
    pub name: String,
    pub binary_path: PathBuf,
    #[serde(default)]
    pub env_overrides: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            wine: WineSettings::default(),
            app: AppSettings::default(),
        }
    }
}

impl Default for WineSettings {
    fn default() -> Self {
        Self {
            default: String::new(),
            download_urls: vec![
                "https://github.com/Gcenx/macOS_Wine_builds/releases/download/v9.0/wine-devel-9.0-osx64.tar.xz".into(),
            ],
            dxvk_download_url: default_dxvk_url(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            first_run_complete: false,
        }
    }
}

impl AppConfig {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(AppConfig::default())
        }
    }

    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_config_has_download_url() {
        let config = AppConfig::default();
        assert!(!config.wine.download_urls.is_empty());
        assert!(!config.app.first_run_complete);
    }

    #[test]
    fn config_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        let mut config = AppConfig::default();
        config.wine.default = "wine-9.0".into();
        config.app.first_run_complete = true;

        config.save(&path).unwrap();
        let loaded = AppConfig::load(&path).unwrap();

        assert_eq!(loaded.wine.default, "wine-9.0");
        assert!(loaded.app.first_run_complete);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.toml");
        let config = AppConfig::load(&path).unwrap();
        assert!(!config.app.first_run_complete);
    }
}
