# Viral Features Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a compatibility database, Steam/GOG game detection, and one-click bundle auto-configuration to OpenGameCore — making it the "does this game work on Mac?" answer that drives GitHub stars.

**Architecture:** Four new lib modules (`compat.rs`, `bundle.rs`, `store_detect.rs`, `data_update.rs`) load and query committed JSON/TOML data files. The app gets a new "Game Database" screen and enhanced first-run flow with store detection. The CLI gets three new commands. All data is committed to `data/` — no scraping infrastructure in the repo.

**Tech Stack:** Rust, serde_json (new dep for compatibility.json), existing serde/toml/reqwest/tokio/iced stack

**Spec:** `docs/superpowers/specs/2026-03-28-viral-features-design.md`

---

## File Structure

```
data/
  compatibility.json                        # master compatibility database
  bundles/
    cyberpunk-2077.toml                     # example bundle config
    baldurs-gate-3.toml                     # example bundle config
    stardew-valley.toml                     # example bundle config

opengamecore-lib/
  Cargo.toml                                # add serde_json dep
  src/
    lib.rs                                  # register new modules + re-exports
    error.rs                                # add JsonParse variant
    config.rs                               # add database URL settings to AppSettings
    paths.rs                                # add data_dir path helper
    compat.rs                               # NEW — CompatDatabase, CompatEntry, CompatRating, load/query
    bundle.rs                               # NEW — BundleConfig types, load/match/apply
    store_detect.rs                         # NEW — Steam ACF parsing, GOG detection, fuzzy matching
    data_update.rs                          # NEW — fetch + cache compat data from GitHub

opengamecore-app/
  src/
    app.rs                                  # add Database screen, store detection messages, bundle messages
    views/
      mod.rs                                # register game_database
      game_database.rs                      # NEW — compatibility browser view
      add_game.rs                           # add AutoDetect tab
      first_run.rs                          # add game detection step after Wine setup
      sidebar.rs                            # add "Game Database" nav entry

opengamecore-cli/
  src/
    main.rs                                 # add detect, database, setup commands

opengamecore-lib/
  tests/
    integration.rs                          # add viral features integration tests
```

---

## Task 1: Seed Data — compatibility.json and Example Bundles

**Files:**
- Create: `data/compatibility.json`
- Create: `data/bundles/cyberpunk-2077.toml`
- Create: `data/bundles/baldurs-gate-3.toml`
- Create: `data/bundles/stardew-valley.toml`

- [ ] **Step 1: Create data directory**

```bash
mkdir -p data/bundles
```

- [ ] **Step 2: Create compatibility.json with 3 seed games**

Create `data/compatibility.json`:
```json
{
  "version": 1,
  "last_updated": "2026-03-28",
  "games": [
    {
      "name": "Cyberpunk 2077",
      "slug": "cyberpunk-2077",
      "rating": "gold",
      "confidence": 0.85,
      "sources": {
        "protondb": "gold",
        "winehq": "platinum",
        "crossover": "supported"
      },
      "recommended_backend": "gptk",
      "bundle_available": true,
      "steam_appid": 1091500,
      "gog_id": "cyberpunk_2077",
      "tags": ["dx12", "open-world", "rpg"],
      "last_updated": "2026-03-28"
    },
    {
      "name": "Baldur's Gate 3",
      "slug": "baldurs-gate-3",
      "rating": "platinum",
      "confidence": 0.95,
      "sources": {
        "protondb": "platinum",
        "winehq": "gold",
        "crossover": "supported"
      },
      "recommended_backend": "gptk",
      "bundle_available": true,
      "steam_appid": 1086940,
      "gog_id": "baldurs_gate_3",
      "tags": ["dx11", "rpg", "turn-based"],
      "last_updated": "2026-03-28"
    },
    {
      "name": "Stardew Valley",
      "slug": "stardew-valley",
      "rating": "platinum",
      "confidence": 0.98,
      "sources": {
        "protondb": "platinum",
        "winehq": "platinum",
        "crossover": "supported"
      },
      "recommended_backend": "wine",
      "bundle_available": true,
      "steam_appid": 413150,
      "gog_id": "stardew_valley",
      "tags": ["dx9", "indie", "simulation"],
      "last_updated": "2026-03-28"
    }
  ]
}
```

