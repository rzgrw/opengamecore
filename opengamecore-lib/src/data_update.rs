use std::path::Path;

use crate::config::AppConfig;
use crate::error::{Error, Result};

/// Check if the local database file is stale (>24h old).
pub fn is_stale(path: &Path) -> bool {
    if !path.exists() {
        return true;
    }
    match std::fs::metadata(path) {
        Ok(meta) => match meta.modified() {
            Ok(modified) => {
                let age = std::time::SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or_default();
                age.as_secs() > 86400 // 24 hours
            }
            Err(_) => true,
        },
        Err(_) => true,
    }
}

/// Fetch a file from a URL and write it atomically to dest.
pub async fn fetch_and_save(url: &str, dest: &Path) -> Result<()> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| Error::Download(e.to_string()))?;

    if !response.status().is_success() {
        return Err(Error::Download(format!("HTTP {}: {}", response.status(), url)));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| Error::Download(e.to_string()))?;

    let content = String::from_utf8_lossy(&bytes);

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    crate::fs_utils::backup(dest)?;
    crate::fs_utils::atomic_write(dest, &content)?;
    Ok(())
}

/// Check if update is needed and fetch if so. Returns true if updated.
pub async fn check_and_update(config: &AppConfig, data_dir: &Path) -> Result<bool> {
    if !config.app.auto_update_database {
        return Ok(false);
    }

    let db_path = data_dir.join("compatibility.json");
    if !is_stale(&db_path) {
        return Ok(false);
    }

    fetch_and_save(&config.app.database_url, &db_path).await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn stale_when_missing() {
        assert!(is_stale(Path::new("/nonexistent/file.json")));
    }

    #[test]
    fn not_stale_when_fresh() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.json");
        std::fs::write(&path, "{}").unwrap();
        assert!(!is_stale(&path));
    }
}
