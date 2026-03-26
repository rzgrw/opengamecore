# OpenGameCore Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a macOS Wine game launcher — two-crate Rust workspace with iced GUI, Wine management, APFS bottle cloning, and sidebar+grid game library.

**Architecture:** Cargo workspace with `opengamecore-lib` (core logic, no UI) and `opengamecore-app` (iced GUI). The lib handles Wine discovery/download, bottle management (APFS clonefile), game library (TOML), and game launching. The app provides sidebar+grid layout with first-run flow, game management, and settings.

**Tech Stack:** Rust, iced (GUI), tokio (async), serde + toml (serialization), dirs (platform paths), reqwest (HTTP downloads)

**Spec:** `docs/superpowers/specs/2026-03-25-opengamecore-design.md`

---

## File Structure

```
Cargo.toml                          # workspace root
opengamecore-lib/
  Cargo.toml
  src/
    lib.rs                          # public API re-exports
    error.rs                        # Error enum for all lib operations
    paths.rs                        # platform path resolution (~Library/Application Support/...)
    config.rs                       # AppConfig, WineConfig structs, load/save config.toml
    bottle.rs                       # bottle create (APFS clonefile), list, delete, reset
    library.rs                      # Game struct, GameLibrary CRUD on games.toml
    wine.rs                         # Wine discovery, download, extraction
    runner.rs                       # assemble Wine+prefix+exe, spawn process, capture output
opengamecore-app/
  Cargo.toml
  src/
    main.rs                         # entry point, launches iced app
    app.rs                          # top-level App struct, Message enum, update/view
    theme.rs                        # dark color palette, iced styling
    views/
      mod.rs                        # re-exports
      sidebar.rs                    # sidebar navigation component
      game_grid.rs                  # game card grid + play button
      add_game.rs                   # add game dialog (3 modes)
      bottle_detail.rs              # bottle info + reset/delete actions
      settings.rs                   # Wine management, download URLs
      first_run.rs                  # first-run Wine download + template creation
```

---

## Task 1: Workspace Scaffold & CI Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `opengamecore-lib/Cargo.toml`
- Create: `opengamecore-lib/src/lib.rs`
- Create: `opengamecore-app/Cargo.toml`
- Create: `opengamecore-app/src/main.rs`
- Create: `.gitignore`

- [ ] **Step 1: Initialize git repo**

```bash
cd /Users/rz/Opengamecore
git init
```

- [ ] **Step 2: Create workspace Cargo.toml**

Create `Cargo.toml`:
```toml
[workspace]
members = ["opengamecore-lib", "opengamecore-app"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
```

- [ ] **Step 3: Create opengamecore-lib crate**

Create `opengamecore-lib/Cargo.toml`:
```toml
[package]
name = "opengamecore-lib"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
toml = "0.8"
dirs = "6"
tokio = { version = "1", features = ["process", "fs", "rt-multi-thread", "macros"] }
reqwest = { version = "0.12", features = ["stream"] }
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
slug = "0.1"
futures-util = "0.3"

[dev-dependencies]
tempfile = "3"
tokio = { version = "1", features = ["test-util"] }
```

Create `opengamecore-lib/src/lib.rs`:
```rust
pub mod config;
pub mod error;
pub mod paths;
pub mod bottle;
pub mod library;
pub mod wine;
pub mod runner;
```

- [ ] **Step 4: Create opengamecore-app crate**

Create `opengamecore-app/Cargo.toml`:
```toml
[package]
name = "opengamecore-app"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
opengamecore-lib = { path = "../opengamecore-lib" }
iced = { version = "0.13", features = ["tokio"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
rfd = "0.15"
```

Create `opengamecore-app/src/main.rs`:
```rust
fn main() {
    println!("OpenGameCore");
}
```

- [ ] **Step 5: Create .gitignore**

Create `.gitignore`:
```
/target
.DS_Store
.superpowers/
```

- [ ] **Step 6: Verify workspace builds**