- [ ] **Step 3: Create bundle TOMLs**

Create `data/bundles/cyberpunk-2077.toml`:
```toml
[game]
name = "Cyberpunk 2077"
slug = "cyberpunk-2077"
rating = "gold"

[wine]
backend = "gptk"
min_version = "2.0"

[settings]
dxvk_enabled = false
env = { WINEESYNC = "1", MTL_HUD_ENABLED = "0" }

[workarounds]
notes = "Disable overlay if crashes on launch"

[install]
exe_path = "bin/x64/Cyberpunk2077.exe"
exe_alternatives = ["Cyberpunk2077.exe"]
steam_appid = 1091500
gog_id = "cyberpunk_2077"
```

Create `data/bundles/baldurs-gate-3.toml`:
```toml
[game]
name = "Baldur's Gate 3"
slug = "baldurs-gate-3"
rating = "platinum"

[wine]
backend = "gptk"
min_version = "2.0"

[settings]
dxvk_enabled = false
env = { WINEESYNC = "1" }

[install]
exe_path = "bin/bg3_dx11.exe"
exe_alternatives = ["bg3.exe", "bg3_dx11.exe"]
steam_appid = 1086940
gog_id = "baldurs_gate_3"
```

Create `data/bundles/stardew-valley.toml`:
```toml
[game]
name = "Stardew Valley"
slug = "stardew-valley"
rating = "platinum"

[wine]
backend = "wine"

[settings]
dxvk_enabled = false
env = {}

[install]
exe_path = "Stardew Valley.exe"
exe_alternatives = ["StardewValley.exe"]
steam_appid = 413150
gog_id = "stardew_valley"
```

- [ ] **Step 4: Commit**

```bash
git add data/
git commit -m "feat: add seed compatibility database and example bundles"
```

---

## Task 2: Compatibility Database Module (`compat.rs`)

**Files:**
- Modify: `opengamecore-lib/Cargo.toml` (add `serde_json`)
- Modify: `opengamecore-lib/src/error.rs` (add `JsonParse` variant)
- Modify: `opengamecore-lib/src/paths.rs` (add `compat_db_path`, `bundles_dir`)
- Create: `opengamecore-lib/src/compat.rs`
- Modify: `opengamecore-lib/src/lib.rs` (register module)

- [ ] **Step 1: Add serde_json dependency**

In `opengamecore-lib/Cargo.toml`, add to `[dependencies]`:
```toml
serde_json = "1"
```

- [ ] **Step 2: Add JsonParse error variant**

In `opengamecore-lib/src/error.rs`, add after the `TomlSerialize` variant:
```rust
#[error("JSON parse error: {0}")]
JsonParse(#[from] serde_json::Error),
```

Update `user_message()` to handle the new variant:
```rust
Error::JsonParse(_) => "Failed to parse compatibility database. It may be corrupted.".into(),
```

- [ ] **Step 3: Add path helpers**

In `opengamecore-lib/src/paths.rs`, add:
```rust
pub fn compat_db_path() -> Result<PathBuf> {
    data_dir().map(|d| d.join("compatibility.json"))
}

pub fn bundles_dir() -> Result<PathBuf> {
    data_dir().map(|d| d.join("bundles"))
}
```

- [ ] **Step 4: Write compat.rs with tests**

