use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::config::WineConfig;
use crate::error::{Error, Result};

/// Scan known locations for Wine binaries.
pub fn discover(wine_dir: &Path) -> Result<Vec<WineConfig>> {
    let mut configs = Vec::new();

    // Check app-managed wine directory
    if wine_dir.exists() {
        for entry in std::fs::read_dir(wine_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(binary) = find_wine_binary(&entry.path()) {
                configs.push(WineConfig {
                    name,
                    binary_path: binary,
                    env_overrides: Default::default(),
                });
            }
        }
    }

    // Check Homebrew locations
    for brew_path in &[
        "/opt/homebrew/bin/wine64",
        "/usr/local/bin/wine64",
        "/opt/homebrew/bin/wine",
        "/usr/local/bin/wine",
    ] {
        let p = PathBuf::from(brew_path);
        if p.exists() {
            let name = format!("homebrew-{}", p.file_name().unwrap().to_string_lossy());
            if !configs.iter().any(|c| c.binary_path == p) {
                configs.push(WineConfig {
                    name,
                    binary_path: p,
                    env_overrides: Default::default(),
                });
            }
        }
    }

    // Check for Game Porting Toolkit
    for gptk_path in &[
        "/opt/homebrew/bin/game-porting-toolkit",
        "/usr/local/bin/game-porting-toolkit",
        "/opt/homebrew/bin/game-porting-toolkit-no-hud",
    ] {
        let p = PathBuf::from(gptk_path);
        if p.exists() {
            let name = if gptk_path.contains("no-hud") {
                "gptk-no-hud".to_string()
            } else {
                "gptk".to_string()
            };
            if !configs.iter().any(|c| c.binary_path == p) {
                configs.push(WineConfig {
                    name,
                    binary_path: p,
                    env_overrides: gptk_env_overrides(),
                });
            }
        }
    }

    Ok(configs)
}

/// Recommended environment variables for GPTK.
fn gptk_env_overrides() -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("MTL_HUD_ENABLED".into(), "0".into());
    env.insert("WINEESYNC".into(), "1".into());
    env
}

/// Search a Wine installation directory for the wine binary.
fn find_wine_binary(dir: &Path) -> Option<PathBuf> {
    for candidate in &["bin/wine64", "bin/wine"] {
        let p = dir.join(candidate);
        if p.exists() {
            return Some(p);
        }
    }
    // Search one level of subdirectories (for archives like wine-9.0/bin/wine)
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                for candidate in &["bin/wine64", "bin/wine"] {
                    let p = entry.path().join(candidate);
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
        }
    }
    None
}

/// Download and extract a Wine build.
pub async fn download_and_extract(url: &str, wine_dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(wine_dir)?;

    let archive_name = url.split('/').next_back().unwrap_or("wine.tar.xz");
    let archive_path = wine_dir.join(archive_name);

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
    let before: HashSet<PathBuf> = std::fs::read_dir(wine_dir)?
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();

    let status = std::process::Command::new("tar")
        .args(["xf"])
        .arg(&archive_path)
        .current_dir(wine_dir)
        .status()?;

    if !status.success() {
        return Err(Error::Download("Failed to extract Wine archive".into()));
    }

    std::fs::remove_file(&archive_path).ok();

    // Find the new directory by diffing against snapshot
    let new_dir = std::fs::read_dir(wine_dir)?
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .find(|p| !before.contains(p))
        .ok_or_else(|| Error::Download("No new directory found after extraction".into()))?;

    // Remove macOS quarantine attribute so Gatekeeper doesn't block the binaries
    let _ = std::process::Command::new("xattr")
        .args(["-rd", "com.apple.quarantine"])
        .arg(&new_dir)
        .status();

    Ok(new_dir)
}

/// Resolve which WineConfig to use given a config name.
pub fn resolve(configs: &[WineConfig], name: &str) -> Result<WineConfig> {
    if name == "default" || name.is_empty() {
        configs
            .first()
            .cloned()
            .ok_or_else(|| Error::WineNotFound("No Wine installations found".into()))
    } else {
        configs
            .iter()
            .find(|c| c.name == name)
            .cloned()
            .ok_or_else(|| Error::WineNotFound(format!("Wine config '{}' not found", name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn find_wine_binary_direct() {
        let tmp = TempDir::new().unwrap();
        let bin = tmp.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        std::fs::write(bin.join("wine64"), "fake").unwrap();

        assert_eq!(find_wine_binary(tmp.path()), Some(bin.join("wine64")));
    }

    #[test]
    fn find_wine_binary_nested() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("wine-9.0").join("bin");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("wine"), "fake").unwrap();

        assert_eq!(find_wine_binary(tmp.path()), Some(nested.join("wine")));
    }

    #[test]
    fn find_wine_binary_missing() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(find_wine_binary(tmp.path()), None);
    }

    #[test]
    fn discover_finds_local_installs() {
        let tmp = TempDir::new().unwrap();
        let install = tmp.path().join("wine-9.0").join("bin");
        std::fs::create_dir_all(&install).unwrap();
        std::fs::write(install.join("wine64"), "fake").unwrap();

        let configs = discover(tmp.path()).unwrap();
        assert!(configs.iter().any(|c| c.name == "wine-9.0"));
    }

    #[test]
    fn resolve_default_picks_first() {
        let configs = vec![WineConfig {
            name: "wine-9.0".into(),
            binary_path: "/fake/bin/wine".into(),
            env_overrides: Default::default(),
        }];
        let resolved = resolve(&configs, "default").unwrap();
        assert_eq!(resolved.name, "wine-9.0");
    }

    #[test]
    fn resolve_by_name() {
        let configs = vec![
            WineConfig {
                name: "wine-9.0".into(),
                binary_path: "/fake/bin/wine".into(),
                env_overrides: Default::default(),
            },
            WineConfig {
                name: "gptk".into(),
                binary_path: "/fake/gptk/wine".into(),
                env_overrides: Default::default(),
            },
        ];
        let resolved = resolve(&configs, "gptk").unwrap();
        assert_eq!(resolved.name, "gptk");
    }

    #[test]
    fn resolve_missing_errors() {
        let configs: Vec<WineConfig> = vec![];
        assert!(resolve(&configs, "default").is_err());
    }

    #[test]
    fn gptk_env_overrides_set() {
        let env = gptk_env_overrides();
        assert_eq!(env.get("WINEESYNC").unwrap(), "1");
        assert_eq!(env.get("MTL_HUD_ENABLED").unwrap(), "0");
    }

    #[test]
    fn resolve_empty_string_picks_first() {
        let configs = vec![WineConfig {
            name: "wine-9.0".into(),
            binary_path: "/fake/bin/wine".into(),
            env_overrides: Default::default(),
        }];
        let resolved = resolve(&configs, "").unwrap();
        assert_eq!(resolved.name, "wine-9.0");
    }

    #[test]
    fn discover_nonexistent_dir_returns_empty() {
        let configs = discover(Path::new("/definitely/does/not/exist")).unwrap();
        // May contain homebrew results but should not error
        // Just check it doesn't panic
        let _ = configs;
    }
}
