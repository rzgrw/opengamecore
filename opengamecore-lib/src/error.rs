use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Wine not found: {0}")]
    WineNotFound(String),

    #[error("Bottle not found: {0}")]
    BottleNotFound(PathBuf),

    #[error("Game not found: {0}")]
    GameNotFound(String),

    #[error("Download failed: {0}")]
    Download(String),

    #[error("Process error: {0}")]
    Process(String),
}

pub type Result<T> = std::result::Result<T, Error>;