Run: `cargo build`
Expected: compiles with no errors (warnings OK for unused modules)

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml opengamecore-lib/ opengamecore-app/ .gitignore
git commit -m "feat: scaffold workspace with lib and app crates"
```

---

## Task 2: Error Types & Platform Paths

**Files:**
- Create: `opengamecore-lib/src/error.rs`
- Create: `opengamecore-lib/src/paths.rs`
- Test: `opengamecore-lib/src/paths.rs` (inline tests)

- [ ] **Step 1: Write error.rs**

Create `opengamecore-lib/src/error.rs`:
```rust
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Wine not found: {0}")]
    WineNotFound(String),

    #[error("Bottle not found: {0}")]
    BottleNotFound(PathBuf),

    #[error("Game not found: {0}")]
    GameNotFound(String),

    #[error("Download failed: {0}")]
    Download(String),

    #[error("Process error: {0}")]
    Process(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

- [ ] **Step 2: Write paths.rs with tests**

Create `opengamecore-lib/src/paths.rs`:
```rust
use std::path::PathBuf;

use crate::error::{Error, Result};

const APP_NAME: &str = "OpenGameCore";

/// Returns ~/Library/Application Support/OpenGameCore/
pub fn data_dir() -> Result<PathBuf> {
    dirs::data_dir()
        .map(|d| d.join(APP_NAME))
        .ok_or_else(|| Error::Config("Could not resolve data directory".into()))
}

pub fn config_path() -> Result<PathBuf> {
    data_dir().map(|d| d.join("config.toml"))
}

pub fn games_path() -> Result<PathBuf> {
    data_dir().map(|d| d.join("games.toml"))
}

pub fn bottles_dir() -> Result<PathBuf> {
    data_dir().map(|d| d.join("bottles"))
}

pub fn template_bottle_dir() -> Result<PathBuf> {
    bottles_dir().map(|d| d.join("_template"))
}

pub fn bottle_dir(slug: &str) -> Result<PathBuf> {
    bottles_dir().map(|d| d.join(slug))
}

pub fn wine_dir() -> Result<PathBuf> {
    data_dir().map(|d| d.join("wine"))
}

pub fn icons_dir() -> Result<PathBuf> {
    data_dir().map(|d| d.join("icons"))
}

/// Ensure all app directories exist.
pub fn ensure_dirs() -> Result<()> {
    let dirs = [data_dir()?, bottles_dir()?, wine_dir()?, icons_dir()?];
    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_dir_ends_with_app_name() {
        let d = data_dir().unwrap();
        assert!(d.ends_with(APP_NAME));
    }

    #[test]
    fn config_path_is_toml() {
        let p = config_path().unwrap();
        assert_eq!(p.extension().unwrap(), "toml");
        assert!(p.starts_with(data_dir().unwrap()));
    }

    #[test]
    fn bottle_dir_uses_slug() {
        let p = bottle_dir("my-game").unwrap();
        assert!(p.ends_with("my-game"));
        assert!(p.starts_with(bottles_dir().unwrap()));
    }

    #[test]
    fn ensure_dirs_creates_directories() {
        // This test creates real directories — it's an integration test
        // that verifies the happy path. Directories are under ~/Library/...
        // so they persist, which is fine for a real app directory.
        ensure_dirs().unwrap();
        assert!(data_dir().unwrap().exists());
        assert!(bottles_dir().unwrap().exists());
        assert!(wine_dir().unwrap().exists());
        assert!(icons_dir().unwrap().exists());
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p opengamecore-lib`
Expected: all tests pass

- [ ] **Step 4: Commit**

```bash
git add opengamecore-lib/src/error.rs opengamecore-lib/src/paths.rs
git commit -m "feat: add error types and platform path resolution"
```

---

## Task 3: Config Module

**Files:**
- Create: `opengamecore-lib/src/config.rs`
- Test: inline tests in `config.rs`

- [ ] **Step 1: Write failing test for config round-trip**

Create `opengamecore-lib/src/config.rs` with test first:
```rust
use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub wine: WineSettings,
    #[serde(default)]
    pub app: AppSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineSettings {
    pub default: String,
    pub download_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub first_run_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineConfig {
    pub name: String,
    pub binary_path: PathBuf,
    #[serde(default)]
    pub env_overrides: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            wine: WineSettings::default(),
            app: AppSettings::default(),
        }
    }
}

impl Default for WineSettings {
    fn default() -> Self {
        Self {
            default: String::new(),
            download_urls: vec![
                "https://github.com/Gcenx/macOS_Wine_builds/releases/download/v9.0/wine-devel-9.0-osx64.tar.xz".into(),
            ],
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            first_run_complete: false,
        }
    }
}

impl AppConfig {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(AppConfig::default())
        }
    }

    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_config_has_download_url() {
        let config = AppConfig::default();
        assert!(!config.wine.download_urls.is_empty());
        assert!(!config.app.first_run_complete);
    }

    #[test]
    fn config_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        let mut config = AppConfig::default();
        config.wine.default = "wine-9.0".into();
        config.app.first_run_complete = true;

        config.save(&path).unwrap();
        let loaded = AppConfig::load(&path).unwrap();

        assert_eq!(loaded.wine.default, "wine-9.0");
        assert!(loaded.app.first_run_complete);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.toml");
        let config = AppConfig::load(&path).unwrap();
        assert!(!config.app.first_run_complete);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p opengamecore-lib config`
Expected: all 3 tests pass

- [ ] **Step 3: Commit**

```bash
git add opengamecore-lib/src/config.rs
git commit -m "feat: add config module with load/save and defaults"
```

---

## Task 4: Game Library Module

**Files:**
- Create: `opengamecore-lib/src/library.rs`
- Test: inline tests in `library.rs`

- [ ] **Step 1: Write library.rs with Game struct and CRUD**

Create `opengamecore-lib/src/library.rs`:
```rust
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
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p opengamecore-lib library`
Expected: all 5 tests pass

- [ ] **Step 3: Commit**

```bash
git add opengamecore-lib/src/library.rs
git commit -m "feat: add game library with TOML persistence and CRUD"
```

---

## Task 5: Bottle Manager

**Files:**
- Create: `opengamecore-lib/src/bottle.rs`
- Test: inline tests in `bottle.rs`

- [ ] **Step 1: Write bottle.rs**

Create `opengamecore-lib/src/bottle.rs`:
```rust
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

/// Reset a bottle: delete and re-clone from template.
pub fn reset(template_dir: &Path, bottle_dir: &Path) -> Result<()> {
    if bottle_dir.exists() {
        std::fs::remove_dir_all(bottle_dir)?;
    }
    create(template_dir, bottle_dir)
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
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p opengamecore-lib bottle`
Expected: all 6 tests pass

- [ ] **Step 3: Commit**

```bash
git add opengamecore-lib/src/bottle.rs
git commit -m "feat: add bottle manager with APFS clonefile and fallback copy"
```

---

## Task 6: Wine Manager

**Files:**
- Create: `opengamecore-lib/src/wine.rs`
- Test: inline tests in `wine.rs`

- [ ] **Step 1: Write wine.rs**

Create `opengamecore-lib/src/wine.rs`:
```rust
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

    Ok(configs)
}

/// Search a Wine installation directory for the wine binary.
/// Looks for bin/wine64, bin/wine in the directory tree.
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
/// Returns the path to the extracted directory.
pub async fn download_and_extract(
    url: &str,
    wine_dir: &Path,
) -> Result<PathBuf> {
    std::fs::create_dir_all(wine_dir)?;

    let archive_name = url
        .split('/')
        .last()
        .unwrap_or("wine.tar.xz");
    let archive_path = wine_dir.join(archive_name);

    // Download
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

    // Extract with tar
    let status = std::process::Command::new("tar")
        .args(["xf"])
        .arg(&archive_path)
        .current_dir(wine_dir)
        .status()?;

    if !status.success() {
        return Err(Error::Download("Failed to extract Wine archive".into()));
    }

    // Clean up archive
    std::fs::remove_file(&archive_path).ok();

    // Find the extracted directory (newest dir in wine_dir)
    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;
    for entry in std::fs::read_dir(wine_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let modified = entry.metadata()?.modified()?;
            if newest.as_ref().map_or(true, |(_, t)| modified > *t) {
                newest = Some((entry.path(), modified));
            }
        }
    }

    newest
        .map(|(p, _)| p)
        .ok_or_else(|| Error::Download("No directory found after extraction".into()))
}

/// Resolve which WineConfig to use given a config name.
/// "default" uses the first available; otherwise matches by name.
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

        assert_eq!(
            find_wine_binary(tmp.path()),
            Some(bin.join("wine64"))
        );
    }

    #[test]
    fn find_wine_binary_nested() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("wine-9.0").join("bin");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("wine"), "fake").unwrap();

        assert_eq!(
            find_wine_binary(tmp.path()),
            Some(nested.join("wine"))
        );
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
        // Should find at least the local install (Homebrew may or may not exist)
        assert!(configs.iter().any(|c| c.name == "wine-9.0"));
    }

    #[test]
    fn resolve_default_picks_first() {
        let configs = vec![
            WineConfig {
                name: "wine-9.0".into(),
                binary_path: "/fake/bin/wine".into(),
                env_overrides: Default::default(),
            },
        ];
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
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p opengamecore-lib wine`
Expected: all 7 tests pass

- [ ] **Step 3: Commit**

```bash
git add opengamecore-lib/src/wine.rs
git commit -m "feat: add Wine discovery, download, and resolution"
```

---

## Task 7: Runner Module

**Files:**
- Create: `opengamecore-lib/src/runner.rs`
- Test: inline tests in `runner.rs`

- [ ] **Step 1: Write runner.rs**

Create `opengamecore-lib/src/runner.rs`:
```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::{Child, Command};

use crate::config::WineConfig;
use crate::error::{Error, Result};

/// Everything needed to launch a game.
#[derive(Debug, Clone)]
pub struct LaunchConfig {
    pub wine_binary: PathBuf,
    pub prefix: PathBuf,
    pub exe: PathBuf,
    pub env: HashMap<String, String>,
}

impl LaunchConfig {
    /// Build a LaunchConfig from wine config, bottle path, and game exe (relative to bottle).
    pub fn new(
        wine: &WineConfig,
        bottle_dir: &Path,
        exe_relative: &str,
        game_env: &HashMap<String, String>,
    ) -> Self {
        let mut env = wine.env_overrides.clone();
        env.extend(game_env.clone());

        Self {
            wine_binary: wine.binary_path.clone(),
            prefix: bottle_dir.to_path_buf(),
            exe: bottle_dir.join(exe_relative),
            env,
        }
    }
}

/// Spawn a Wine process for the given launch config.
/// Returns the child process handle.
pub fn spawn(config: &LaunchConfig) -> Result<Child> {
    if !config.wine_binary.exists() {
        return Err(Error::WineNotFound(format!(
            "Binary not found: {}",
            config.wine_binary.display()
        )));
    }

    let child = Command::new(&config.wine_binary)
        .arg(&config.exe)
        .env("WINEPREFIX", &config.prefix)
        .envs(&config.env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::Process(format!("Failed to spawn Wine: {}", e)))?;

    Ok(child)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_config_merges_env() {
        let wine = WineConfig {
            name: "test".into(),
            binary_path: "/bin/wine".into(),
            env_overrides: HashMap::from([("WINEDEBUG".into(), "-all".into())]),
        };
        let game_env = HashMap::from([("DXVK_HUD".into(), "1".into())]);

        let config = LaunchConfig::new(
            &wine,
            Path::new("/bottles/game"),
            "drive_c/game.exe",
            &game_env,
        );

        assert_eq!(config.env.get("WINEDEBUG").unwrap(), "-all");
        assert_eq!(config.env.get("DXVK_HUD").unwrap(), "1");
        assert_eq!(
            config.exe,
            PathBuf::from("/bottles/game/drive_c/game.exe")
        );
        assert_eq!(config.prefix, PathBuf::from("/bottles/game"));
    }

    #[test]
    fn launch_config_game_env_overrides_wine_env() {
        let wine = WineConfig {
            name: "test".into(),
            binary_path: "/bin/wine".into(),
            env_overrides: HashMap::from([("KEY".into(), "wine-val".into())]),
        };
        let game_env = HashMap::from([("KEY".into(), "game-val".into())]);

        let config = LaunchConfig::new(
            &wine,
            Path::new("/bottles/game"),
            "drive_c/game.exe",
            &game_env,
        );

        assert_eq!(config.env.get("KEY").unwrap(), "game-val");
    }

    #[tokio::test]
    async fn spawn_fails_if_binary_missing() {
        let config = LaunchConfig {
            wine_binary: PathBuf::from("/nonexistent/wine"),
            prefix: PathBuf::from("/tmp/prefix"),
            exe: PathBuf::from("/tmp/game.exe"),
            env: HashMap::new(),
        };
        assert!(spawn(&config).is_err());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p opengamecore-lib runner`
Expected: all 3 tests pass

- [ ] **Step 3: Commit**

```bash
git add opengamecore-lib/src/runner.rs
git commit -m "feat: add game runner with Wine process spawning"
```

---

## Task 8: Lib Public API

**Files:**
- Modify: `opengamecore-lib/src/lib.rs`

- [ ] **Step 1: Update lib.rs with re-exports**

Replace `opengamecore-lib/src/lib.rs` with:
```rust
pub mod bottle;
pub mod config;
pub mod error;
pub mod library;
pub mod paths;
pub mod runner;
pub mod wine;

pub use config::{AppConfig, WineConfig};
pub use error::{Error, Result};
pub use library::{Game, GameLibrary, InstallType};
pub use runner::LaunchConfig;
```

- [ ] **Step 2: Verify full lib builds and tests pass**

Run: `cargo test -p opengamecore-lib`
Expected: all tests pass (paths, config, library, bottle, wine, runner)

- [ ] **Step 3: Commit**

```bash
git add opengamecore-lib/src/lib.rs
git commit -m "feat: finalize lib public API with re-exports"
```

---

## Task 9: App Theme & Shell

**Files:**
- Create: `opengamecore-app/src/theme.rs`
- Create: `opengamecore-app/src/app.rs`
- Create: `opengamecore-app/src/views/mod.rs`
- Modify: `opengamecore-app/src/main.rs`

- [ ] **Step 1: Create theme.rs**

Create `opengamecore-app/src/theme.rs`:
```rust
use iced::Color;

pub const BG_DARK: Color = Color::from_rgb(0.1, 0.1, 0.18);
pub const BG_SIDEBAR: Color = Color::from_rgb(0.086, 0.13, 0.24);
pub const BG_CARD: Color = Color::from_rgb(0.086, 0.13, 0.24);
pub const ACCENT: Color = Color::from_rgb(0.39, 1.0, 0.855);
pub const TEXT_PRIMARY: Color = Color::from_rgb(0.88, 0.88, 0.88);
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.53, 0.53, 0.53);
pub const BUTTON_GREEN: Color = Color::from_rgb(0.024, 0.306, 0.231);
pub const BUTTON_GREEN_TEXT: Color = Color::from_rgb(0.431, 0.906, 0.714);
```

- [ ] **Step 2: Create views/mod.rs placeholder**

Create `opengamecore-app/src/views/mod.rs`:
```rust
pub mod sidebar;
pub mod game_grid;
pub mod add_game;
pub mod bottle_detail;
pub mod settings;
pub mod first_run;
```

- [ ] **Step 3: Create app.rs with skeleton**

Create `opengamecore-app/src/app.rs`:
```rust
use iced::widget::{column, container, row, text};
use iced::{Element, Length, Task, Theme};

use opengamecore_lib::{AppConfig, GameLibrary};

#[derive(Debug, Clone)]
pub enum Screen {
    FirstRun,
    Library,
    Bottles,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(Screen),
    Loaded(Box<AppState>),
    // More messages added in later tasks
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub library: GameLibrary,
}

pub struct App {
    screen: Screen,
    config: AppConfig,
    library: GameLibrary,
    loading: bool,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            screen: Screen::Library,
            config: AppConfig::default(),
            library: GameLibrary::default(),
            loading: true,
        };

        let task = Task::perform(
            async {
                let _ = opengamecore_lib::paths::ensure_dirs();
                let config = AppConfig::load(
                    &opengamecore_lib::paths::config_path().unwrap_or_default(),
                )
                .unwrap_or_default();

                let library = GameLibrary::load(
                    &opengamecore_lib::paths::games_path().unwrap_or_default(),
                )
                .unwrap_or_default();

                let screen_hint = if !config.app.first_run_complete {
                    Screen::FirstRun
                } else {
                    Screen::Library
                };

                (config, library, screen_hint)
            },
            |(config, library, screen)| {
                Message::Loaded(Box::new(AppState { config, library }))
            },
        );

        (app, task)
    }

    pub fn title(&self) -> String {
        "OpenGameCore".into()
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(screen) => {
                self.screen = screen;
                Task::none()
            }
            Message::Loaded(state) => {
                self.config = state.config;
                self.library = state.library;
                self.loading = false;
                if !self.config.app.first_run_complete {
                    self.screen = Screen::FirstRun;
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        if self.loading {
            return container(text("Loading..."))
                .center(Length::Fill)
                .into();
        }

        let sidebar = container(
            column![
                text("OpenGameCore").size(20),
                text("All Games").size(14),
                text("Bottles").size(14),
                text("Settings").size(14),
            ]
            .spacing(12)
            .padding(16),
        )
        .width(200);

        let content = container(
            text(format!("{} games", self.library.games.len())).size(16),
        )
        .width(Length::Fill)
        .padding(24);

        container(row![sidebar, content])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
```

- [ ] **Step 4: Update main.rs**

Replace `opengamecore-app/src/main.rs`:
```rust
mod app;
mod theme;
mod views;

use iced::window;
use iced::Size;

fn main() -> iced::Result {
    iced::application(app::App::title, app::App::update, app::App::view)
        .theme(app::App::theme)
        .window(window::Settings {
            size: Size::new(1024.0, 700.0),
            min_size: Some(Size::new(640.0, 480.0)),
            ..Default::default()
        })
        .run_with(app::App::new)
}
```

- [ ] **Step 5: Create placeholder view files**

Create each of these as empty placeholder files so the module compiles:

`opengamecore-app/src/views/sidebar.rs`:
```rust
// Sidebar view — implemented in Task 10
```

`opengamecore-app/src/views/game_grid.rs`:
```rust
// Game grid view — implemented in Task 11
```

`opengamecore-app/src/views/add_game.rs`:
```rust
// Add game dialog — implemented in Task 12
```

`opengamecore-app/src/views/bottle_detail.rs`:
```rust
// Bottle detail view — implemented in Task 13
```

`opengamecore-app/src/views/settings.rs`:
```rust
// Settings view — implemented in Task 14
```

`opengamecore-app/src/views/first_run.rs`:
```rust
// First run view — implemented in Task 15
```

- [ ] **Step 6: Verify the app compiles**

Run: `cargo build -p opengamecore-app`
Expected: compiles successfully (warnings for unused imports OK)

- [ ] **Step 7: Commit**

```bash
git add opengamecore-app/
git commit -m "feat: add app shell with iced, theme, and screen routing"
```

---

## Task 10: Sidebar View

**Files:**
- Modify: `opengamecore-app/src/views/sidebar.rs`
- Modify: `opengamecore-app/src/app.rs`

- [ ] **Step 1: Implement sidebar.rs**

Replace `opengamecore-app/src/views/sidebar.rs`:
```rust
use iced::widget::{button, column, container, text, Space};
use iced::{Background, Border, Element, Length, Theme};

use crate::app::{Message, Screen};
use crate::theme;

pub fn view(current: &Screen) -> Element<Message> {
    let header = text("OpenGameCore")
        .size(20)
        .color(theme::ACCENT);

    let nav_button = |label: &str, screen: Screen, current: &Screen| {
        let is_active = matches!(
            (current, &screen),
            (Screen::Library, Screen::Library)
                | (Screen::Bottles, Screen::Bottles)
                | (Screen::Settings, Screen::Settings)
        );

        let label_color = if is_active {
            theme::ACCENT
        } else {
            theme::TEXT_PRIMARY
        };

        button(text(label).size(14).color(label_color))
            .on_press(Message::NavigateTo(screen))
            .padding([8, 12])
            .width(Length::Fill)
            .style(move |_theme: &Theme, status| {
                let bg = if is_active {
                    Some(Background::Color(theme::BG_DARK))
                } else {
                    None
                };
                button::Style {
                    background: bg,
                    text_color: label_color,
                    border: Border::default().rounded(6),
                    ..Default::default()
                }
            })
            .into()
    };

    let nav = column![
        nav_button("All Games", Screen::Library, current),
        nav_button("Bottles", Screen::Bottles, current),
        nav_button("Settings", Screen::Settings, current),
    ]
    .spacing(4);

    container(
        column![header, Space::with_height(20), nav]
            .spacing(0)
            .padding(16),
    )
    .width(200)
    .height(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(theme::BG_SIDEBAR)),
        ..Default::default()
    })
    .into()
}
```

- [ ] **Step 2: Update app.rs to use sidebar**

In `app.rs`, update the `view` method to use the sidebar component. Replace the existing `view` method body (after the loading check) with:
```rust
        let sidebar = crate::views::sidebar::view(&self.screen);

        let content: Element<Message> = match &self.screen {
            Screen::FirstRun => text("First run setup...").size(16).into(),
            Screen::Library => text(format!("{} games in library", self.library.games.len())).size(16).into(),
            Screen::Bottles => text("Bottles").size(16).into(),
            Screen::Settings => text("Settings").size(16).into(),
        };

        let main = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(crate::theme::BG_DARK)),
                ..Default::default()
            });

        container(row![sidebar, main])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
```

Add the necessary import to app.rs: `use iced::Theme;` (if not already present).

- [ ] **Step 3: Verify compiles and run manually**

Run: `cargo build -p opengamecore-app`
Expected: compiles. Optionally `cargo run -p opengamecore-app` to visually verify sidebar renders.

- [ ] **Step 4: Commit**

```bash
git add opengamecore-app/src/views/sidebar.rs opengamecore-app/src/app.rs
git commit -m "feat: add sidebar navigation with screen routing"
```

---

## Task 11: Game Grid View

**Files:**
- Modify: `opengamecore-app/src/views/game_grid.rs`
- Modify: `opengamecore-app/src/app.rs`

- [ ] **Step 1: Implement game_grid.rs**

Replace `opengamecore-app/src/views/game_grid.rs`:
```rust
use iced::widget::{button, column, container, row, text, Space, Scrollable};
use iced::{Background, Border, Element, Length, Theme};

use opengamecore_lib::Game;

use crate::app::Message;
use crate::theme;

pub fn view<'a>(games: &'a [Game]) -> Element<'a, Message> {
    let header = row![
        text("All Games").size(24).color(theme::TEXT_PRIMARY),
        Space::with_width(Length::Fill),
        button(text("+ Add Game").size(14).color(theme::ACCENT))
            .on_press(Message::OpenAddGame)
            .padding([8, 16])
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(Background::Color(theme::BG_CARD)),
                text_color: theme::ACCENT,
                border: Border::default().rounded(6).color(theme::ACCENT).width(1),
                ..Default::default()
            }),
    ]
    .align_y(iced::Alignment::Center);

    if games.is_empty() {
        return column![
            header,
            Space::with_height(40),
            container(
                column![
                    text("No games yet").size(18).color(theme::TEXT_SECONDARY),
                    text("Click '+ Add Game' to get started")
                        .size(14)
                        .color(theme::TEXT_SECONDARY),
                ]
                .spacing(8)
                .align_x(iced::Alignment::Center),
            )
            .center(Length::Fill),
        ]
        .spacing(0)
        .into();
    }

    let cards_per_row = 3;
    let mut grid_rows: Vec<Element<Message>> = Vec::new();

    for chunk in games.chunks(cards_per_row) {
        let mut card_row = row![].spacing(16);
        for game in chunk {
            card_row = card_row.push(game_card(game));
        }
        // Fill remaining space if row is incomplete
        for _ in chunk.len()..cards_per_row {
            card_row = card_row.push(Space::with_width(Length::Fill));
        }
        grid_rows.push(card_row.into());
    }

    let grid = column(grid_rows).spacing(16);

    column![
        header,
        Space::with_height(20),
        Scrollable::new(grid).height(Length::Fill),
    ]
    .spacing(0)
    .into()
}

fn game_card(game: &Game) -> Element<Message> {
    let icon_placeholder = container(
        text(&game.name[..1.min(game.name.len())])
            .size(24)
            .color(theme::ACCENT),
    )
    .width(64)
    .height(64)
    .center(Length::Shrink)
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(theme::BG_DARK)),
        border: Border::default().rounded(8),
        ..Default::default()
    });

    let play_btn = button(text("Play").size(12).color(theme::BUTTON_GREEN_TEXT))
        .on_press(Message::PlayGame(game.slug.clone()))
        .padding([4, 12])
        .style(|_theme: &Theme, _status| button::Style {
            background: Some(Background::Color(theme::BUTTON_GREEN)),
            text_color: theme::BUTTON_GREEN_TEXT,
            border: Border::default().rounded(4),
            ..Default::default()
        });

    let info = column![
        text(&game.name).size(14).color(theme::TEXT_PRIMARY),
        text(&game.wine_config).size(11).color(theme::TEXT_SECONDARY),
    ]
    .spacing(4);

    container(
        column![icon_placeholder, info, play_btn]
            .spacing(8)
            .padding(12)
            .align_x(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(theme::BG_CARD)),
        border: Border::default().rounded(8),
        ..Default::default()
    })
    .into()
}
```

- [ ] **Step 2: Add new messages to app.rs**

Add to the `Message` enum in `app.rs`:
```rust
    OpenAddGame,
    PlayGame(String),
```

Add match arms in `update`:
```rust
            Message::OpenAddGame => {
                // Will be handled in Task 12
                Task::none()
            }
            Message::PlayGame(_slug) => {
                // Will be handled in Task 16
                Task::none()
            }
```

Update the `Screen::Library` match arm in `view` to:
```rust
            Screen::Library => crate::views::game_grid::view(&self.library.games),
```

- [ ] **Step 3: Verify compiles**

Run: `cargo build -p opengamecore-app`
Expected: compiles successfully

- [ ] **Step 4: Commit**

```bash
git add opengamecore-app/src/views/game_grid.rs opengamecore-app/src/app.rs
git commit -m "feat: add game grid view with cards and play buttons"
```

---

## Task 12: Add Game Dialog

**Files:**
- Modify: `opengamecore-app/src/views/add_game.rs`
- Modify: `opengamecore-app/src/app.rs`

- [ ] **Step 1: Implement add_game.rs**

Replace `opengamecore-app/src/views/add_game.rs`:
```rust
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Background, Border, Element, Length, Theme};

use crate::app::Message;
use crate::theme;

#[derive(Debug, Clone, PartialEq)]
pub enum AddGameTab {
    Installer,
    Portable,
    FromFolder,
}

#[derive(Debug, Clone)]
pub struct AddGameState {
    pub tab: AddGameTab,
    pub name: String,
    pub path: String,
}

impl Default for AddGameState {
    fn default() -> Self {
        Self {
            tab: AddGameTab::Installer,
            name: String::new(),
            path: String::new(),
        }
    }
}

pub fn view(state: &AddGameState) -> Element<Message> {
    let tabs = row![
        tab_button("Installer", AddGameTab::Installer, &state.tab),
        tab_button("Portable", AddGameTab::Portable, &state.tab),
        tab_button("From Folder", AddGameTab::FromFolder, &state.tab),
    ]
    .spacing(8);

    let path_label = match state.tab {
        AddGameTab::Installer => "Installer (.exe)",
        AddGameTab::Portable => "Game Folder",
        AddGameTab::FromFolder => "Setup Folder",
    };

    let browse_btn = button(text("Browse...").size(13).color(theme::TEXT_PRIMARY))
        .on_press(Message::AddGameBrowse)
        .padding([6, 12])
        .style(|_theme: &Theme, _status| button::Style {
            background: Some(Background::Color(theme::BG_SIDEBAR)),
            border: Border::default().rounded(4).color(theme::TEXT_SECONDARY).width(1),
            ..Default::default()
        });

    let path_display = if state.path.is_empty() {
        text("No file selected").size(13).color(theme::TEXT_SECONDARY)
    } else {
        text(&state.path).size(13).color(theme::TEXT_PRIMARY)
    };

    let form = column![
        text(path_label).size(13).color(theme::TEXT_SECONDARY),
        row![path_display, Space::with_width(Length::Fill), browse_btn]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        Space::with_height(12),
        text("Game Name").size(13).color(theme::TEXT_SECONDARY),
        text_input("Enter game name...", &state.name)
            .on_input(Message::AddGameNameChanged)
            .padding(8)
            .size(14),
    ]
    .spacing(8);

    let can_confirm = !state.name.is_empty() && !state.path.is_empty();

    let actions = row![
        Space::with_width(Length::Fill),
        button(text("Cancel").size(14).color(theme::TEXT_SECONDARY))
            .on_press(Message::CloseAddGame)
            .padding([8, 16])
            .style(|_theme: &Theme, _status| button::Style {
                background: None,
                ..Default::default()
            }),
        if can_confirm {
            button(text("Add Game").size(14).color(theme::BUTTON_GREEN_TEXT))
                .on_press(Message::ConfirmAddGame)
                .padding([8, 16])
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(Background::Color(theme::BUTTON_GREEN)),
                    border: Border::default().rounded(6),
                    ..Default::default()
                })
        } else {
            button(text("Add Game").size(14).color(theme::TEXT_SECONDARY))
                .padding([8, 16])
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(Background::Color(theme::BG_SIDEBAR)),
                    border: Border::default().rounded(6),
                    ..Default::default()
                })
        },
    ]
    .spacing(8);

    container(
        column![
            text("Add Game").size(20).color(theme::TEXT_PRIMARY),
            Space::with_height(16),
            tabs,
            Space::with_height(16),
            form,
            Space::with_height(24),
            actions,
        ]
        .padding(24),
    )
    .max_width(500)
    .style(|_theme: &Theme| container::Style {
        background: Some(Background::Color(theme::BG_CARD)),
        border: Border::default().rounded(12),
        ..Default::default()
    })
    .into()
}

fn tab_button<'a>(
    label: &'a str,
    tab: AddGameTab,
    current: &AddGameTab,
) -> Element<'a, Message> {
    let is_active = &tab == current;
    let color = if is_active {
        theme::ACCENT
    } else {
        theme::TEXT_SECONDARY
    };

    button(text(label).size(13).color(color))
        .on_press(Message::AddGameTabChanged(tab))
        .padding([6, 12])
        .style(move |_theme: &Theme, _status| {
            let bg = if is_active {
                Some(Background::Color(theme::BG_DARK))
            } else {
                None
            };
            button::Style {
                background: bg,
                border: Border::default().rounded(4),
                ..Default::default()
            }
        })
        .into()
}
```

- [ ] **Step 2: Add messages and state to app.rs**

Add to `Message` enum:
```rust
    AddGameTabChanged(crate::views::add_game::AddGameTab),
    AddGameNameChanged(String),
    AddGameBrowse,
    ConfirmAddGame,
    CloseAddGame,
    AddGamePathSelected(Option<String>),
```

Add field to `App` struct:
```rust
    add_game: Option<crate::views::add_game::AddGameState>,
```

Initialize in `App::new()`: `add_game: None,`

Add match arms in `update`:
```rust
            Message::OpenAddGame => {
                self.add_game = Some(Default::default());
                Task::none()
            }
            Message::CloseAddGame => {
                self.add_game = None;
                Task::none()
            }
            Message::AddGameTabChanged(tab) => {
                if let Some(state) = &mut self.add_game {
                    state.tab = tab;
                    state.path.clear();
                }
                Task::none()
            }
            Message::AddGameNameChanged(name) => {
                if let Some(state) = &mut self.add_game {
                    state.name = name;
                }
                Task::none()
            }
            Message::AddGameBrowse => {
                let is_file = self.add_game.as_ref()
                    .map(|s| s.tab == crate::views::add_game::AddGameTab::Installer)
                    .unwrap_or(false);

                Task::perform(
                    async move {
                        if is_file {
                            rfd::AsyncFileDialog::new()
                                .add_filter("Executable", &["exe"])
                                .pick_file()
                                .await
                                .map(|f| f.path().to_string_lossy().to_string())
                        } else {
                            rfd::AsyncFileDialog::new()
                                .pick_folder()
                                .await
                                .map(|f| f.path().to_string_lossy().to_string())
                        }
                    },
                    Message::AddGamePathSelected,
                )
            }
            Message::AddGamePathSelected(path) => {
                if let (Some(path), Some(state)) = (path, &mut self.add_game) {
                    state.path = path;
                }
                Task::none()
            }
            Message::ConfirmAddGame => {
                if let Some(state) = self.add_game.take() {
                    let install_type = match state.tab {
                        crate::views::add_game::AddGameTab::Installer => {
                            opengamecore_lib::InstallType::Installer
                        }
                        crate::views::add_game::AddGameTab::Portable => {
                            opengamecore_lib::InstallType::Portable
                        }
                        crate::views::add_game::AddGameTab::FromFolder => {
                            opengamecore_lib::InstallType::FolderInstall
                        }
                    };
                    let slug = opengamecore_lib::library::slugify(&state.name);

                    // Create a bottle for this game (clone from template)
                    if let (Ok(template), Ok(bottle_path)) = (
                        opengamecore_lib::paths::template_bottle_dir(),
                        opengamecore_lib::paths::bottle_dir(&slug),
                    ) {
                        let _ = opengamecore_lib::bottle::create(&template, &bottle_path);

                        // For portable games, symlink the game folder into drive_c
                        if install_type == opengamecore_lib::InstallType::Portable {
                            let game_dest = bottle_path.join("drive_c").join("game");
                            let _ = std::os::unix::fs::symlink(&state.path, &game_dest)
                                .or_else(|_| {
                                    // Fallback: copy if symlink fails
                                    opengamecore_lib::bottle::create(
                                        std::path::Path::new(&state.path),
                                        &game_dest,
                                    )
                                });
                        }
                    }

                    // For installer/folder_install, the exe path will need to be
                    // set after running the installer. For portable, derive from
                    // the folder. For now, store the user-provided path and the
                    // user can update it after install completes.
                    let game = opengamecore_lib::Game {
                        name: state.name.clone(),
                        slug,
                        exe: state.path.clone(),
                        install_type,
                        wine_config: "default".into(),
                        env: Default::default(),
                        added_at: chrono::Utc::now(),
                        last_played: None,
                    };
                    self.library.add(game);
                    // Save library and refresh bottles list
                    if let Ok(path) = opengamecore_lib::paths::games_path() {
                        let _ = self.library.save(&path);
                    }
                    if let Ok(dir) = opengamecore_lib::paths::bottles_dir() {
                        self.bottles = opengamecore_lib::bottle::list(&dir).unwrap_or_default();
                    }
                }
                Task::none()
            }
```

In `view`, overlay the add game dialog when active. After building the main layout but before the final `into()`, wrap it:
```rust
        let layout: Element<Message> = container(row![sidebar, main])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        if let Some(add_state) = &self.add_game {
            let overlay = container(crate::views::add_game::view(add_state))
                .center(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(
                        iced::Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                    )),
                    ..Default::default()
                });
            // Use iced's stack or overlay — simplest is a column with the dialog on top
            iced::widget::stack![layout, overlay].into()
        } else {
            layout
        }
```

Add imports at top of app.rs: `use chrono;`

- [ ] **Step 3: Verify compiles**

Run: `cargo build -p opengamecore-app`
Expected: compiles successfully

- [ ] **Step 4: Commit**

```bash
git add opengamecore-app/src/views/add_game.rs opengamecore-app/src/app.rs
git commit -m "feat: add game dialog with installer/portable/folder modes"
```

---

## Task 13: Bottle Detail View

**Files:**
- Modify: `opengamecore-app/src/views/bottle_detail.rs`
- Modify: `opengamecore-app/src/app.rs`

- [ ] **Step 1: Implement bottle_detail.rs**

Replace `opengamecore-app/src/views/bottle_detail.rs`:
```rust
use iced::widget::{button, column, container, row, text, Scrollable, Space};
use iced::{Background, Border, Element, Length, Theme};

use opengamecore_lib::bottle::BottleInfo;

use crate::app::Message;
use crate::theme;

pub fn list_view(bottles: &[BottleInfo]) -> Element<Message> {
    let header = text("Bottles").size(24).color(theme::TEXT_PRIMARY);

    if bottles.is_empty() {
        return column![
            header,
            Space::with_height(20),
            text("No bottles yet. Add a game to create one.")
                .size(14)
                .color(theme::TEXT_SECONDARY),
        ]
        .spacing(0)
        .into();
    }

    let mut items: Vec<Element<Message>> = Vec::new();
    for bottle in bottles {
        let size_mb = bottle.size_bytes / (1024 * 1024);
        let item = container(
            row![
                column![
                    text(&bottle.slug).size(14).color(theme::TEXT_PRIMARY),
                    text(format!("{} MB", size_mb))
                        .size(11)
                        .color(theme::TEXT_SECONDARY),
                ]
                .spacing(4),
                Space::with_width(Length::Fill),
                button(text("Reset").size(12).color(theme::TEXT_SECONDARY))
                    .on_press(Message::ResetBottle(bottle.slug.clone()))
                    .padding([4, 10])
                    .style(|_theme: &Theme, _status| button::Style {
                        border: Border::default()
                            .rounded(4)
                            .color(theme::TEXT_SECONDARY)
                            .width(1),
                        ..Default::default()
                    }),
                button(text("Delete").size(12).color(iced::Color::from_rgb(0.9, 0.3, 0.3)))
                    .on_press(Message::DeleteBottle(bottle.slug.clone()))
                    .padding([4, 10])
                    .style(|_theme: &Theme, _status| button::Style {
                        border: Border::default()
                            .rounded(4)
                            .color(iced::Color::from_rgb(0.9, 0.3, 0.3))
                            .width(1),
                        ..Default::default()
                    }),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding(12),
        )
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(theme::BG_CARD)),
            border: Border::default().rounded(8),
            ..Default::default()
        })
        .into();

        items.push(item);
    }

    let list = column(items).spacing(8);

    column![
        header,
        Space::with_height(20),
        Scrollable::new(list).height(Length::Fill),
    ]
    .spacing(0)
    .into()
}
```

- [ ] **Step 2: Add messages and state to app.rs**

Add to `Message`:
```rust
    ResetBottle(String),
    DeleteBottle(String),
    BottlesLoaded(Vec<opengamecore_lib::bottle::BottleInfo>),
```

Add field to `App`:
```rust
    bottles: Vec<opengamecore_lib::bottle::BottleInfo>,
```

Initialize: `bottles: Vec::new(),`

In the `Message::Loaded` arm, also load bottles:
```rust
            Message::Loaded(state) => {
                self.config = state.config;
                self.library = state.library;
                self.loading = false;
                if !self.config.app.first_run_complete {
                    self.screen = Screen::FirstRun;
                }
                // Load bottles
                if let Ok(dir) = opengamecore_lib::paths::bottles_dir() {
                    self.bottles = opengamecore_lib::bottle::list(&dir).unwrap_or_default();
                }
                Task::none()
            }
```

Add update arms:
```rust
            Message::ResetBottle(slug) => {
                if let (Ok(template), Ok(bottle)) = (
                    opengamecore_lib::paths::template_bottle_dir(),
                    opengamecore_lib::paths::bottle_dir(&slug),
                ) {
                    let _ = opengamecore_lib::bottle::reset(&template, &bottle);
                    if let Ok(dir) = opengamecore_lib::paths::bottles_dir() {
                        self.bottles = opengamecore_lib::bottle::list(&dir).unwrap_or_default();
                    }
                }
                Task::none()
            }
            Message::DeleteBottle(slug) => {
                if let Ok(bottle) = opengamecore_lib::paths::bottle_dir(&slug) {
                    let _ = opengamecore_lib::bottle::delete(&bottle);
                    self.library.remove(&slug).ok();
                    if let Ok(path) = opengamecore_lib::paths::games_path() {
                        let _ = self.library.save(&path);
                    }
                    if let Ok(dir) = opengamecore_lib::paths::bottles_dir() {
                        self.bottles = opengamecore_lib::bottle::list(&dir).unwrap_or_default();
                    }
                }
                Task::none()
            }
            Message::BottlesLoaded(bottles) => {
                self.bottles = bottles;
                Task::none()
            }
```

Update `Screen::Bottles` view arm:
```rust
            Screen::Bottles => crate::views::bottle_detail::list_view(&self.bottles),
```

- [ ] **Step 3: Verify compiles**

Run: `cargo build -p opengamecore-app`
Expected: compiles successfully

- [ ] **Step 4: Commit**

```bash
git add opengamecore-app/src/views/bottle_detail.rs opengamecore-app/src/app.rs
git commit -m "feat: add bottle list view with reset and delete actions"
```

---

## Task 14: Settings View

**Files:**
- Modify: `opengamecore-app/src/views/settings.rs`
- Modify: `opengamecore-app/src/app.rs`

- [ ] **Step 1: Implement settings.rs**

Replace `opengamecore-app/src/views/settings.rs`:
```rust
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Background, Border, Element, Length, Theme};

use opengamecore_lib::config::WineConfig;

use crate::app::Message;
use crate::theme;

pub fn view<'a>(
    wine_configs: &'a [WineConfig],
    default_wine: &'a str,
    download_urls: &'a [String],
) -> Element<'a, Message> {
    let header = text("Settings").size(24).color(theme::TEXT_PRIMARY);

    // Wine installations section
    let wine_header = text("Wine Installations").size(16).color(theme::TEXT_PRIMARY);

    let mut wine_items: Vec<Element<Message>> = Vec::new();
    for config in wine_configs {
        let is_default = config.name == default_wine;
        let label = if is_default {
            format!("{} (default)", config.name)
        } else {
            config.name.clone()
        };

        let item = container(
            row![
                column![
                    text(label).size(13).color(theme::TEXT_PRIMARY),
                    text(config.binary_path.display().to_string())
                        .size(11)
                        .color(theme::TEXT_SECONDARY),
                ]
                .spacing(2),
                Space::with_width(Length::Fill),
                if !is_default {
                    button(text("Set Default").size(11).color(theme::ACCENT))
                        .on_press(Message::SetDefaultWine(config.name.clone()))
                        .padding([4, 8])
                        .style(|_theme: &Theme, _status| button::Style {
                            border: Border::default().rounded(4).color(theme::ACCENT).width(1),
                            ..Default::default()
                        })
                } else {
                    button(text("Default").size(11).color(theme::TEXT_SECONDARY))
                        .padding([4, 8])
                        .style(|_theme: &Theme, _status| button::Style {
                            ..Default::default()
                        })
                },
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding(10),
        )
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(theme::BG_CARD)),
            border: Border::default().rounded(6),
            ..Default::default()
        })
        .into();

        wine_items.push(item);
    }

    if wine_items.is_empty() {
        wine_items.push(
            text("No Wine installations found")
                .size(13)
                .color(theme::TEXT_SECONDARY)
                .into(),
        );
    }

    let add_wine_btn = button(
        text("+ Add Custom Wine Path").size(13).color(theme::ACCENT),
    )
    .on_press(Message::AddCustomWinePath)
    .padding([8, 12])
    .style(|_theme: &Theme, _status| button::Style {
        border: Border::default().rounded(4).color(theme::ACCENT).width(1),
        ..Default::default()
    });

    // Download URLs section
    let urls_header = text("Download Sources").size(16).color(theme::TEXT_PRIMARY);
    let mut url_items: Vec<Element<Message>> = Vec::new();
    for url in download_urls {
        url_items.push(
            text(url)
                .size(12)
                .color(theme::TEXT_SECONDARY)
                .into(),
        );
    }

    column![
        header,
        Space::with_height(24),
        wine_header,
        Space::with_height(8),
        column(wine_items).spacing(6),
        Space::with_height(8),
        add_wine_btn,
        Space::with_height(24),
        urls_header,
        Space::with_height(8),
        column(url_items).spacing(4),
    ]
    .spacing(0)
    .into()
}
```

- [ ] **Step 2: Add messages and state to app.rs**

Add to `Message`:
```rust
    SetDefaultWine(String),
    AddCustomWinePath,
    CustomWinePathSelected(Option<String>),
```

Add field to `App`:
```rust
    wine_configs: Vec<opengamecore_lib::config::WineConfig>,
```

Initialize: `wine_configs: Vec::new(),`

In `Message::Loaded`, discover Wine:
```rust
                if let Ok(dir) = opengamecore_lib::paths::wine_dir() {
                    self.wine_configs = opengamecore_lib::wine::discover(&dir).unwrap_or_default();
                }
```

Add update arms:
```rust
            Message::SetDefaultWine(name) => {
                self.config.wine.default = name;
                if let Ok(path) = opengamecore_lib::paths::config_path() {
                    let _ = self.config.save(&path);
                }
                Task::none()
            }
            Message::AddCustomWinePath => {
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .pick_folder()
                            .await
                            .map(|f| f.path().to_string_lossy().to_string())
                    },
                    Message::CustomWinePathSelected,
                )
            }
            Message::CustomWinePathSelected(path) => {
                if let Some(path) = path {
                    let name = std::path::Path::new(&path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "custom".into());
                    self.wine_configs.push(opengamecore_lib::config::WineConfig {
                        name,
                        binary_path: path.into(),
                        env_overrides: Default::default(),
                    });
                }
                Task::none()
            }
```

Update `Screen::Settings` view arm:
```rust
            Screen::Settings => crate::views::settings::view(
                &self.wine_configs,
                &self.config.wine.default,
                &self.config.wine.download_urls,
            ),
```

- [ ] **Step 3: Verify compiles**

Run: `cargo build -p opengamecore-app`
Expected: compiles successfully

- [ ] **Step 4: Commit**

```bash
git add opengamecore-app/src/views/settings.rs opengamecore-app/src/app.rs
git commit -m "feat: add settings view with Wine management"
```

---

## Task 15: First Run View

**Files:**
- Modify: `opengamecore-app/src/views/first_run.rs`
- Modify: `opengamecore-app/src/app.rs`

- [ ] **Step 1: Implement first_run.rs**

Replace `opengamecore-app/src/views/first_run.rs`:
```rust
use iced::widget::{button, column, container, progress_bar, text, Space};
use iced::{Background, Border, Element, Length, Theme};

use crate::app::Message;
use crate::theme;

#[derive(Debug, Clone)]
pub enum FirstRunPhase {
    Welcome,
    Downloading { progress: f32, status: String },
    CreatingTemplate { status: String },
    Done,
    Error(String),
}

pub fn view(phase: &FirstRunPhase) -> Element<Message> {
    let content: Element<Message> = match phase {
        FirstRunPhase::Welcome => {
            column![
                text("Welcome to OpenGameCore").size(28).color(theme::ACCENT),
                Space::with_height(12),
                text("Run your Windows games on macOS with Wine.")
                    .size(16)
                    .color(theme::TEXT_PRIMARY),
                Space::with_height(8),
                text("To get started, we need to download Wine. This only happens once.")
                    .size(14)
                    .color(theme::TEXT_SECONDARY),
                Space::with_height(24),
                button(
                    text("Download Wine & Get Started")
                        .size(16)
                        .color(theme::BUTTON_GREEN_TEXT),
                )
                .on_press(Message::StartFirstRun)
                .padding([12, 24])
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(Background::Color(theme::BUTTON_GREEN)),
                    border: Border::default().rounded(8),
                    ..Default::default()
                }),
                Space::with_height(16),
                button(
                    text("I'll configure Wine manually in Settings")
                        .size(12)
                        .color(theme::TEXT_SECONDARY),
                )
                .on_press(Message::SkipFirstRun)
                .padding([6, 12])
                .style(|_theme: &Theme, _status| button::Style {
                    ..Default::default()
                }),
            ]
            .align_x(iced::Alignment::Center)
            .into()
        }
        FirstRunPhase::Downloading { progress, status } => {
            column![
                text("Downloading Wine...").size(20).color(theme::TEXT_PRIMARY),
                Space::with_height(16),
                progress_bar(0.0..=100.0, *progress).height(8),
                Space::with_height(8),
                text(status).size(12).color(theme::TEXT_SECONDARY),
            ]
            .align_x(iced::Alignment::Center)
            .width(400)
            .into()
        }
        FirstRunPhase::CreatingTemplate { status } => {
            column![
                text("Setting up Wine...").size(20).color(theme::TEXT_PRIMARY),
                Space::with_height(16),
                text(status).size(12).color(theme::TEXT_SECONDARY),
            ]
            .align_x(iced::Alignment::Center)
            .into()
        }
        FirstRunPhase::Done => {
            column![
                text("All set!").size(24).color(theme::ACCENT),
                Space::with_height(12),
                text("Wine is ready. You can now add your games.")
                    .size(14)
                    .color(theme::TEXT_PRIMARY),
                Space::with_height(20),
                button(text("Go to Library").size(14).color(theme::BUTTON_GREEN_TEXT))
                    .on_press(Message::FinishFirstRun)
                    .padding([10, 20])
                    .style(|_theme: &Theme, _status| button::Style {
                        background: Some(Background::Color(theme::BUTTON_GREEN)),
                        border: Border::default().rounded(8),
                        ..Default::default()
                    }),
            ]
            .align_x(iced::Alignment::Center)
            .into()
        }
        FirstRunPhase::Error(err) => {
            column![
                text("Setup Failed").size(20).color(iced::Color::from_rgb(0.9, 0.3, 0.3)),
                Space::with_height(12),
                text(err).size(13).color(theme::TEXT_SECONDARY),
                Space::with_height(16),
                text("You can configure Wine manually in Settings.")
                    .size(12)
                    .color(theme::TEXT_SECONDARY),
                Space::with_height(16),
                button(text("Go to Settings").size(14).color(theme::ACCENT))
                    .on_press(Message::SkipFirstRun)
                    .padding([8, 16])
                    .style(|_theme: &Theme, _status| button::Style {
                        border: Border::default().rounded(6).color(theme::ACCENT).width(1),
                        ..Default::default()
                    }),
            ]
            .align_x(iced::Alignment::Center)
            .into()
        }
    };

    container(content)
        .center(Length::Fill)
        .into()
}
```

- [ ] **Step 2: Add messages and state to app.rs**

Add to `Message`:
```rust
    StartFirstRun,
    SkipFirstRun,
    FinishFirstRun,
    FirstRunProgress(f32, String),
    FirstRunTemplateCreating,
    FirstRunComplete,
    FirstRunError(String),
```

Add field to `App`:
```rust
    first_run_phase: crate::views::first_run::FirstRunPhase,
```

Initialize: `first_run_phase: crate::views::first_run::FirstRunPhase::Welcome,`

Add update arms:
```rust
            Message::StartFirstRun => {
                self.first_run_phase = crate::views::first_run::FirstRunPhase::Downloading {
                    progress: 0.0,
                    status: "Starting download...".into(),
                };
                let url = self.config.wine.download_urls.first().cloned()
                    .unwrap_or_else(|| "https://github.com/Gcenx/macOS_Wine_builds/releases/download/v9.0/wine-devel-9.0-osx64.tar.xz".into());

                Task::perform(
                    async move {
                        let wine_dir = opengamecore_lib::paths::wine_dir()
                            .map_err(|e| e.to_string())?;
                        let extracted = opengamecore_lib::wine::download_and_extract(&url, &wine_dir)
                            .await
                            .map_err(|e| e.to_string())?;

                        // Find wine binary in extracted dir
                        let configs = opengamecore_lib::wine::discover(&wine_dir)
                            .map_err(|e| e.to_string())?;
                        let wine_config = configs.first()
                            .ok_or_else(|| "No Wine binary found after extraction".to_string())?;

                        // Create template bottle
                        let template_dir = opengamecore_lib::paths::template_bottle_dir()
                            .map_err(|e| e.to_string())?;
                        opengamecore_lib::bottle::create_template(
                            &wine_config.binary_path,
                            &template_dir,
                        )
                        .map_err(|e| e.to_string())?;

                        Ok::<String, String>(wine_config.name.clone())
                    },
                    |result| match result {
                        Ok(name) => Message::FirstRunComplete,
                        Err(e) => Message::FirstRunError(e),
                    },
                )
            }
            Message::SkipFirstRun => {
                self.config.app.first_run_complete = true;
                if let Ok(path) = opengamecore_lib::paths::config_path() {
                    let _ = self.config.save(&path);
                }
                self.screen = Screen::Settings;
                Task::none()
            }
            Message::FinishFirstRun => {
                self.config.app.first_run_complete = true;
                if let Ok(path) = opengamecore_lib::paths::config_path() {
                    let _ = self.config.save(&path);
                }
                self.screen = Screen::Library;
                // Refresh wine configs
                if let Ok(dir) = opengamecore_lib::paths::wine_dir() {
                    self.wine_configs = opengamecore_lib::wine::discover(&dir).unwrap_or_default();
                    if let Some(first) = self.wine_configs.first() {
                        self.config.wine.default = first.name.clone();
                    }
                }
                Task::none()
            }
            Message::FirstRunProgress(progress, status) => {
                self.first_run_phase = crate::views::first_run::FirstRunPhase::Downloading {
                    progress,
                    status,
                };
                Task::none()
            }
            Message::FirstRunTemplateCreating => {
                self.first_run_phase = crate::views::first_run::FirstRunPhase::CreatingTemplate {
                    status: "Initializing Wine prefix...".into(),
                };
                Task::none()
            }
            Message::FirstRunComplete => {
                self.first_run_phase = crate::views::first_run::FirstRunPhase::Done;
                Task::none()
            }
            Message::FirstRunError(err) => {
                self.first_run_phase = crate::views::first_run::FirstRunPhase::Error(err);
                Task::none()
            }
```

Update `Screen::FirstRun` view arm:
```rust
            Screen::FirstRun => crate::views::first_run::view(&self.first_run_phase),
```

- [ ] **Step 3: Verify compiles**

Run: `cargo build -p opengamecore-app`
Expected: compiles successfully

- [ ] **Step 4: Commit**

```bash
git add opengamecore-app/src/views/first_run.rs opengamecore-app/src/app.rs
git commit -m "feat: add first-run flow with Wine download and template creation"
```

---

## Task 16: Wire Up Game Launching

**Files:**
- Modify: `opengamecore-app/src/app.rs`

- [ ] **Step 1: Implement PlayGame handler**

Update the `Message::PlayGame` arm in `app.rs`:
```rust
            Message::PlayGame(slug) => {
                let game = match self.library.find(&slug) {
                    Some(g) => g.clone(),
                    None => return Task::none(),
                };

                let wine_config = match opengamecore_lib::wine::resolve(
                    &self.wine_configs,
                    &game.wine_config,
                ) {
                    Ok(c) => c,
                    Err(_) => return Task::none(),
                };

                let bottle_dir = match opengamecore_lib::paths::bottle_dir(&slug) {
                    Ok(d) => d,
                    Err(_) => return Task::none(),
                };

                let launch = opengamecore_lib::LaunchConfig::new(
                    &wine_config,
                    &bottle_dir,
                    &game.exe,
                    &game.env,
                );

                // Update last_played
                if let Some(g) = self.library.find_mut(&slug) {
                    g.last_played = Some(chrono::Utc::now());
                    if let Ok(path) = opengamecore_lib::paths::games_path() {
                        let _ = self.library.save(&path);
                    }
                }

                Task::perform(
                    async move {
                        let mut child = opengamecore_lib::runner::spawn(&launch)
                            .map_err(|e| e.to_string())?;
                        let status = child.wait().await.map_err(|e| e.to_string())?;
                        Ok::<String, String>(format!("Exited: {}", status))
                    },
                    |result| match result {
                        Ok(status) => Message::GameExited(status),
                        Err(e) => Message::GameExited(format!("Error: {}", e)),
                    },
                )
            }
```

Add to `Message`:
```rust
    GameExited(String),
```

Add handler:
```rust
            Message::GameExited(_status) => {
                Task::none()
            }
```

- [ ] **Step 2: Verify compiles**

Run: `cargo build -p opengamecore-app`
Expected: compiles successfully

- [ ] **Step 3: Commit**

```bash
git add opengamecore-app/src/app.rs
git commit -m "feat: wire up game launching with Wine runner"
```

---

## Task 17: Integration Test & Final Polish

**Files:**
- Modify: `opengamecore-app/src/app.rs` (add missing `use` statements, fix warnings)
- Verify: all workspace tests

- [ ] **Step 1: Fix any remaining compiler warnings**

Run: `cargo build --workspace 2>&1`
Review output, fix any unused imports or dead code warnings.

- [ ] **Step 2: Run all lib tests**

Run: `cargo test -p opengamecore-lib`
Expected: all tests pass

- [ ] **Step 3: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Fix any issues found.

- [ ] **Step 4: Verify app launches**

Run: `cargo run -p opengamecore-app`
Expected: window opens with sidebar (All Games, Bottles, Settings), shows "No games yet" in main area. If first_run_complete is false in config, shows Welcome screen.

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: fix warnings and polish for initial release"
```
