# OpenGameCore

A macOS-native Wine game launcher written in Rust. Manage Windows games with per-game bottles, DXVK integration, and a built-in compatibility database — all from a clean native UI or the `ogc` CLI.

## Features

- **Wine management** — auto-discover, download, and switch between multiple Wine builds
- **APFS bottle cloning** — near-instant per-game Wine prefixes via `clonefile`
- **DXVK integration** — toggle DirectX-to-Vulkan-to-Metal translation per game
- **Game library** — TOML-based storage with import/export and cover art support
- **Compatibility database** — built-in ratings for popular games (Platinum/Gold/Silver/Bronze/Borked)
- **Steam/GOG auto-detection** — scan installed games and show compatibility at a glance
- **One-click bundles** — pre-configured Wine settings for popular games
- **Data safety** — atomic writes, automatic backups, and crash recovery
- **CLI companion** — full `ogc` CLI for scripting and automation

## Requirements

- macOS 12 Monterey or later (Apple Silicon recommended)
- Rust 1.75+ (for building from source)

## Build from Source

```sh
git clone https://github.com/rzgrw/opengamecore.git
cd opengamecore
cargo build --release
```

Run the GUI:

```sh
cargo run -p opengamecore-app --release
```

Run the CLI:

```sh
cargo run -p opengamecore-cli --release -- --help
```

## CLI Usage

```sh
# Library management
ogc list                          # List all games
ogc add -n "Game" -e /path/to.exe # Add a game
ogc remove my-game                # Remove a game
ogc run my-game                   # Launch a game
ogc run my-game --dxvk            # Launch with DXVK

# Bottles and Wine
ogc bottles                       # List bottles
ogc reset-bottle my-game          # Reset a game's bottle
ogc wine                          # List Wine installations

# Import / export
ogc export library.toml
ogc import library.toml

# Compatibility database
ogc database                      # List all rated games
ogc database "cyberpunk"          # Search by name
ogc database --rating gold        # Filter by rating

# Auto-detection
ogc detect                        # Scan Steam/GOG for installed games

# One-click setup from bundle
ogc setup stardew-valley --path ~/Games/StardewValley

# Info
ogc info                          # Show app directories
```

## Game Compatibility

OpenGameCore ships with a compatibility database rating popular Windows games on macOS. Each game has a rating, recommended Wine backend, and optional pre-configured bundle.

| Rating | Meaning |
|--------|---------|
| Platinum | Works perfectly out of the box |
| Gold | Playable with minor tweaks |
| Silver | Playable with workarounds |
| Bronze | Runs but has significant issues |
| Borked | Does not work |

**Browse:** In the app sidebar click "Game Database", or run `ogc database` in the CLI.

**Auto-detect:** The first-run wizard scans Steam and GOG automatically. Or run `ogc detect`.

**One-click setup:** Games with bundles can be configured in one step — the bundle sets the correct Wine backend, environment variables, and workarounds.

### Contributing compatibility data

**Report a game:** Open a [Discussion](https://github.com/rzgrw/opengamecore/discussions/new?category=compatibility-report) with your game name, macOS version, chip, backend, and rating.

**Add a bundle:** Create a TOML file in `data/bundles/` matching the format of existing bundles, and open a PR. CI will validate the format automatically.

## Architecture

```
opengamecore-lib/     # Core library — config, bottles, Wine, compat DB, bundles
opengamecore-app/     # iced GUI application
opengamecore-cli/     # ogc CLI tool
data/
  compatibility.json  # Game compatibility database
  bundles/            # Per-game TOML configs
```

All user data lives under `~/Library/Application Support/OpenGameCore/`:

```
config.toml           # App settings
games.toml            # Game library
compatibility.json    # Cached compat database
bottles/              # Per-game Wine prefixes
wine/                 # Wine installations
icons/                # Game icons
logs/                 # Game run logs
bundles/              # Bundle configs
```

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Commit your changes following the existing style
4. Open a pull request against `main`

Please keep PRs focused. Bug fixes and well-scoped features are welcome. For larger changes, open an issue first to discuss the approach.

## License

MIT
