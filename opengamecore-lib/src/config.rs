use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    #[serde(default = "default_gptk_info_url")]
    pub gptk_info_url: String,
}

fn default_dxvk_url() -> String {
    "https://github.com/doitsujin/dxvk/releases/download/v2.5.3/dxvk-2.5.3.tar.gz".into()
}

fn default_gptk_info_url() -> String {
    "https://developer.apple.com/games/game-porting-toolkit/".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub first_run_complete: bool,
    #[serde(default = "default_auto_update")]
    pub auto_update_database: bool,
    #[serde(default = "default_database_url")]
    pub database_url: String,
    #[serde(default = "default_bundles_url")]
    pub bundles_url: String,
}

fn default_auto_update() -> bool {
    true
}

fn default_database_url() -> String {
    "https://raw.githubusercontent.com/user/opengamecore/main/data/compatibility.json".into()
}

fn default_bundles_url() -> String {
    "https://raw.githubusercontent.com/user/opengamecore/main/data/bundles/".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineConfig {
    pub name: String,
    pub binary_path: PathBuf,
    #[serde(default)]
    pub env_overrides: HashMap<String, String>,
}

impl Default for WineSettings {
    fn default() -> Self {
        Self {
            default: String::new(),
            download_urls: vec![
                "https://github.com/Gcenx/macOS_Wine_builds/releases/download/11.5/wine-devel-11.5-osx64.tar.xz".into(),
            ],
            dxvk_download_url: default_dxvk_url(),
            gptk_info_url: default_gptk_info_url(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            first_run_complete: false,
            auto_update_database: default_auto_update(),
            database_url: default_database_url(),
            bundles_url: default_bundles_url(),
        }
    }
}

impl AppConfig {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        // Try to restore from backup if file is missing or corrupt
        if let Err(e) = crate::fs_utils::restore_from_backup(path) {
            eprintln!(
                "Warning: failed to restore backup for {}: {}",
                path.display(),
                e
            );
        }

        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            if content.trim().is_empty() {
                return Ok(AppConfig::default());
            }
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
        crate::fs_utils::backup(path)?;
        crate::fs_utils::atomic_write(path, &content)?;
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

    #[test]
    fn load_corrupt_toml_returns_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "this is { not valid toml !!!").unwrap();
        let result = AppConfig::load(&path);
        assert!(result.is_err());
    }
}