Create `opengamecore-lib/src/compat.rs`:
```rust
use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
            Self::Platinum => "Platinum",
            Self::Gold => "Gold",
            Self::Silver => "Silver",
            Self::Bronze => "Bronze",
            Self::Borked => "Borked",
        }
    }

    pub fn is_playable(&self) -> bool {
        !matches!(self, Self::Borked)
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
    pub steam_appid: Option<u64>,
    pub gog_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub last_updated: Option<String>,
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
        self.games.iter().find(|g| g.gog_id.as_deref() == Some(gog_id))
    }

    pub fn search(&self, query: &str) -> Vec<&CompatEntry> {
        let q = query.to_lowercase();
        self.games
            .iter()
            .filter(|g| g.name.to_lowercase().contains(&q) || g.slug.contains(&q))
            .collect()
    }

    pub fn filter_by_rating(&self, rating: CompatRating) -> Vec<&CompatEntry> {
        self.games.iter().filter(|g| g.rating == rating).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_test_db() -> String {
        r#"{
            "version": 1,
            "last_updated": "2026-03-28",
            "games": [
                {
                    "name": "Test Game",
                    "slug": "test-game",
                    "rating": "gold",
                    "confidence": 0.85,
                    "sources": { "protondb": "gold" },
                    "recommended_backend": "gptk",
                    "bundle_available": true,
                    "steam_appid": 12345,
                    "gog_id": "test_game",
                    "tags": ["dx11"],
                    "last_updated": "2026-03-28"
                },
                {
                    "name": "Borked Game",
                    "slug": "borked-game",
                    "rating": "borked",
                    "confidence": 0.9,
                    "sources": {},
                    "recommended_backend": "wine",
                    "bundle_available": false,
                    "steam_appid": null,
                    "gog_id": null,
                    "tags": [],
                    "last_updated": "2026-03-28"
                }
            ]
        }"#
        .to_string()
    }

    #[test]
    fn load_database() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("compat.json");
        std::fs::write(&path, make_test_db()).unwrap();

        let db = CompatDatabase::load(&path).unwrap();
        assert_eq!(db.games.len(), 2);
        assert_eq!(db.version, 1);
    }

    #[test]
    fn find_by_slug() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("compat.json");
        std::fs::write(&path, make_test_db()).unwrap();
        let db = CompatDatabase::load(&path).unwrap();

        assert!(db.find_by_slug("test-game").is_some());
        assert!(db.find_by_slug("nonexistent").is_none());
    }

    #[test]
    fn find_by_steam_appid() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("compat.json");
        std::fs::write(&path, make_test_db()).unwrap();
        let db = CompatDatabase::load(&path).unwrap();

        let entry = db.find_by_steam_appid(12345).unwrap();
        assert_eq!(entry.name, "Test Game");
        assert!(db.find_by_steam_appid(99999).is_none());
    }

    #[test]
    fn search_by_name() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("compat.json");
        std::fs::write(&path, make_test_db()).unwrap();
        let db = CompatDatabase::load(&path).unwrap();

        let results = db.search("test");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "test-game");
    }

    #[test]
    fn filter_by_rating() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("compat.json");
        std::fs::write(&path, make_test_db()).unwrap();
        let db = CompatDatabase::load(&path).unwrap();

        let gold = db.filter_by_rating(CompatRating::Gold);
        assert_eq!(gold.len(), 1);
        let borked = db.filter_by_rating(CompatRating::Borked);
        assert_eq!(borked.len(), 1);
    }

    #[test]
    fn rating_playable() {
        assert!(CompatRating::Platinum.is_playable());
        assert!(CompatRating::Gold.is_playable());
        assert!(CompatRating::Silver.is_playable());
        assert!(CompatRating::Bronze.is_playable());
        assert!(!CompatRating::Borked.is_playable());
    }
}
```

- [ ] **Step 5: Register module in lib.rs**

In `opengamecore-lib/src/lib.rs`, add:
```rust
pub mod compat;
```

And add to re-exports:
```rust
pub use compat::{CompatDatabase, CompatEntry, CompatRating};
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p opengamecore-lib compat`
Expected: all 6 tests pass

- [ ] **Step 7: Commit**

```bash
git add opengamecore-lib/
git commit -m "feat: add compatibility database module with JSON parsing and query"
```

---

## Task 3: Bundle Module (`bundle.rs`)

**Files:**
- Create: `opengamecore-lib/src/bundle.rs`
- Modify: `opengamecore-lib/src/lib.rs` (register module)

- [ ] **Step 1: Write bundle.rs with types, load, match, and apply**

