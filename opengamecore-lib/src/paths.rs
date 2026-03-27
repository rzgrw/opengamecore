use std::path::PathBuf;

use crate::error::{Error, Result};

const APP_NAME: &str = "OpenGameCore";

/// Returns ~/Library/Application Support/OpenGameCore/
pub fn data_dir() -> Result<PathBuf> {
    dirs::data_dir()
        .map(|d| d.join(APP_NAME))
        .ok_or_else(|| Error::Config("Could not resolve data directory".into()))
}

pub fn config_path() -> Result<PathBuf> {
    data_dir().map(|d| d.join("config.toml"))
}

pub fn games_path() -> Result<PathBuf> {
    data_dir().map(|d| d.join("games.toml"))
}

pub fn bottles_dir() -> Result<PathBuf> {
    data_dir().map(|d| d.join("bottles"))
}

pub fn template_bottle_dir() -> Result<PathBuf> {
    bottles_dir().map(|d| d.join("_template"))
}

pub fn bottle_dir(slug: &str) -> Result<PathBuf> {
    bottles_dir().map(|d| d.join(slug))
}

pub fn wine_dir() -> Result<PathBuf> {
    data_dir().map(|d| d.join("wine"))
}

pub fn icons_dir() -> Result<PathBuf> {
    data_dir().map(|d| d.join("icons"))
}

pub fn logs_dir() -> Result<PathBuf> {
    data_dir().map(|d| d.join("logs"))
}

/// Ensure all app directories exist.
pub fn ensure_dirs() -> Result<()> {
    let dirs = [
        data_dir()?,
        bottles_dir()?,
        wine_dir()?,
        icons_dir()?,
        logs_dir()?,
    ];
    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_dir_ends_with_app_name() {
        let d = data_dir().unwrap();
        assert!(d.ends_with(APP_NAME));
    }

    #[test]
    fn config_path_is_toml() {
        let p = config_path().unwrap();
        assert_eq!(p.extension().unwrap(), "toml");
        assert!(p.starts_with(data_dir().unwrap()));
    }

    #[test]
    fn bottle_dir_uses_slug() {
        let p = bottle_dir("my-game").unwrap();
        assert!(p.ends_with("my-game"));
        assert!(p.starts_with(bottles_dir().unwrap()));
    }

    #[test]
    fn ensure_dirs_creates_directories() {
        ensure_dirs().unwrap();
        assert!(data_dir().unwrap().exists());
        assert!(bottles_dir().unwrap().exists());
        assert!(wine_dir().unwrap().exists());
        assert!(icons_dir().unwrap().exists());
        assert!(logs_dir().unwrap().exists());
    }
}
