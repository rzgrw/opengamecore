use crate::error::{Error, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// DXVK DLL names that get installed into Wine bottles
const DXVK_DLLS: &[&str] = &["d3d9", "d3d10core", "d3d11", "dxgi"];

/// Check if DXVK is installed in a bottle
pub fn is_installed(bottle_dir: &Path) -> bool {
    let sys32 = bottle_dir.join("drive_c/windows/system32");
    DXVK_DLLS
        .iter()
        .all(|dll| sys32.join(format!("{}.dll", dll)).exists())
}

/// Install DXVK into a bottle from a DXVK directory (containing x64/ and x32/ subdirs)
pub fn install(dxvk_dir: &Path, bottle_dir: &Path) -> Result<()> {
    let sys32 = bottle_dir.join("drive_c/windows/system32");
    let syswow64 = bottle_dir.join("drive_c/windows/syswow64");

    std::fs::create_dir_all(&sys32)?;

    // Install 64-bit DLLs to system32
    let x64_dir = dxvk_dir.join("x64");
    if x64_dir.exists() {
        for dll in DXVK_DLLS {
            let src = x64_dir.join(format!("{}.dll", dll));
            if src.exists() {
                // Backup original if exists
                let dest = sys32.join(format!("{}.dll", dll));
                if dest.exists() {
                    let backup = sys32.join(format!("{}.dll.orig", dll));
                    if !backup.exists() {
                        std::fs::rename(&dest, &backup)?;
                    }
                }
                std::fs::copy(&src, &dest)?;
            }
        }
    }

    // Install 32-bit DLLs to syswow64 if it exists
    let x32_dir = dxvk_dir.join("x32");
    if x32_dir.exists() && syswow64.exists() {
        for dll in DXVK_DLLS {
            let src = x32_dir.join(format!("{}.dll", dll));
            if src.exists() {
                let dest = syswow64.join(format!("{}.dll", dll));
                if dest.exists() {
                    let backup = syswow64.join(format!("{}.dll.orig", dll));
                    if !backup.exists() {
                        std::fs::rename(&dest, &backup)?;
                    }
                }
                std::fs::copy(&src, &dest)?;
            }
        }
    }

    Ok(())
}

/// Uninstall DXVK from a bottle (restore originals)
pub fn uninstall(bottle_dir: &Path) -> Result<()> {
    let sys32 = bottle_dir.join("drive_c/windows/system32");
    let syswow64 = bottle_dir.join("drive_c/windows/syswow64");

    for dir in &[sys32, syswow64] {
        if !dir.exists() {
            continue;
        }
        for dll in DXVK_DLLS {
            let backup = dir.join(format!("{}.dll.orig", dll));
            let dest = dir.join(format!("{}.dll", dll));
            if backup.exists() {
                std::fs::rename(&backup, &dest)?;
            }
        }
    }

    Ok(())
}

/// Download and extract DXVK release
pub async fn download_and_extract(url: &str, data_dir: &Path) -> Result<PathBuf> {
    let dxvk_dir = data_dir.join("dxvk");
    std::fs::create_dir_all(&dxvk_dir)?;

    let archive_name = url.split('/').next_back().unwrap_or("dxvk.tar.gz");
    let archive_path = dxvk_dir.join(archive_name);

    let response = reqwest::get(url)
        .await
        .map_err(|e| Error::Download(e.to_string()))?;

    if !response.status().is_success() {
        return Err(Error::Download(format!(
            "HTTP {}: {}",
            response.status(),
            url
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| Error::Download(e.to_string()))?;
    std::fs::write(&archive_path, &bytes)?;

    // Snapshot existing directories before extraction
    let before: HashSet<PathBuf> = std::fs::read_dir(&dxvk_dir)?
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();

    let status = std::process::Command::new("tar")
        .args(["xf"])
        .arg(&archive_path)
        .current_dir(&dxvk_dir)
        .status()?;

    if !status.success() {
        return Err(Error::Download("Failed to extract DXVK archive".into()));
    }

    std::fs::remove_file(&archive_path).ok();

    // Find the new directory by diffing against snapshot
    let new_dir = std::fs::read_dir(&dxvk_dir)?
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .find(|p| !before.contains(p))
        .ok_or_else(|| Error::Download("No new directory found after extraction".into()))?;

    // Remove macOS quarantine attribute
    let _ = std::process::Command::new("xattr")
        .args(["-rd", "com.apple.quarantine"])
        .arg(&new_dir)
        .status();

    Ok(new_dir)
}

/// Get the environment variables needed for DXVK DLL overrides
pub fn env_overrides() -> std::collections::HashMap<String, String> {
    let mut env = std::collections::HashMap::new();
    let overrides = DXVK_DLLS
        .iter()
        .map(|dll| format!("{}=n", dll))
        .collect::<Vec<_>>()
        .join(";");
    env.insert("WINEDLLOVERRIDES".into(), overrides);
    env
}

/// Get DXVK HUD environment variable
pub fn hud_env(enabled: bool) -> std::collections::HashMap<String, String> {
    let mut env = std::collections::HashMap::new();
    if enabled {
        env.insert("DXVK_HUD".into(), "fps,devinfo".into());
    }
    env
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_fake_bottle(dir: &Path) {
        std::fs::create_dir_all(dir.join("drive_c/windows/system32")).unwrap();
    }

    fn make_fake_dxvk(dir: &Path) {
        let x64 = dir.join("x64");
        std::fs::create_dir_all(&x64).unwrap();
        for dll in DXVK_DLLS {
            std::fs::write(x64.join(format!("{}.dll", dll)), "fake dxvk").unwrap();
        }
    }

    #[test]
    fn not_installed_in_fresh_bottle() {
        let tmp = TempDir::new().unwrap();
        make_fake_bottle(tmp.path());
        assert!(!is_installed(tmp.path()));
    }

    #[test]
    fn install_and_check() {
        let tmp = TempDir::new().unwrap();
        let bottle = tmp.path().join("bottle");
        make_fake_bottle(&bottle);

        let dxvk = tmp.path().join("dxvk");
        make_fake_dxvk(&dxvk);

        install(&dxvk, &bottle).unwrap();
        assert!(is_installed(&bottle));
    }

    #[test]
    fn install_backs_up_originals() {
        let tmp = TempDir::new().unwrap();
        let bottle = tmp.path().join("bottle");
        make_fake_bottle(&bottle);

        // Put original DLLs in place
        let sys32 = bottle.join("drive_c/windows/system32");
        for dll in DXVK_DLLS {
            std::fs::write(sys32.join(format!("{}.dll", dll)), "original").unwrap();
        }

        let dxvk = tmp.path().join("dxvk");
        make_fake_dxvk(&dxvk);

        install(&dxvk, &bottle).unwrap();

        // Check backups exist
        for dll in DXVK_DLLS {
            let backup = sys32.join(format!("{}.dll.orig", dll));
            assert!(backup.exists());
            assert_eq!(std::fs::read_to_string(&backup).unwrap(), "original");
        }
    }

    #[test]
    fn uninstall_restores_originals() {
        let tmp = TempDir::new().unwrap();
        let bottle = tmp.path().join("bottle");
        make_fake_bottle(&bottle);

        let sys32 = bottle.join("drive_c/windows/system32");
        for dll in DXVK_DLLS {
            std::fs::write(sys32.join(format!("{}.dll", dll)), "original").unwrap();
        }

        let dxvk = tmp.path().join("dxvk");
        make_fake_dxvk(&dxvk);

        install(&dxvk, &bottle).unwrap();
        uninstall(&bottle).unwrap();

        for dll in DXVK_DLLS {
            let content = std::fs::read_to_string(sys32.join(format!("{}.dll", dll))).unwrap();
            assert_eq!(content, "original");
        }
    }

    #[test]
    fn env_overrides_sets_dll_overrides() {
        let env = env_overrides();
        let overrides = env.get("WINEDLLOVERRIDES").unwrap();
        assert!(overrides.contains("d3d11=n"));
        assert!(overrides.contains("dxgi=n"));
    }

    #[test]
    fn is_installed_false_with_partial_dlls() {
        let tmp = TempDir::new().unwrap();
        let sys32 = tmp.path().join("drive_c/windows/system32");
        std::fs::create_dir_all(&sys32).unwrap();
        // Only install 2 of 4 DLLs
        std::fs::write(sys32.join("d3d9.dll"), "fake").unwrap();
        std::fs::write(sys32.join("d3d11.dll"), "fake").unwrap();
        assert!(!is_installed(tmp.path()));
    }

    #[test]
    fn uninstall_without_backups_is_safe_noop() {
        let tmp = TempDir::new().unwrap();
        let sys32 = tmp.path().join("drive_c/windows/system32");
        std::fs::create_dir_all(&sys32).unwrap();
        // Put DLLs with no .orig backups
        for dll in &["d3d9", "d3d10core", "d3d11", "dxgi"] {
            std::fs::write(sys32.join(format!("{}.dll", dll)), "original").unwrap();
        }
        uninstall(tmp.path()).unwrap();
        // Original DLLs should still be intact
        for dll in &["d3d9", "d3d10core", "d3d11", "dxgi"] {
            assert_eq!(
                std::fs::read_to_string(sys32.join(format!("{}.dll", dll))).unwrap(),
                "original"
            );
        }
    }
}