Create `opengamecore-lib/src/bundle.rs`:
```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::library::{Game, GameLibrary, InstallType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleConfig {
    pub game: BundleGameInfo,
    pub wine: BundleWineSettings,
    pub settings: BundleSettings,
    pub workarounds: Option<BundleWorkarounds>,
    pub install: BundleInstallInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleGameInfo {
    pub name: String,
    pub slug: String,
    pub rating: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleWineSettings {
    pub backend: String,
    pub min_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleSettings {
    #[serde(default)]
    pub dxvk_enabled: bool,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleWorkarounds {
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleInstallInfo {
    pub exe_path: String,
    #[serde(default)]
    pub exe_alternatives: Vec<String>,
    pub steam_appid: Option<u64>,
    pub gog_id: Option<String>,
}

/// Load all bundle TOML files from a directory.
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

/// Find a matching bundle for a game folder by scanning for exe files.
pub fn match_bundle_for_folder(
    folder: &Path,
    bundles: &HashMap<String, BundleConfig>,
) -> Option<BundleConfig> {
    let exe_files = scan_exe_files(folder, 3);
    for exe in &exe_files {
        let exe_name = exe.file_name()?.to_string_lossy().to_lowercase();
        for bundle in bundles.values() {
            // Check primary exe_path
            let primary = Path::new(&bundle.install.exe_path)
                .file_name()
                .map(|f| f.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if exe_name == primary {
                return Some(bundle.clone());
            }
            // Check alternatives
            for alt in &bundle.install.exe_alternatives {
                if exe_name == alt.to_lowercase() {
                    return Some(bundle.clone());
                }
            }
        }
    }
    None
}

/// Recursively scan a directory for .exe files, up to max_depth.
fn scan_exe_files(dir: &Path, max_depth: usize) -> Vec<PathBuf> {
    let mut results = Vec::new();
    scan_exe_recursive(dir, max_depth, 0, &mut results);
    results
}

fn scan_exe_recursive(dir: &Path, max_depth: usize, current_depth: usize, results: &mut Vec<PathBuf>) {
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
        } else if path.extension().and_then(|e| e.to_str()) == Some("exe") {
            results.push(path);
        }
    }
}

/// Apply a bundle config to create a game entry. Returns the game slug.
pub fn apply_bundle(
    bundle: &BundleConfig,
    install_path: &Path,
    library: &mut GameLibrary,
) -> Result<String> {
    let slug = bundle.game.slug.clone();

    let game = Game {
        name: bundle.game.name.clone(),
        slug: slug.clone(),
        exe: bundle.install.exe_path.clone(),
        install_type: InstallType::Portable,
        wine_config: if bundle.wine.backend == "gptk" {
            "gptk".into()
        } else {
            "default".into()
        },
        env: bundle.settings.env.clone(),
        added_at: chrono::Utc::now(),
        last_played: None,
        icon_path: None,
        dxvk_enabled: bundle.settings.dxvk_enabled,
        use_gptk: bundle.wine.backend == "gptk",
    };

    library.add(game)?;
    Ok(slug)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_test_bundle_toml() -> &'static str {
        r#"
[game]
name = "Test Game"
slug = "test-game"
rating = "gold"

[wine]
backend = "gptk"

[settings]
dxvk_enabled = false
env = { WINEESYNC = "1" }

[install]
exe_path = "bin/TestGame.exe"
exe_alternatives = ["testgame.exe"]
steam_appid = 12345
"#
    }

    #[test]
    fn load_bundles_from_dir() {
        let dir = TempDir::new().unwrap();
        let bundles_dir = dir.path().join("bundles");
        std::fs::create_dir(&bundles_dir).unwrap();
        std::fs::write(bundles_dir.join("test-game.toml"), make_test_bundle_toml()).unwrap();

        let bundles = load_bundles(&bundles_dir).unwrap();
        assert_eq!(bundles.len(), 1);
        assert!(bundles.contains_key("test-game"));
        assert_eq!(bundles["test-game"].wine.backend, "gptk");
    }

    #[test]
    fn load_bundles_empty_dir() {
        let dir = TempDir::new().unwrap();
        let bundles = load_bundles(&dir.path().join("nonexistent")).unwrap();
        assert!(bundles.is_empty());
    }

    #[test]
    fn match_bundle_by_primary_exe() {
        let dir = TempDir::new().unwrap();
        let game_dir = dir.path().join("game");
        std::fs::create_dir_all(game_dir.join("bin")).unwrap();
        std::fs::write(game_dir.join("bin/TestGame.exe"), "fake").unwrap();

        let bundles_dir = dir.path().join("bundles");
        std::fs::create_dir(&bundles_dir).unwrap();
        std::fs::write(bundles_dir.join("test-game.toml"), make_test_bundle_toml()).unwrap();

        let bundles = load_bundles(&bundles_dir).unwrap();
        let matched = match_bundle_for_folder(&game_dir, &bundles);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().game.slug, "test-game");
    }

    #[test]
    fn match_bundle_by_alternative_exe() {
        let dir = TempDir::new().unwrap();
        let game_dir = dir.path().join("game");
        std::fs::create_dir(&game_dir).unwrap();
        std::fs::write(game_dir.join("testgame.exe"), "fake").unwrap();

        let bundles_dir = dir.path().join("bundles");
        std::fs::create_dir(&bundles_dir).unwrap();
        std::fs::write(bundles_dir.join("test-game.toml"), make_test_bundle_toml()).unwrap();

        let bundles = load_bundles(&bundles_dir).unwrap();
        let matched = match_bundle_for_folder(&game_dir, &bundles);
        assert!(matched.is_some());
    }

    #[test]
    fn no_match_when_no_exes() {
        let dir = TempDir::new().unwrap();
        let game_dir = dir.path().join("game");
        std::fs::create_dir(&game_dir).unwrap();
        std::fs::write(game_dir.join("readme.txt"), "text").unwrap();

        let bundles_dir = dir.path().join("bundles");
        std::fs::create_dir(&bundles_dir).unwrap();
        std::fs::write(bundles_dir.join("test-game.toml"), make_test_bundle_toml()).unwrap();

        let bundles = load_bundles(&bundles_dir).unwrap();
        let matched = match_bundle_for_folder(&game_dir, &bundles);
        assert!(matched.is_none());
    }

    #[test]
    fn apply_bundle_creates_game() {
        let dir = TempDir::new().unwrap();
        let bundles_dir = dir.path().join("bundles");
        std::fs::create_dir(&bundles_dir).unwrap();
        std::fs::write(bundles_dir.join("test-game.toml"), make_test_bundle_toml()).unwrap();

        let bundles = load_bundles(&bundles_dir).unwrap();
        let bundle = &bundles["test-game"];

        let mut library = GameLibrary::default();
        let slug = apply_bundle(bundle, dir.path(), &mut library).unwrap();

        assert_eq!(slug, "test-game");
        let game = library.find("test-game").unwrap();
        assert_eq!(game.wine_config, "gptk");
        assert!(game.use_gptk);
        assert!(!game.dxvk_enabled);
        assert_eq!(game.env.get("WINEESYNC").unwrap(), "1");
    }
}
```

