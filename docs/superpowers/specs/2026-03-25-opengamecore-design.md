# OpenGameCore — Design Spec

A simplified, open-source Lutris alternative for macOS. Lets users run Windows games through Wine with a clean native launcher. Built in Rust for performance.

## Goals

- Make running Windows games on macOS as easy as possible for non-technical users
- Auto-download Wine on first run — value immediately, no Homebrew required
- Clean sidebar + grid UI for browsing and launching games
- One Wine bottle (prefix) per game for isolation
- Flexible Wine backend — users can plug in upstream Wine, GPTK, or custom builds
- Open-source and community-friendly

## Non-Goals (v1)

- No support for general Windows apps (games only)
- No cloud saves or sync
- No automatic game metadata/cover art fetching
- No Linux or Windows support (macOS only)
- No plugin system

## Architecture

### Workspace: Two Crates

**`opengamecore-lib`** — core library, no UI dependencies. Reusable by CLI tools or alternative frontends.

**`opengamecore-app`** — iced-based GUI that depends on `opengamecore-lib`.

### opengamecore-lib Modules

#### Wine Manager

Discovers and manages Wine installations.

- Scans known locations for Wine binaries (Homebrew, `~/Library/Application Support/OpenGameCore/wine/`)
- Auto-downloads Wine builds from configurable URLs on first run. Expects `.tar.xz` archives containing a `wine` binary at a known relative path (e.g., `wine-<version>/bin/wine`). The app extracts and locates the binary automatically.
- Wine backends are represented as a simple config struct:

```rust
pub struct WineConfig {
    pub name: String,
    pub binary_path: PathBuf,
    pub env_overrides: HashMap<String, String>,
}
```

No trait abstraction — the difference between Wine flavors is which binary to call and which env vars to set. This can be promoted to a trait later if genuinely different behaviors emerge.

Download sources are configurable in `config.toml` so the community can point to alternative builds without changing code.

#### Bottle Manager

Creates and manages isolated Wine prefixes (bottles).

- Each game gets its own bottle under `~/Library/Application Support/OpenGameCore/bottles/<game-slug>/`
- A `_template` prefix is created once during first run and used as the base for all new bottles
- New bottles are created via APFS `clonefile` from the template — near-instant, near-zero disk cost on macOS. Falls back to a regular directory copy if `clonefile` fails (e.g., non-APFS volumes).
- Operations: create (clone from template), list, delete, **reset** (delete + re-clone from template)
- Reset handles corrupted prefixes without losing the game entry from the library

#### Game Library

Stores game metadata in `games.toml`.

- Three ways to add a game:
  1. **Installer** — pick a `.exe` installer, run it inside a new bottle
  2. **Portable folder** — point to a folder containing the game, link/copy into bottle's `drive_c`
  3. **Install from folder** — point to a directory containing a setup program, run it inside a new bottle
- Each game entry tracks: name, slug, exe path (relative to bottle), install type, wine config name, custom env vars, timestamps (added, last played)

#### Runner

Launches games by assembling the right Wine binary + prefix + exe + environment.

- Spawns the Wine process with correct `WINEPREFIX`, binary path, and env vars
- Captures stdout/stderr for debugging
- Reports process state (running, exited, crashed) back to the UI

### opengamecore-app Screens

#### Layout

Sidebar + Grid layout:
- **Left sidebar** — navigation sections: All Games, Recently Played, Bottles
- **Main area** — grid of game cards (icon, name, play button) or detail views
- **Top bar** — app title, Add Game button, Settings button

#### Screens & Flows

1. **First Run** — "No Wine found. Download Wine?" with progress bar. Creates `_template` prefix. Transitions to empty library.

2. **Game Library (main screen)** — Grid of game cards. Each card shows icon (or placeholder), game name, and a Play button. Clicking Play launches the game; card shows "Running..." state while active.

3. **Add Game Dialog** — Three tabs/modes:
   - **Installer**: file picker for `.exe`, name field, confirm → clones bottle, runs installer
   - **Portable**: folder picker, name field, confirm → clones bottle, symlinks game folder into `drive_c` (preserves original location; falls back to copy if symlink fails)
   - **From Folder**: folder picker (expects setup inside), name field, confirm → clones bottle, runs setup

4. **Bottle Detail** — accessed from Bottles sidebar section. Shows: game name, Wine version, prefix path, disk usage. Actions: Open in Finder, Reset Bottle, Delete Bottle.

5. **Settings** — Wine installations management (list installed, add custom path, download new version, set default). Download URL configuration.

## Data Layout

```
~/Library/Application Support/OpenGameCore/
  config.toml              # app settings
  games.toml               # game library metadata
  bottles/
    _template/             # base Wine prefix, cloned for new games
    <game-slug>/           # APFS clone of template + game install
  wine/
    <version-name>/        # downloaded Wine builds
  icons/
    <game-slug>.png        # game icons
```

### config.toml

```toml
[wine]
default = "wine-9.0-macos"
download_urls = [
  "https://github.com/Gcenx/macOS_Wine_builds/releases/download/v9.0/wine-devel-9.0-osx64.tar.xz"
]

[app]
first_run_complete = true
```

### games.toml

```toml
[[games]]
name = "Cyberpunk 2077"
slug = "cyberpunk-2077"
exe = "drive_c/GOG Games/Cyberpunk 2077/bin/x64/Cyberpunk2077.exe"
install_type = "installer"
wine_config = "default"
env = { DXVK_HUD = "1" }
added_at = 2026-03-25T12:00:00Z
last_played = 2026-03-25T14:30:00Z
```

## Key Dependencies

- **iced** — Rust GUI framework (cross-platform, pure Rust)
- **tokio** — async runtime for Wine process management and downloads
- **serde + toml** — config/game library serialization
- **dirs** — platform-appropriate directory resolution
- **reqwest** — HTTP client for Wine downloads

## Design Decisions & Rationale

| Decision | Rationale |
|----------|-----------|
| TOML over SQLite | Human-readable, simple for v1, easy to hand-edit. Migrate if scale demands it. |
| Config struct over trait for Wine backends | Backends differ only in binary path + env vars. Trait is premature abstraction. |
| APFS clonefile for bottles | Makes one-bottle-per-game nearly free on disk. macOS-specific but this is a macOS app. |
| Template prefix | Initialize Wine once, clone instantly. Avoids slow `wineboot` for every new game. |
| Configurable download URLs | No hardcoded dependency on a single host. Community can fork and point elsewhere. |
| macOS-native paths | `~/Library/Application Support/` is where macOS apps store data. Respects platform conventions. |
| Two-crate workspace | Library is reusable for CLI tools or alternative frontends. Good for open-source contributions. |

## Error Handling

- **Wine download fails** — retry with backoff, show error with manual download instructions
- **Bottle creation fails** — surface Wine error output, suggest resetting template prefix
- **Game crashes** — capture Wine stderr, show in a log viewer accessible from the game card
- **Corrupted bottle** — Reset Bottle action re-clones from template (game library entry preserved)
- **Missing Wine binary** — prompt user to download or configure a Wine path in Settings

## Future Considerations (not v1)

- Cover art fetching (IGDB, SteamGridDB)
- Game-specific Wine configuration presets (community-maintained)
- CLI companion tool using `opengamecore-lib`
- DXVK/MoltenVK integration toggles per game
- Import/export game library for backup
