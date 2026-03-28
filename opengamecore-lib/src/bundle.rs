use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::library::{Game, GameLibrary, InstallType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleConfig {
    pub game: BundleGameInfo,
    #[serde(default)]
    pub wine: BundleWineSettings,
    #[serde(default)]
    pub settings: BundleSettings,
    #[serde(default)]
    pub workarounds: BundleWorkarounds,
    #[serde(default)]
    pub install: BundleInstallInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleGameInfo {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub rating: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleWineSettings {
    #[serde(default)]
    pub backend: String,
    #[serde(default)]
    pub min_version: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleSettings {
    #[serde(default)]
    pub dxvk_enabled: bool,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleWorkarounds {
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleInstallInfo {
    #[serde(default)]
    pub exe_path: String,
    #[serde(default)]
    pub exe_alternatives: Vec<String>,
    #[serde(default)]
    pub steam_appid: Option<u64>,
    #[serde(default)]
    pub gog_id: Option<String>,
}

/// Load all bundle configs from a directory of .toml files.
/// Returns a map of slug -> BundleConfig.
pub fn load_bundles(bundles_dir: &Path) -> Result<HashMap<String, BundleConfig>> {
    let mut bundles = HashMap::new();
    if !bundles_dir.exists() {
        return Ok(bundles);
    }
    for entry in std::fs::read_dir(bundles_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            let content = std::fs::read_to_string(&path)?;
            let bundle: BundleConfig = toml::from_str(&content)?;
            bundles.insert(bundle.game.slug.clone(), bundle);
        }
    }
    Ok(bundles)
}

/// Scan a directory recursively for .exe files, up to max_depth levels.
pub fn scan_exe_files(dir: &Path, max_depth: u32) -> Vec<PathBuf> {
    let mut results = Vec::new();
    scan_exe_recursive(dir, max_depth, 0, &mut results);
    results
}

fn scan_exe_recursive(dir: &Path, max_depth: u32, current_depth: u32, results: &mut Vec<PathBuf>) {
    if current_depth > max_depth {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_exe_recursive(&path, max_depth, current_depth + 1, results);
        } else if let Some(ext) = path.extension() {
            if ext.to_ascii_lowercase() == "exe" {
                results.push(path);
            }
        }
    }
}

/// Match a bundle for a given game folder by scanning for .exe files and
/// comparing against bundle exe_path and exe_alternatives (case-insensitive).
pub fn match_bundle_for_folder(
    folder: &Path,
    bundles: &HashMap<String, BundleConfig>,
) -> Option<BundleConfig> {
    let exe_files = scan_exe_files(folder, 3);
    let exe_names: Vec<String> = exe_files
        .iter()
        .filter_map(|p| p.file_name())
        .filter_map(|n| n.to_str())
        .map(|s| s.to_lowercase())
        .collect();

    for bundle in bundles.values() {
        // Check primary exe_path (just the filename part)
        let primary = Path::new(&bundle.install.exe_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !primary.is_empty() && exe_names.contains(&primary) {
            return Some(bundle.clone());
        }

        // Check alternatives
        for alt in &bundle.install.exe_alternatives {
            let alt_lower = alt.to_lowercase();
            if exe_names.contains(&alt_lower) {
                return Some(bundle.clone());
            }
        }
    }
    None
}

/// Apply a bundle config to create a Game entry in the library.
/// Returns the slug of the created game.
pub fn apply_bundle(
    bundle: &BundleConfig,
    install_path: &Path,
    library: &mut GameLibrary,
) -> Result<String> {
    let exe = if bundle.install.exe_path.is_empty() {
        "game.exe".into()
    } else {
        bundle.install.exe_path.clone()
    };

    let use_gptk = bundle.wine.backend.to_lowercase() == "gptk";

    let game = Game {
        name: bundle.game.name.clone(),
        slug: bundle.game.slug.clone(),
        exe: format!("drive_c/{}", exe),
        install_type: InstallType::FolderInstall,
        wine_config: if use_gptk {
            "gptk".into()
        } else {
            "default".into()
        },
        env: bundle.settings.env.clone(),
        added_at: chrono::Utc::now(),
        last_played: None,
        icon_path: None,
        dxvk_enabled: bundle.settings.dxvk_enabled,
        use_gptk,
    };

    let slug = game.slug.clone();
    library.add(game)?;

    // Create symlink or note about install_path (the bottle setup would handle this)
    let _ = install_path; // used by caller for bottle setup

    Ok(slug)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn bundles_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("data/bundles")
    }

    #[test]
    fn load_bundles_from_data() {
        let bundles = load_bundles(&bundles_path()).unwrap();
        assert_eq!(bundles.len(), 5);
        assert!(bundles.contains_key("cyberpunk-2077"));
        assert!(bundles.contains_key("stardew-valley"));

        let cp = &bundles["cyberpunk-2077"];
        assert_eq!(cp.game.name, "Cyberpunk 2077");
        assert_eq!(cp.install.exe_path, "bin/x64/Cyberpunk2077.exe");
    }

    #[test]
    fn load_bundles_empty_dir() {
        let dir = TempDir::new().unwrap();
        let bundles = load_bundles(dir.path()).unwrap();
        assert!(bundles.is_empty());
    }

    #[test]
    fn match_primary_exe() {
        let dir = TempDir::new().unwrap();
        // Create nested exe matching Cyberpunk
        let exe_dir = dir.path().join("bin").join("x64");
        std::fs::create_dir_all(&exe_dir).unwrap();
        std::fs::write(exe_dir.join("Cyberpunk2077.exe"), "fake").unwrap();

        let bundles = load_bundles(&bundles_path()).unwrap();
        let matched = match_bundle_for_folder(dir.path(), &bundles).unwrap();
        assert_eq!(matched.game.slug, "cyberpunk-2077");
    }

    #[test]
    fn match_alternative_exe() {
        let dir = TempDir::new().unwrap();
        // Create an exe matching an alternative name
        std::fs::write(dir.path().join("StardewValley.exe"), "fake").unwrap();

        let bundles = load_bundles(&bundles_path()).unwrap();
        let matched = match_bundle_for_folder(dir.path(), &bundles).unwrap();
        assert_eq!(matched.game.slug, "stardew-valley");
    }

    #[test]
    fn no_match_for_unknown_exe() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("unknown_game.exe"), "fake").unwrap();

        let bundles = load_bundles(&bundles_path()).unwrap();
        let matched = match_bundle_for_folder(dir.path(), &bundles);
        assert!(matched.is_none());
    }

    #[test]
    fn apply_bundle_creates_game() {
        let bundles = load_bundles(&bundles_path()).unwrap();
        let bundle = &bundles["cyberpunk-2077"];

        let mut library = GameLibrary::default();
        let slug = apply_bundle(bundle, Path::new("/tmp/fake"), &mut library).unwrap();

        assert_eq!(slug, "cyberpunk-2077");
        let game = library.find("cyberpunk-2077").unwrap();
        assert_eq!(game.name, "Cyberpunk 2077");
        assert!(game.use_gptk);
        assert_eq!(game.wine_config, "gptk");
        assert!(game.exe.contains("Cyberpunk2077.exe"));
    }
}
