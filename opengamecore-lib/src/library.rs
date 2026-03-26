use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstallType {
    Installer,
    Portable,
    FolderInstall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub name: String,
    pub slug: String,
    pub exe: String,
    pub install_type: InstallType,
    #[serde(default = "default_wine_config")]
    pub wine_config: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub added_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_played: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<String>,
    #[serde(default)]
    pub dxvk_enabled: bool,
}

fn default_wine_config() -> String {
    "default".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameLibrary {
    #[serde(default)]
    pub games: Vec<Game>,
}

impl GameLibrary {
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let lib: GameLibrary = toml::from_str(&content)?;
            Ok(lib)
        } else {
            Ok(GameLibrary::default())
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn add(&mut self, game: Game) {
        self.games.push(game);
    }

    pub fn remove(&mut self, slug: &str) -> Result<()> {
        let len = self.games.len();
        self.games.retain(|g| g.slug != slug);
        if self.games.len() == len {
            return Err(Error::GameNotFound(slug.into()));
        }
        Ok(())
    }

    pub fn find(&self, slug: &str) -> Option<&Game> {
        self.games.iter().find(|g| g.slug == slug)
    }

    pub fn find_mut(&mut self, slug: &str) -> Option<&mut Game> {
        self.games.iter_mut().find(|g| g.slug == slug)
    }

    pub fn recently_played(&self) -> Vec<&Game> {
        let mut played: Vec<&Game> = self.games.iter()
            .filter(|g| g.last_played.is_some())
            .collect();
        played.sort_by(|a, b| b.last_played.cmp(&a.last_played));
        played
    }
}

pub fn slugify(name: &str) -> String {
    slug::slugify(name)
}

/// Copy an image file to the icons directory, returning the destination path.
pub fn set_game_icon(slug: &str, source: &std::path::Path) -> Result<std::path::PathBuf> {
    let icons_dir = crate::paths::icons_dir()?;
    std::fs::create_dir_all(&icons_dir)?;
    let ext = source.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let dest = icons_dir.join(format!("{}.{}", slug, ext));
    std::fs::copy(source, &dest)?;
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_game(name: &str) -> Game {
        Game {
            name: name.into(),
            slug: slugify(name),
            exe: "drive_c/game/game.exe".into(),
            install_type: InstallType::Installer,
            wine_config: "default".into(),
            env: HashMap::new(),
            added_at: Utc::now(),
            last_played: None,
            icon_path: None,
            dxvk_enabled: false,
        }
    }

    #[test]
    fn add_and_find_game() {
        let mut lib = GameLibrary::default();
        lib.add(make_game("Test Game"));
        assert!(lib.find("test-game").is_some());
        assert_eq!(lib.find("test-game").unwrap().name, "Test Game");
    }

    #[test]
    fn remove_game() {
        let mut lib = GameLibrary::default();
        lib.add(make_game("Test Game"));
        lib.remove("test-game").unwrap();
        assert!(lib.find("test-game").is_none());
    }

    #[test]
    fn remove_missing_game_errors() {
        let mut lib = GameLibrary::default();
        assert!(lib.remove("nope").is_err());
    }

    #[test]
    fn round_trip_toml() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("games.toml");

        let mut lib = GameLibrary::default();
        lib.add(make_game("Cyberpunk 2077"));
        lib.save(&path).unwrap();

        let loaded = GameLibrary::load(&path).unwrap();
        assert_eq!(loaded.games.len(), 1);
        assert_eq!(loaded.games[0].slug, "cyberpunk-2077");
    }

    #[test]
    fn recently_played_sorted() {
        let mut lib = GameLibrary::default();
        let mut g1 = make_game("Old Game");
        g1.last_played = Some(Utc::now() - chrono::Duration::hours(2));
        let mut g2 = make_game("New Game");
        g2.last_played = Some(Utc::now());
        let g3 = make_game("Never Played");

        lib.add(g1);
        lib.add(g2);
        lib.add(g3);

        let recent = lib.recently_played();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].slug, "new-game");
        assert_eq!(recent[1].slug, "old-game");
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let dir = TempDir::new().unwrap();
        let lib = GameLibrary::load(&dir.path().join("nope.toml")).unwrap();
        assert!(lib.games.is_empty());
    }
}
