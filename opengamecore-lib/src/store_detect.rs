use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::compat::{CompatDatabase, CompatRating};
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameStore {
    Steam,
    Gog,
}

#[derive(Debug, Clone)]
pub struct DetectedGame {
    pub name: String,
    pub store: GameStore,
    pub install_path: PathBuf,
    pub exe_path: Option<PathBuf>,
    pub rating: Option<CompatRating>,
    pub bundle_available: bool,
}

/// Parse a Valve ACF (Application Cache File) to extract (appid, name, installdir).
/// ACF files use a KeyValues format with `"key"\t\t"value"` lines.
pub fn parse_acf(content: &str) -> Option<(u64, String, String)> {
    let mut appid: Option<u64> = None;
    let mut name: Option<String> = None;
    let mut installdir: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        // Match lines like: "key"		"value"
        let parts: Vec<&str> = trimmed
            .split('"')
            .filter(|s| !s.trim().is_empty())
            .collect();
        if parts.len() >= 2 {
            let key = parts[0].to_lowercase();
            let value = parts[1].to_string();
            match key.as_str() {
                "appid" => appid = value.parse().ok(),
                "name" => name = Some(value),
                "installdir" => installdir = Some(value),
                _ => {}
            }
        }
    }

    match (appid, name, installdir) {
        (Some(a), Some(n), Some(d)) => Some((a, n, d)),
        _ => None,
    }
}

/// Detect Steam games by scanning steamapps directory for ACF manifests.
pub fn detect_steam_games(
    steamapps_dir: &Path,
    compat_db: &CompatDatabase,
) -> Result<Vec<DetectedGame>> {
    let mut games = Vec::new();
    if !steamapps_dir.exists() {
        return Ok(games);
    }

    for entry in std::fs::read_dir(steamapps_dir)? {
        let entry = entry?;
        let path = entry.path();
        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        if filename.starts_with("appmanifest_") && filename.ends_with(".acf") {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            if let Some((appid, name, installdir)) = parse_acf(&content) {
                let install_path = steamapps_dir.join("common").join(&installdir);
                let compat_entry = compat_db.find_by_steam_appid(appid);

                games.push(DetectedGame {
                    name,
                    store: GameStore::Steam,
                    install_path,
                    exe_path: None,
                    rating: compat_entry.map(|e| e.rating.clone()),
                    bundle_available: compat_entry.map(|e| e.bundle_available).unwrap_or(false),
                });
            }
        }
    }
    Ok(games)
}

/// Detect GOG games by scanning known GOG directories.
pub fn detect_gog_games(
    gog_dirs: &[PathBuf],
    compat_db: &CompatDatabase,
) -> Result<Vec<DetectedGame>> {
    let mut games = Vec::new();

    for gog_dir in gog_dirs {
        if !gog_dir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(gog_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let folder_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            // Try to match by fuzzy slug: convert folder name to slug and compare
            let folder_slug = slug::slugify(&folder_name);
            let compat_entry = compat_db.find_by_slug(&folder_slug).or_else(|| {
                // Also try matching gog_id
                let folder_id = folder_name.to_lowercase().replace(' ', "_");
                compat_db.find_by_gog_id(&folder_id)
            });

            games.push(DetectedGame {
                name: folder_name,
                store: GameStore::Gog,
                install_path: path,
                exe_path: None,
                rating: compat_entry.map(|e| e.rating.clone()),
                bundle_available: compat_entry.map(|e| e.bundle_available).unwrap_or(false),
            });
        }
    }
    Ok(games)
}

