use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Information about an existing bottle.
#[derive(Debug, Clone)]
pub struct BottleInfo {
    pub slug: String,
    pub path: PathBuf,
    pub size_bytes: u64,
}

/// Clone a directory tree using APFS clonefile.
/// Falls back to a recursive copy if clonefile fails.
fn clone_dir(src: &Path, dst: &Path) -> Result<()> {
    // Try APFS clonefile first via `cp -c` (clone flag on macOS)
    let status = std::process::Command::new("cp")
        .args(["-cR"])
        .arg(src)
        .arg(dst)
        .status()?;

    if status.success() {
        return Ok(());
    }

    // Fallback: regular recursive copy
    copy_dir_recursive(src, dst)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

/// Create the template bottle by running wineboot.
pub fn create_template(wine_binary: &Path, template_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(template_dir)?;

    let status = std::process::Command::new(wine_binary)
        .arg("wineboot")
        .arg("--init")
        .env("WINEPREFIX", template_dir)
        .status()?;

    if !status.success() {
        return Err(Error::Process(
            "wineboot failed to initialize template prefix".into(),
        ));
    }
    Ok(())
}

/// Create a new bottle by cloning the template.
pub fn create(template_dir: &Path, bottle_dir: &Path) -> Result<()> {
    if !template_dir.exists() {
        return Err(Error::BottleNotFound(template_dir.to_path_buf()));
    }
    if bottle_dir.exists() {
        return Err(Error::Config(format!(
            "Bottle already exists: {}",
            bottle_dir.display()
        )));
    }
    clone_dir(template_dir, bottle_dir)
}

/// List all bottles (excluding _template).
pub fn list(bottles_dir: &Path) -> Result<Vec<BottleInfo>> {
    let mut bottles = Vec::new();
    if !bottles_dir.exists() {
        return Ok(bottles);
    }
    for entry in std::fs::read_dir(bottles_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('_') || !entry.file_type()?.is_dir() {
            continue;
        }
        bottles.push(BottleInfo {
            slug: name,
            path: entry.path(),
            size_bytes: dir_size(&entry.path()).unwrap_or(0),
        });
    }
    Ok(bottles)
}

/// Delete a bottle directory.
pub fn delete(bottle_dir: &Path) -> Result<()> {
    if !bottle_dir.exists() {
        return Err(Error::BottleNotFound(bottle_dir.to_path_buf()));
    }
    std::fs::remove_dir_all(bottle_dir)?;
    Ok(())
}

/// Reset a bottle: backup metadata, delete, and re-clone from template.
pub fn reset(template_dir: &Path, bottle_dir: &Path) -> Result<()> {
    // Back up the user.reg file if it exists (contains app-specific settings)
    let user_reg = bottle_dir.join("user.reg");
    let user_reg_backup = bottle_dir.with_file_name(format!(
        "{}.user.reg.bak",
        bottle_dir.file_name().unwrap_or_default().to_string_lossy()
    ));
    if user_reg.exists() {
        std::fs::copy(&user_reg, &user_reg_backup)?;
    }

    if bottle_dir.exists() {
        std::fs::remove_dir_all(bottle_dir)?;
    }
    create(template_dir, bottle_dir)?;

    // Clean up backup after successful reset
    if user_reg_backup.exists() {
        std::fs::remove_file(&user_reg_backup).ok();
    }

    Ok(())
}

fn dir_size(path: &Path) -> Result<u64> {
    let mut total = 0u64;
    if path.is_file() {
        return Ok(std::fs::metadata(path)?.len());
    }
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            total += dir_size(&entry.path())?;
        } else {
            total += entry.metadata()?.len();
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_fake_template(dir: &Path) {
        std::fs::create_dir_all(dir.join("drive_c")).unwrap();
        std::fs::write(dir.join("system.reg"), "fake registry").unwrap();
    }

    #[test]
    fn create_clones_template() {
        let tmp = TempDir::new().unwrap();
        let template = tmp.path().join("_template");
        make_fake_template(&template);

        let bottle = tmp.path().join("my-game");
        create(&template, &bottle).unwrap();

        assert!(bottle.join("drive_c").exists());
        assert!(bottle.join("system.reg").exists());
    }

    #[test]
    fn create_fails_if_template_missing() {
        let tmp = TempDir::new().unwrap();
        let result = create(&tmp.path().join("nope"), &tmp.path().join("bottle"));
        assert!(result.is_err());
    }

    #[test]
    fn create_fails_if_bottle_exists() {
        let tmp = TempDir::new().unwrap();
        let template = tmp.path().join("_template");
        make_fake_template(&template);

        let bottle = tmp.path().join("my-game");
        std::fs::create_dir(&bottle).unwrap();

        assert!(create(&template, &bottle).is_err());
    }

    #[test]
    fn list_excludes_template() {
        let tmp = TempDir::new().unwrap();
        make_fake_template(&tmp.path().join("_template"));
        std::fs::create_dir(tmp.path().join("game-one")).unwrap();
        std::fs::create_dir(tmp.path().join("game-two")).unwrap();

        let bottles = list(tmp.path()).unwrap();
        assert_eq!(bottles.len(), 2);
        let slugs: Vec<&str> = bottles.iter().map(|b| b.slug.as_str()).collect();
        assert!(slugs.contains(&"game-one"));
        assert!(slugs.contains(&"game-two"));
    }

    #[test]
    fn delete_removes_bottle() {
        let tmp = TempDir::new().unwrap();
        let bottle = tmp.path().join("game");
        std::fs::create_dir(&bottle).unwrap();
        delete(&bottle).unwrap();
        assert!(!bottle.exists());
    }

    #[test]
    fn reset_recreates_bottle() {
        let tmp = TempDir::new().unwrap();
        let template = tmp.path().join("_template");
        make_fake_template(&template);

        let bottle = tmp.path().join("game");
        create(&template, &bottle).unwrap();
        // Corrupt it
        std::fs::write(bottle.join("corrupt"), "bad").unwrap();

        reset(&template, &bottle).unwrap();
        assert!(bottle.join("drive_c").exists());
        assert!(!bottle.join("corrupt").exists());
    }

    #[test]
    fn reset_creates_bottle_even_if_missing() {
        let tmp = TempDir::new().unwrap();
        let template = tmp.path().join("_template");
        make_fake_template(&template);

        let bottle = tmp.path().join("nonexistent-game");
        assert!(!bottle.exists());

        reset(&template, &bottle).unwrap();
        assert!(bottle.join("drive_c").exists());
    }
}
