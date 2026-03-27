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

impl Error {
    /// Returns a user-friendly error message suitable for display in the GUI.
    pub fn user_message(&self) -> String {
        match self {
            Error::Io(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                "Permission denied. Check file permissions.".into()
            }
            Error::Io(e) if e.kind() == std::io::ErrorKind::NotFound => {
                "File or directory not found.".into()
            }
            Error::Io(e) => format!("System error: {}", e),
            Error::Config(msg) => format!("Configuration error: {}", msg),
            Error::TomlParse(_) => {
                "Failed to parse configuration file. It may be corrupted.".into()
            }
            Error::TomlSerialize(_) => "Failed to save configuration.".into(),
            Error::WineNotFound(msg) => format!(
                "Wine not found: {}. Install Wine or configure the path in Settings.",
                msg
            ),
            Error::BottleNotFound(path) => format!(
                "Game bottle not found at {}. Try resetting the bottle.",
                path.display()
            ),
            Error::GameNotFound(slug) => format!("Game '{}' not found in library.", slug),
            Error::Download(msg) => {
                format!("Download failed: {}. Check your internet connection.", msg)
            }
            Error::Process(msg) => format!("Process error: {}. The game may have crashed.", msg),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_messages_are_helpful() {
        let err = Error::WineNotFound("wine64".into());
        let msg = err.user_message();
        assert!(msg.contains("Wine not found"));
        assert!(msg.contains("Settings"));

        let err = Error::BottleNotFound(std::path::PathBuf::from("/foo/bar"));
        let msg = err.user_message();
        assert!(msg.contains("bottle not found"));
    }

    #[test]
    fn user_message_io_permission_denied() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test");
        let err = Error::Io(io_err);
        assert!(err.user_message().contains("Permission denied"));
    }

    #[test]
    fn user_message_download() {
        let err = Error::Download("connection refused".into());
        let msg = err.user_message();
        assert!(msg.contains("Download failed"));
        assert!(msg.contains("internet connection"));
    }

    #[test]
    fn user_message_process() {
        let err = Error::Process("segfault".into());
        assert!(err.user_message().contains("crashed"));
    }
}
