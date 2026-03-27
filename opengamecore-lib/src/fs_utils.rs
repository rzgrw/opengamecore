use std::path::Path;
use crate::error::Result;

/// Write to a file atomically: write to a temp file, then rename.
/// This prevents data corruption from crashes mid-write.
pub fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, content)?;
    std::fs::rename(&temp_path, path)?;
    Ok(())
}

/// Create a backup of a file before modifying it.
/// Backup is stored as <filename>.bak alongside the original.
pub fn backup(path: &Path) -> Result<()> {
    if path.exists() {
        let backup_path = path.with_extension("bak");
        std::fs::copy(path, &backup_path)?;
    }
    Ok(())
}

/// Restore from backup if the original file is missing or corrupt.
pub fn restore_from_backup(path: &Path) -> Result<bool> {
    let backup_path = path.with_extension("bak");
    if !path.exists() && backup_path.exists() {
        std::fs::rename(&backup_path, path)?;
        return Ok(true);
    }
    // Also restore if the file exists but is empty (corrupt write)
    if path.exists() {
        let metadata = std::fs::metadata(path)?;
        if metadata.len() == 0 && backup_path.exists() {
            std::fs::rename(&backup_path, path)?;
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn atomic_write_creates_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.toml");
        atomic_write(&path, "hello = 'world'").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello = 'world'");
        // Temp file should be cleaned up
        assert!(!path.with_extension("tmp").exists());
    }

    #[test]
    fn backup_creates_bak_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.toml");
        std::fs::write(&path, "original").unwrap();
        backup(&path).unwrap();
        assert_eq!(std::fs::read_to_string(path.with_extension("bak")).unwrap(), "original");
    }

    #[test]
    fn backup_of_missing_file_is_noop() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.toml");
        backup(&path).unwrap(); // should not error
    }

    #[test]
    fn restore_from_backup_when_original_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.toml");
        let backup_path = path.with_extension("bak");
        std::fs::write(&backup_path, "backup data").unwrap();

        let restored = restore_from_backup(&path).unwrap();
        assert!(restored);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "backup data");
    }

    #[test]
    fn restore_from_backup_when_original_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.toml");
        std::fs::write(&path, "").unwrap(); // corrupt/empty
        let backup_path = path.with_extension("bak");
        std::fs::write(&backup_path, "good data").unwrap();

        let restored = restore_from_backup(&path).unwrap();
        assert!(restored);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "good data");
    }

    #[test]
    fn no_restore_when_original_exists_and_valid() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.toml");
        std::fs::write(&path, "valid data").unwrap();

        let restored = restore_from_backup(&path).unwrap();
        assert!(!restored);
    }
}
