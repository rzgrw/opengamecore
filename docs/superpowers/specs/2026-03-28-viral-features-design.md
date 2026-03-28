# OpenGameCore: Viral Features Design

## Goal

Make OpenGameCore the single answer to "can I play this Windows game on Mac?" by shipping a compatibility database aggregated from WineHQ, ProtonDB, CrossOver, and PlayOnMac — with one-click bundle configs and automatic game store detection. Drive GitHub stars through first-run "wow" moments and organic Google discoverability.

## Architecture Overview

Three new subsystems added to the existing codebase:

1. **Compatibility Database** — a committed `data/` directory with `compatibility.json` and per-game bundle TOMLs
2. **Store Detection** — new lib module that scans Steam and GOG local installs, matches against the database
3. **Bundle Auto-Configuration** — new lib module that applies bundle configs to auto-create fully configured game entries

Data is scraped and curated externally (not part of this repo). The repo only contains the clean output.

---

## 1. Compatibility Database (`data/`)

### File Structure

```
data/
  compatibility.json          # master database (~5000 games)
  bundles/
    cyberpunk-2077.toml       # per-game install config
    elden-ring.toml
    baldurs-gate-3.toml
    ...
```

### compatibility.json Schema

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
    }
  ]
}
```

**Rating scale:**
- **Platinum** — works perfectly out of the box
- **Gold** — minor tweaks needed (bundle handles them)
- **Silver** — playable with workarounds
- **Bronze** — runs but significant issues
- **Borked** — does not work

**Confidence score** (0.0–1.0): derived from source agreement. Multiple sources reporting the same rating = higher confidence.

### Bundle TOML Schema (`bundles/<slug>.toml`)

```toml
[game]
name = "Cyberpunk 2077"
slug = "cyberpunk-2077"
rating = "gold"

[wine]
backend = "gptk"                # "wine" | "gptk"
min_version = "2.0"             # minimum Wine/GPTK version

[settings]
dxvk_enabled = false
env = { WINEESYNC = "1", MTL_HUD_ENABLED = "0" }

[workarounds]
notes = "Disable overlay if crashes on launch"

[install]
exe_path = "bin/x64/Cyberpunk2077.exe"     # relative to game root
exe_alternatives = ["Cyberpunk2077.exe"]    # fallback exe names for fuzzy matching
steam_appid = 1091500                       # optional, for store detection
gog_id = "cyberpunk_2077"                   # optional, for store detection
```

`steam_appid` and `gog_id` are optional metadata used only for auto-detection. Bundles work with any local folder — the `exe_path` and `exe_alternatives` fields are the primary matching mechanism.

### Data Updates

On app launch, optionally fetch latest `compatibility.json` from the repo's raw GitHub URL with a 24-hour cache. Falls back to the version bundled with the app binary if offline. No forced updates. Controlled by a config setting:

```toml
[app]
auto_update_database = true
database_url = "https://raw.githubusercontent.com/<org>/opengamecore/main/data/compatibility.json"
```

---

## 2. Store Detection (`opengamecore-lib/src/store_detect.rs`)

### Supported Stores

**Steam:**
- Scan `~/Library/Application Support/Steam/steamapps/` for `appmanifest_*.acf` files
- Parse each ACF (Valve KeyValues format) to extract `appid`, `name`, `installdir`
- Resolve install path: `steamapps/common/<installdir>/`
- Match `appid` against `compatibility.json` `steam_appid`

**GOG:**
- Check for GOG Galaxy at `~/Library/Application Support/GOG.com/Galaxy/Storage/`
- Scan common install paths: `/Applications/GOG Games/`, `~/GOG Games/`
- Match game folder names against compatibility database using fuzzy string matching (slug comparison + Levenshtein distance as fallback)

### Types

```rust
#[derive(Debug, Clone)]
pub enum GameStore {
    Steam,
    Gog,
}

#[derive(Debug, Clone)]
pub struct DetectedGame {
    pub name: String,
    pub store: GameStore,
    pub install_path: PathBuf,
    pub exe_path: Option<String>,
    pub compatibility: Option<CompatEntry>,
    pub bundle_available: bool,
}