- [ ] **Step 2: Register module**

In `opengamecore-lib/src/lib.rs`, add `pub mod bundle;` and re-export:
```rust
pub use bundle::BundleConfig;
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p opengamecore-lib bundle`
Expected: all 6 tests pass

- [ ] **Step 4: Commit**

```bash
git add opengamecore-lib/
git commit -m "feat: add bundle config loading, exe matching, and auto-apply"
```

---

## Task 4: Store Detection Module (`store_detect.rs`)

**Files:**
- Create: `opengamecore-lib/src/store_detect.rs`
- Modify: `opengamecore-lib/src/lib.rs`

- [ ] **Step 1: Write store_detect.rs**

Create `opengamecore-lib/src/store_detect.rs` with:

- `GameStore` enum (Steam, Gog)
- `DetectedGame` struct
- `detect_steam_games()` — parses ACF files from `~/Library/Application Support/Steam/steamapps/`
- `detect_gog_games()` — scans `/Applications/GOG Games/` and `~/GOG Games/`
- `detect_installed_games()` — combines both, matches against `CompatDatabase`
- `parse_acf()` — minimal Valve KeyValues parser for appmanifest ACF files (extract `appid`, `name`, `installdir`)
- Inline tests using mock ACF files in tempdir

ACF parsing: these are simple key-value files like:
```
"AppState"
{
    "appid"     "1091500"
    "name"      "Cyberpunk 2077"
    "installdir"        "Cyberpunk 2077"
}
```

