use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompatRating {
    Platinum,
    Gold,
    Silver,
    Bronze,
    Borked,
}

impl CompatRating {
    pub fn label(&self) -> &'static str {
        match self {
            CompatRating::Platinum => "Platinum",
            CompatRating::Gold => "Gold",
            CompatRating::Silver => "Silver",
            CompatRating::Bronze => "Bronze",
            CompatRating::Borked => "Borked",
        }
    }

    pub fn is_playable(&self) -> bool {
        matches!(
            self,
            CompatRating::Platinum | CompatRating::Gold | CompatRating::Silver
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatEntry {
    pub name: String,
    pub slug: String,
    pub rating: CompatRating,
    pub confidence: f64,
    #[serde(default)]
    pub sources: HashMap<String, String>,
    #[serde(default)]
    pub recommended_backend: String,
    #[serde(default)]
    pub bundle_available: bool,
    #[serde(default)]
    pub steam_appid: Option<u64>,
    #[serde(default)]
    pub gog_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatDatabase {
    pub version: u32,
    pub last_updated: String,
    pub games: Vec<CompatEntry>,
}

impl CompatDatabase {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let db: CompatDatabase = serde_json::from_str(&content)?;
        Ok(db)
    }

    pub fn find_by_slug(&self, slug: &str) -> Option<&CompatEntry> {
        self.games.iter().find(|g| g.slug == slug)
    }

    pub fn find_by_steam_appid(&self, appid: u64) -> Option<&CompatEntry> {
        self.games.iter().find(|g| g.steam_appid == Some(appid))
    }

    pub fn find_by_gog_id(&self, gog_id: &str) -> Option<&CompatEntry> {
        self.games
            .iter()
            .find(|g| g.gog_id.as_deref() == Some(gog_id))
    }

    pub fn search(&self, query: &str) -> Vec<&CompatEntry> {
        let query_lower = query.to_lowercase();
        self.games
            .iter()
            .filter(|g| {
                g.name.to_lowercase().contains(&query_lower) || g.slug.contains(&query_lower)
            })
            .collect()
    }

    pub fn filter_by_rating(&self, rating: &CompatRating) -> Vec<&CompatEntry> {
        self.games.iter().filter(|g| &g.rating == rating).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn db_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("data/compatibility.json")
    }

    #[test]
    fn load_compatibility_db() {
        let db = CompatDatabase::load(&db_path()).unwrap();
        assert!(db.version >= 1);
        assert!(!db.games.is_empty());
        assert_eq!(db.games.len(), 5);
    }

    #[test]
    fn find_by_slug() {
        let db = CompatDatabase::load(&db_path()).unwrap();
        let entry = db.find_by_slug("cyberpunk-2077").unwrap();
        assert_eq!(entry.name, "Cyberpunk 2077");
        assert_eq!(entry.rating, CompatRating::Gold);
    }

    #[test]
    fn find_by_steam_appid() {
        let db = CompatDatabase::load(&db_path()).unwrap();
        let entry = db.find_by_steam_appid(1091500).unwrap();
        assert_eq!(entry.slug, "cyberpunk-2077");
        assert!(db.find_by_steam_appid(9999999).is_none());
    }

    #[test]
    fn search_games() {
        let db = CompatDatabase::load(&db_path()).unwrap();
        let results = db.search("witcher");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "the-witcher-3-wild-hunt");

        let results = db.search("ELDEN");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn filter_by_rating() {
        let db = CompatDatabase::load(&db_path()).unwrap();
        let platinum = db.filter_by_rating(&CompatRating::Platinum);
        assert!(platinum.len() >= 2);
        for entry in &platinum {
            assert_eq!(entry.rating, CompatRating::Platinum);
        }
    }

    #[test]
    fn rating_playable() {
        assert!(CompatRating::Platinum.is_playable());
        assert!(CompatRating::Gold.is_playable());
        assert!(CompatRating::Silver.is_playable());
        assert!(!CompatRating::Bronze.is_playable());
        assert!(!CompatRating::Borked.is_playable());

        assert_eq!(CompatRating::Platinum.label(), "Platinum");
        assert_eq!(CompatRating::Borked.label(), "Borked");
    }
}
