# OpenGameCore

A macOS Wine game launcher written in Rust — clean UI, fast bottle cloning, and DXVK out of the box.

<!-- TODO: Add screenshot -->

## Features

- **Wine management** — auto-discover, download, and switch between multiple Wine builds
- **APFS bottle cloning** — near-instant per-game Wine prefixes via `clonefile`
- **DXVK integration** — toggle DirectX→Vulkan→Metal translation per game
- **Game library** — TOML-based storage with import/export and cover art support
- **Data safety** — atomic writes, automatic backups, and crash recovery
- **Process monitoring** — log capture, running state tracking, and crash detection
- **CLI companion** — full `ogc` CLI for scripting and automation

## Requirements

- macOS 12 Monterey or later
- Rust 1.75+
- Wine (via Homebrew or bundled build)

## Quick Start

```sh
git clone https://github.com/your-org/opengamecore.git
cd opengamecore
cargo build --release
```

Run the GUI app:

```sh
cargo run -p opengamecore-app --release
```

Run the CLI:

```sh
cargo run -p opengamecore-cli --release -- --help
```

## CLI Usage

The `ogc` command provides full access to your game library from the terminal.

```sh
# List all games
ogc list

# Add a game
ogc add "Game Name" /path/to/game.exe

# Run a game
ogc run "Game Name"

# Show game details
ogc info "Game Name"

# Manage bottles
ogc bottles
ogc reset-bottle "Game Name"

# Wine passthrough
ogc wine -- winecfg

# Import / export library
ogc export library.toml
ogc import library.toml

# Remove a game
ogc remove "Game Name"
```

## How It Works

**Wine bottles** — each game gets its own isolated Wine prefix so settings, registry entries, and DLLs never bleed between titles.

**APFS cloning** — new bottles are created with macOS `clonefile`, making copies near-instant and space-efficient (copy-on-write at the filesystem level).

**TOML config** — the game library and all settings are stored as plain TOML files. No database, no binary formats — easy to inspect, back up, or version control.

## Configuration

All data lives under `~/Library/Application Support/OpenGameCore/`:

```
~/Library/Application Support/OpenGameCore/
├── config.toml          # App settings
├── library.toml         # Game library
├── bottles/             # Per-game Wine prefixes
│   └── <game-id>/
├── logs/                # Per-game Wine logs
└── backups/             # Auto-generated backups
```

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Commit your changes following the existing style
4. Open a pull request against `main`

Please keep PRs focused. Bug fixes and well-scoped features are welcome. For larger changes, open an issue first to discuss the approach.

## License

MIT — see [LICENSE](LICENSE) for details.