/// Scan all supported stores, match against compatibility database.
pub fn detect_installed_games(
    compat_db: &CompatDatabase,
) -> Result<Vec<DetectedGame>>
```

### Fuzzy Matching for Non-Store Games

When a user adds a game via "From Folder", the app:

1. Scans the folder recursively (max depth 3) for `.exe` files
2. Matches exe filenames against `exe_path` and `exe_alternatives` across all bundles
3. If matched: auto-fills name, Wine backend, env vars, DXVK, everything from the bundle
4. If not matched: falls back to manual configuration (current behavior)

This makes the "add from folder" flow seamless — point at a game directory, the app recognizes it automatically.

---

## 3. Bundle Auto-Configuration (`opengamecore-lib/src/bundle.rs`)

### Types

```rust
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
    pub backend: String,        // "wine" | "gptk"
    pub min_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleSettings {
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
```

### Key Functions

```rust
/// Load all bundles from the data directory.
pub fn load_bundles(data_dir: &Path) -> Result<HashMap<String, BundleConfig>>

/// Load the compatibility database.
pub fn load_compat_database(data_dir: &Path) -> Result<CompatDatabase>

/// Find a matching bundle for a given game folder by scanning exes.
pub fn match_bundle_for_folder(
    folder: &Path,
    bundles: &HashMap<String, BundleConfig>,
) -> Option<BundleConfig>

/// Apply a bundle to create a fully configured game entry and bottle.
pub fn apply_bundle(
    bundle: &BundleConfig,
    install_path: &Path,
    library: &mut GameLibrary,
    template_dir: &Path,
) -> Result<String>  // returns slug
```

`apply_bundle` does everything:
1. Creates a `Game` with all fields pre-filled from the bundle
2. Creates a bottle from the template
3. Sets DXVK if required
4. Configures env vars
5. Adds to library and saves

---

## 4. GUI Changes

### New Sidebar Entry: "Game Database"

A new screen showing the full compatibility database as a searchable, filterable list.

- Search bar at the top
- Filter by rating (Platinum/Gold/Silver/Bronze/Borked)
- Filter by backend (Wine/GPTK)
- Each row shows: game name, rating badge (color-coded), backend badge, "Add" button if bundle available
- Tapping "Add" opens a folder picker (for the game files), then auto-applies the bundle

### Updated First-Run Flow

After Wine setup:
1. "Scanning for installed games..." — runs store detection
2. Shows results: "Found N games. X are compatible."
3. List of detected games with compatibility ratings
4. One-click "Set Up" button for games with bundles
5. "Browse All Games" link to the Game Database screen

### Updated "Add Game" Dialog

New fourth tab: **"Auto-Detect"**
- User picks a folder
- App scans for executables, matches against bundles
- If matched: shows the bundle info (rating, backend, settings) and "Add with Recommended Settings" button
- If not matched: falls back to manual config on the current tabs

---

## 5. GitHub Discussions Integration

### Compatibility Reports

A structured GitHub Discussion template for compatibility reports:

```markdown
### Game: [Game Name]
### macOS Version: [e.g., 14.3]
### Chip: [e.g., M3 Pro]
### Wine Backend: [Wine 9.0 / GPTK 2.0]
### Rating: [Platinum / Gold / Silver / Bronze / Borked]
### Notes:
[Describe your experience, any workarounds needed]
```

The app includes a "Report Compatibility" button on each game card that opens a pre-filled GitHub Discussion URL in the browser. Zero backend required.

### Community Bundle Contributions

Contributors submit new bundles as Pull Requests to `data/bundles/`. A simple CI check validates the TOML schema. Maintainers merge after review.

---

## 6. Data Update Module (`opengamecore-lib/src/data_update.rs`)

Handles fetching fresh compatibility data from GitHub:

```rust
/// Check if the local database is stale (>24h old) and fetch updates.
pub async fn check_and_update(
    config: &AppConfig,
    data_dir: &Path,
) -> Result<bool>  // returns true if updated

/// Fetch latest compatibility.json from the configured URL.
pub async fn fetch_compat_database(
    url: &str,
    dest: &Path,
) -> Result<()>
```

Uses the existing `reqwest` dependency. Writes atomically using `fs_utils::atomic_write`. Backs up the previous version before overwriting.

---

## 7. Config Changes

### AppConfig additions

```toml
[app]
auto_update_database = true
database_url = "https://raw.githubusercontent.com/<org>/opengamecore/main/data/compatibility.json"
bundles_url = "https://raw.githubusercontent.com/<org>/opengamecore/main/data/bundles/"
```

---

## 8. New Files Summary

### Lib crate
- `opengamecore-lib/src/compat.rs` — `CompatDatabase`, `CompatEntry`, `CompatRating` types, load/query functions
- `opengamecore-lib/src/bundle.rs` — `BundleConfig` types, load/match/apply functions
- `opengamecore-lib/src/store_detect.rs` — Steam/GOG detection, fuzzy matching
- `opengamecore-lib/src/data_update.rs` — fetch + cache compatibility data from GitHub

### App crate
- `opengamecore-app/src/views/game_database.rs` — compatibility browser view
- Updates to `app.rs` — new messages, new screen, store detection on first run
- Updates to `views/add_game.rs` — new "Auto-Detect" tab
- Updates to `views/first_run.rs` — game detection step after Wine setup

### Data directory
- `data/compatibility.json` — initial database (committed)
- `data/bundles/*.toml` — initial bundle configs (committed)

### CLI additions
- `ogc detect` — scan stores, show detected games with compatibility
- `ogc database` — browse/search compatibility database
- `ogc setup <slug>` — auto-configure a game from its bundle

---

## 9. Testing Strategy

- **compat.rs**: Unit tests for database loading, rating parsing, querying
- **bundle.rs**: Unit tests for TOML parsing, matching logic, apply_bundle with temp dirs
- **store_detect.rs**: Unit tests with mock Steam ACF files and GOG directories
- **data_update.rs**: Unit test for atomic update logic (not live HTTP)
- **Integration test**: Full flow — detect fake Steam game → match bundle → apply → verify game entry

---

## 10. Viral Loop

```
Google "can I play X on Mac"
        ↓
Find OpenGameCore compatibility page (GitHub README / Discussions)
        ↓
Download app → auto-detects their games → shows ratings
        ↓
One-click setup → game works → "wow"
        ↓
Star the repo → tell friends → submit compatibility report
        ↓
More reports → better data → more Google hits → repeat
```

The compatibility database in the repo is the SEO anchor. GitHub Discussions create indexable, searchable pages. The app delivers instant value by showing "which of YOUR games work on Mac" within 30 seconds of first launch.