A minimal parser that extracts quoted key-value pairs from lines matching `"key"\s+"value"` is sufficient. No need for a full recursive parser.

Tests should cover:
- `parse_acf` with mock ACF content
- `detect_steam_games` with mock steamapps directory
- `detect_gog_games` with mock GOG directory
- `detect_installed_games` combining both with compat DB matching

- [ ] **Step 2: Register module**

In `opengamecore-lib/src/lib.rs`, add `pub mod store_detect;`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p opengamecore-lib store_detect`
Expected: all tests pass

- [ ] **Step 4: Commit**

```bash
git add opengamecore-lib/
git commit -m "feat: add Steam ACF and GOG game detection with compat matching"
```

---

## Task 5: Data Update Module (`data_update.rs`)

**Files:**
- Create: `opengamecore-lib/src/data_update.rs`
- Modify: `opengamecore-lib/src/config.rs` (add database URL settings)
- Modify: `opengamecore-lib/src/lib.rs`

- [ ] **Step 1: Add config fields**

In `opengamecore-lib/src/config.rs`, add to `AppSettings`:
```rust
#[serde(default = "default_auto_update")]
pub auto_update_database: bool,

#[serde(default = "default_database_url")]
pub database_url: String,

#[serde(default = "default_bundles_url")]
pub bundles_url: String,
```

Add default functions:
```rust
fn default_auto_update() -> bool { true }

fn default_database_url() -> String {
    "https://raw.githubusercontent.com/user/opengamecore/main/data/compatibility.json".into()
}

fn default_bundles_url() -> String {
    "https://raw.githubusercontent.com/user/opengamecore/main/data/bundles/".into()
}
```

Update `Default for AppSettings` to include the new fields.

- [ ] **Step 2: Write data_update.rs**

Create `opengamecore-lib/src/data_update.rs`:
```rust
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
```

- [ ] **Step 3: Register module**

In `opengamecore-lib/src/lib.rs`, add `pub mod data_update;`.

- [ ] **Step 4: Run tests**

Run: `cargo test -p opengamecore-lib data_update`
Expected: 2 tests pass

- [ ] **Step 5: Commit**

```bash
git add opengamecore-lib/
git commit -m "feat: add database auto-update with staleness check and atomic fetch"
```

---

## Task 6: Game Database View (`game_database.rs`)

**Files:**
- Create: `opengamecore-app/src/views/game_database.rs`
- Modify: `opengamecore-app/src/views/mod.rs`

- [ ] **Step 1: Write game_database.rs**

A new iced view that shows the compatibility database as a searchable, filterable list. Takes `&CompatDatabase`, `search_query: &str`, `filter_rating: Option<CompatRating>`. Shows color-coded rating badges (Platinum=green, Gold=yellow, Silver=gray, Bronze=orange, Borked=red). "Add" button on rows with `bundle_available == true`.

Messages needed (to be wired in Task 8):
- `SearchChanged(String)`
- `FilterRating(Option<CompatRating>)`
- `SetupFromDatabase(String)` — slug of game to set up from bundle

- [ ] **Step 2: Register in views/mod.rs**

Add `pub mod game_database;`

- [ ] **Step 3: Verify build**

Run: `cargo build -p opengamecore-app`
Expected: compiles (warnings OK for unused items)

- [ ] **Step 4: Commit**

```bash
git add opengamecore-app/src/views/
git commit -m "feat: add game database compatibility browser view"
```

---

## Task 7: Sidebar + Add Game Auto-Detect Updates

**Files:**
- Modify: `opengamecore-app/src/views/sidebar.rs` (add Database entry)
- Modify: `opengamecore-app/src/views/add_game.rs` (add AutoDetect tab)

- [ ] **Step 1: Add "Game Database" to sidebar**

In `sidebar.rs`, add a fourth nav button for `Screen::Database` between "All Games" and "Bottles".

- [ ] **Step 2: Add AutoDetect tab to add_game.rs**

Add `AutoDetect` variant to `AddGameTab`. In the AutoDetect tab view, show a "Select Folder" button. After selection, if a bundle match is found, display the bundle info (name, rating badge, backend, env vars) and an "Add with Recommended Settings" button. If no match, show "No compatible game detected" with a link to switch to manual tabs.

Add `matched_bundle: Option<BundleConfig>` to `AddGameState`.

- [ ] **Step 3: Verify build**

Run: `cargo build -p opengamecore-app`

- [ ] **Step 4: Commit**

```bash
git add opengamecore-app/src/views/
git commit -m "feat: add database sidebar entry and auto-detect tab in add game dialog"
```

---

## Task 8: Wire Everything into App State (`app.rs`)

**Files:**
- Modify: `opengamecore-app/src/app.rs`

This is the integration task — add all new state, messages, and handlers.

- [ ] **Step 1: Add Screen::Database variant**

- [ ] **Step 2: Add new state fields to App**

```rust
compat_db: Option<CompatDatabase>,
bundles: HashMap<String, BundleConfig>,
detected_games: Vec<DetectedGame>,
db_search_query: String,
db_filter_rating: Option<CompatRating>,
```

- [ ] **Step 3: Add new messages**

```rust
// Database
SearchChanged(String),
FilterRating(Option<CompatRating>),
SetupFromDatabase(String),
SetupFolderSelected(String, Option<String>),  // slug, folder path