/// Detect all installed games from Steam and GOG.
pub fn detect_installed_games(compat_db: &CompatDatabase) -> Result<Vec<DetectedGame>> {
    let mut all = Vec::new();

    // Steam: ~/Library/Application Support/Steam/steamapps/
    if let Some(home) = dirs::home_dir() {
        let steamapps = home.join("Library/Application Support/Steam/steamapps");
        all.extend(detect_steam_games(&steamapps, compat_db)?);
    }

    // GOG: /Applications/GOG Games/ and ~/GOG Games/
    let mut gog_dirs = vec![PathBuf::from("/Applications/GOG Games")];
    if let Some(home) = dirs::home_dir() {
        gog_dirs.push(home.join("GOG Games"));
    }
    all.extend(detect_gog_games(&gog_dirs, compat_db)?);

    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compat::{CompatEntry, CompatRating};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn mock_compat_db() -> CompatDatabase {
        CompatDatabase {
            version: 1,
            last_updated: "2026-01-01".into(),
            games: vec![
                CompatEntry {
                    name: "Cyberpunk 2077".into(),
                    slug: "cyberpunk-2077".into(),
                    rating: CompatRating::Gold,
                    confidence: 0.85,
                    sources: HashMap::new(),
                    recommended_backend: "gptk".into(),
                    bundle_available: true,
                    steam_appid: Some(1091500),
                    gog_id: Some("cyberpunk_2077".into()),
                    tags: vec![],
                    last_updated: "2026-01-01".into(),
                },
                CompatEntry {
                    name: "Stardew Valley".into(),
                    slug: "stardew-valley".into(),
                    rating: CompatRating::Platinum,
                    confidence: 0.98,
                    sources: HashMap::new(),
                    recommended_backend: "wine".into(),
                    bundle_available: true,
                    steam_appid: Some(413150),
                    gog_id: Some("stardew_valley".into()),
                    tags: vec![],
                    last_updated: "2026-01-01".into(),
                },
            ],
        }
    }

    #[test]
    fn parse_acf_basic() {
        let content = r#"
"AppState"
{
	"appid"		"1091500"
	"Universe"		"1"
	"name"		"Cyberpunk 2077"
	"StateFlags"		"4"
	"installdir"		"Cyberpunk 2077"
}
"#;
        let (appid, name, installdir) = parse_acf(content).unwrap();
        assert_eq!(appid, 1091500);
        assert_eq!(name, "Cyberpunk 2077");
        assert_eq!(installdir, "Cyberpunk 2077");
    }

    #[test]
    fn parse_acf_missing_fields() {
        let content = r#"
"AppState"
{
	"appid"		"1234"
}
"#;
        assert!(parse_acf(content).is_none());
    }

    #[test]
    fn detect_steam_with_mock() {
        let dir = TempDir::new().unwrap();
        let steamapps = dir.path();

        // Create a mock ACF file
        let acf = r#"
"AppState"
{
	"appid"		"1091500"
	"name"		"Cyberpunk 2077"
	"installdir"		"Cyberpunk 2077"
}
"#;
        std::fs::write(steamapps.join("appmanifest_1091500.acf"), acf).unwrap();
        std::fs::create_dir_all(steamapps.join("common/Cyberpunk 2077")).unwrap();

        let db = mock_compat_db();
        let games = detect_steam_games(steamapps, &db).unwrap();

        assert_eq!(games.len(), 1);
        assert_eq!(games[0].name, "Cyberpunk 2077");
        assert_eq!(games[0].store, GameStore::Steam);
        assert_eq!(games[0].rating, Some(CompatRating::Gold));
        assert!(games[0].bundle_available);
    }

    #[test]
    fn detect_gog_with_mock() {
        let dir = TempDir::new().unwrap();
        let gog_dir = dir.path().join("GOG Games");
        std::fs::create_dir_all(gog_dir.join("Stardew Valley")).unwrap();

        let db = mock_compat_db();
        let games = detect_gog_games(&[gog_dir], &db).unwrap();

        assert_eq!(games.len(), 1);
        assert_eq!(games[0].name, "Stardew Valley");
        assert_eq!(games[0].store, GameStore::Gog);
        assert_eq!(games[0].rating, Some(CompatRating::Platinum));
        assert!(games[0].bundle_available);
    }

    #[test]
    fn detect_empty_dirs() {
        let dir = TempDir::new().unwrap();
        let db = mock_compat_db();

        let steam = detect_steam_games(&dir.path().join("nonexistent"), &db).unwrap();
        assert!(steam.is_empty());

        let gog = detect_gog_games(&[dir.path().join("nonexistent")], &db).unwrap();
        assert!(gog.is_empty());
    }
}