// Store detection
DetectGames,
GamesDetected(Vec<DetectedGame>),

// Bundle
ApplyBundle(String),  // slug
BundleApplied(String),

// Auto-detect in add game
AutoDetectFolder,
AutoDetectResult(Option<BundleConfig>),

// Data update
DatabaseUpdated(bool),
```

- [ ] **Step 4: Load compat DB and bundles in App::new()**

In the startup async task, after loading config/library, also load:
- `compat_db` from `data/compatibility.json` (shipped with app, or from data_dir)
- `bundles` from `data/bundles/`
- Trigger `check_and_update` if auto-update enabled

- [ ] **Step 5: Wire message handlers**

- `SearchChanged` → update `db_search_query`
- `FilterRating` → update `db_filter_rating`
- `SetupFromDatabase(slug)` → open folder picker, then `SetupFolderSelected`
- `SetupFolderSelected(slug, Some(path))` → look up bundle, apply, create bottle, reload library
- `DetectGames` → `Task::perform` calling `store_detect::detect_installed_games`
- `GamesDetected` → store results, display in first-run or library
- `AutoDetectFolder` → open folder picker, scan for match, store in `add_game.matched_bundle`

- [ ] **Step 6: Update view() routing**

Add `Screen::Database` match arm calling `game_database::view(...)`.

- [ ] **Step 7: Verify build**

Run: `cargo build -p opengamecore-app`

- [ ] **Step 8: Commit**

```bash
git add opengamecore-app/
git commit -m "feat: wire compatibility database, store detection, and bundles into app"
```

---

## Task 9: First-Run Flow — Game Detection Step

**Files:**
- Modify: `opengamecore-app/src/views/first_run.rs`
- Modify: `opengamecore-app/src/app.rs`

- [ ] **Step 1: Add DetectingGames phase**

Add to `FirstRunPhase`:
```rust
DetectingGames,
GamesFound { detected: Vec<DetectedGame> },
```

- [ ] **Step 2: Update first_run.rs view**

- `DetectingGames` — spinner with "Scanning for installed games..."
- `GamesFound { detected }` — list of detected games with ratings, "Set Up" buttons for bundled games, "Continue to Library" button

- [ ] **Step 3: Wire in app.rs**

After `FirstRunComplete`, transition to `DetectingGames`, trigger store detection. When `GamesDetected` arrives during first-run, transition to `GamesFound`.

- [ ] **Step 4: Verify build and test flow**

Run: `cargo build -p opengamecore-app`

- [ ] **Step 5: Commit**

```bash
git add opengamecore-app/
git commit -m "feat: add game detection step to first-run flow"
```

---

## Task 10: CLI Commands — detect, database, setup

**Files:**
- Modify: `opengamecore-cli/src/main.rs`

- [ ] **Step 1: Add Detect command**

```rust
/// Scan Steam and GOG for installed games and show compatibility
Detect,
```

Handler: load compat DB, call `detect_installed_games`, print table of detected games with ratings.

- [ ] **Step 2: Add Database command**

```rust
/// Search the compatibility database
Database {
    /// Search query
    #[arg(default_value = "")]
    query: String,

    /// Filter by rating
    #[arg(short, long)]
    rating: Option<String>,
},
```

Handler: load compat DB, search/filter, print results.

- [ ] **Step 3: Add Setup command**

```rust
/// Auto-configure a game from its bundle
Setup {
    /// Game slug from the database
    slug: String,

    /// Path to game folder
    #[arg(short, long)]
    path: Option<PathBuf>,
},
```

Handler: load bundle for slug, apply to library, create bottle, print success.

- [ ] **Step 4: Verify build**

Run: `cargo build -p opengamecore-cli`

- [ ] **Step 5: Test CLI commands**

```bash
cargo run -p opengamecore-cli -- detect
cargo run -p opengamecore-cli -- database "cyberpunk"
cargo run -p opengamecore-cli -- setup stardew-valley --path /tmp/fake-game
```

- [ ] **Step 6: Commit**

```bash
git add opengamecore-cli/
git commit -m "feat: add detect, database, and setup CLI commands"
```

---

## Task 11: Integration Tests

**Files:**
- Modify: `opengamecore-lib/tests/integration.rs`

- [ ] **Step 1: Add compat database integration test**

Test: load `data/compatibility.json` from the repo, verify it parses, search works.

- [ ] **Step 2: Add bundle + library integration test**

Test: load bundles from `data/bundles/`, match a fake game folder against them, apply bundle, verify game entry is created correctly.

- [ ] **Step 3: Add store detection integration test**

Test: create mock Steam ACF files in tempdir, run detect, verify games matched against compat DB.

- [ ] **Step 4: Add full pipeline test**

Test: detect mock Steam game → find matching bundle → apply bundle → verify game in library with correct settings.

- [ ] **Step 5: Run all tests**

Run: `cargo test --workspace`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add opengamecore-lib/tests/
git commit -m "test: add integration tests for compatibility, bundles, and store detection"
```

---

## Task 12: GitHub Discussions Template + README Update

**Files:**
- Create: `.github/DISCUSSION_TEMPLATE/compatibility-report.yml`
- Modify: `README.md`

- [ ] **Step 1: Create discussion template**

Create `.github/DISCUSSION_TEMPLATE/compatibility-report.yml`:
```yaml
title: "[Compat] "
labels: ["compatibility-report"]
body:
  - type: input
    id: game
    attributes:
      label: Game Name
    validations:
      required: true
  - type: dropdown
    id: rating
    attributes:
      label: Rating
      options:
        - Platinum (works perfectly)
        - Gold (minor tweaks needed)
        - Silver (playable with workarounds)
        - Bronze (runs but significant issues)
        - Borked (does not work)
    validations:
      required: true
  - type: input
    id: macos_version
    attributes:
      label: macOS Version
      placeholder: "e.g., 14.3"
    validations:
      required: true
  - type: input
    id: chip
    attributes:
      label: Apple Chip
      placeholder: "e.g., M3 Pro"
    validations:
      required: true
  - type: dropdown
    id: backend
    attributes:
      label: Wine Backend
      options:
        - Wine
        - GPTK
        - CrossOver
    validations:
      required: true
  - type: textarea
    id: notes
    attributes:
      label: Notes
      description: "Any workarounds, tweaks, or issues"
```

- [ ] **Step 2: Update README**

Add a "Game Compatibility" section to README with:
- Link to the compatibility database
- How to submit reports via Discussions
- How to contribute bundles via PRs

- [ ] **Step 3: Commit**

```bash
git add .github/ README.md
git commit -m "docs: add compatibility report template and update README"
```

---

## Task 13: Final Polish — CI Validation for Bundles

**Files:**
- Create: `.github/workflows/validate-bundles.yml`

- [ ] **Step 1: Add CI workflow**

Create `.github/workflows/validate-bundles.yml` that validates:
- `data/compatibility.json` is valid JSON and matches expected schema
- All TOML files in `data/bundles/` parse successfully
- Every bundle slug referenced in `compatibility.json` with `bundle_available: true` has a corresponding TOML file

Runs on push/PR when `data/` files change.

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/
git commit -m "ci: add bundle validation workflow for data/ directory"
```
