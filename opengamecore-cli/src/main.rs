use clap::{Parser, Subcommand};
use std::path::PathBuf;

use opengamecore_lib::{
    bottle, library, paths, runner, wine, Game, GameLibrary, InstallType, LaunchConfig,
};

/// Exit code for user errors (bad input, game not found, etc.)
const EXIT_USER_ERROR: i32 = 1;
/// Exit code for system errors (IO, permissions, etc.)
const EXIT_SYSTEM_ERROR: i32 = 2;

#[derive(Parser)]
#[command(name = "ogc", about = "OpenGameCore CLI - Wine game launcher for macOS")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all games in the library
    List,

    /// Add a game to the library
    Add {
        /// Game name
        #[arg(short, long)]
        name: String,

        /// Path to the game executable
        #[arg(short, long)]
        exe: String,

        /// Install type: installer, portable, or folder
        #[arg(short, long, default_value = "portable")]
        install_type: String,

        /// Optional icon path
        #[arg(long)]
        icon: Option<PathBuf>,
    },

    /// Remove a game from the library
    Remove {
        /// Game slug (use 'ogc list' to see slugs)
        slug: String,
    },

    /// Run/launch a game
    Run {
        /// Game slug
        slug: String,

        /// Enable DXVK
        #[arg(long)]
        dxvk: bool,
    },

    /// List available Wine installations
    Wine,

    /// List bottles
    Bottles,

    /// Reset a game's bottle
    ResetBottle {
        /// Game slug
        slug: String,
    },

    /// Export game library to a file
    Export {
        /// Output file path
        #[arg(default_value = "opengamecore-library.toml")]
        path: PathBuf,
    },

    /// Import games from a library file
    Import {
        /// Input file path
        path: PathBuf,
    },

    /// Show app directories and config
    Info,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Ensure app directories exist
    if let Err(e) = paths::ensure_dirs() {
        eprintln!("Error creating app directories: {}", e.user_message());
        std::process::exit(EXIT_SYSTEM_ERROR);
    }

    match cli.command {
        Commands::List => cmd_list(),
        Commands::Add { name, exe, install_type, icon } => {
            cmd_add(&name, &exe, &install_type, icon.as_deref())
        }
        Commands::Remove { slug } => cmd_remove(&slug),
        Commands::Run { slug, dxvk } => cmd_run(&slug, dxvk).await,
        Commands::Wine => cmd_wine(),
        Commands::Bottles => cmd_bottles(),
        Commands::ResetBottle { slug } => cmd_reset_bottle(&slug),
        Commands::Export { path } => cmd_export(&path),
        Commands::Import { path } => cmd_import(&path),
        Commands::Info => cmd_info(),
    }
}

fn load_library() -> GameLibrary {
    let games_path = match paths::games_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving games path: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };
    match GameLibrary::load(&games_path) {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("Error loading library: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    }
}

fn save_library(lib: &GameLibrary) {
    let games_path = match paths::games_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving games path: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };
    if let Err(e) = lib.save(&games_path) {
        eprintln!("Error saving library: {}", e.user_message());
        std::process::exit(EXIT_SYSTEM_ERROR);
    }
}

fn cmd_list() {
    let lib = load_library();
    if lib.games.is_empty() {
        println!("No games in library. Use 'ogc add' to add a game.");
        return;
    }
    println!("{:<20} {:<30} {:<40} {:<15} {}", "SLUG", "NAME", "EXE", "WINE", "DXVK");
    println!("{}", "-".repeat(110));
    for game in &lib.games {
        println!(
            "{:<20} {:<30} {:<40} {:<15} {}",
            game.slug,
            game.name,
            game.exe,
            game.wine_config,
            if game.dxvk_enabled { "yes" } else { "no" }
        );
    }
}

fn cmd_add(name: &str, exe: &str, install_type_str: &str, icon: Option<&std::path::Path>) {
    if name.trim().is_empty() {
        eprintln!("Error: Game name cannot be empty.");
        std::process::exit(EXIT_USER_ERROR);
    }

    if exe.trim().is_empty() {
        eprintln!("Error: Game executable path cannot be empty.");
        std::process::exit(EXIT_USER_ERROR);
    }

    let install_type = match install_type_str {
        "installer" => InstallType::Installer,
        "portable" => InstallType::Portable,
        "folder" => InstallType::FolderInstall,
        other => {
            eprintln!(
                "Unknown install type '{}'. Use: installer, portable, or folder",
                other
            );
            std::process::exit(EXIT_USER_ERROR);
        }
    };

    let slug = library::slugify(name);

    let mut lib = load_library();

    // Check for duplicate slug
    if lib.find(&slug).is_some() {
        eprintln!("A game with slug '{}' already exists.", slug);
        std::process::exit(EXIT_USER_ERROR);
    }

    let game = Game {
        name: name.to_string(),
        slug: slug.clone(),
        exe: exe.to_string(),
        install_type,
        wine_config: "default".to_string(),
        env: std::collections::HashMap::new(),
        added_at: chrono::Utc::now(),
        last_played: None,
        icon_path: None,
        dxvk_enabled: false,
    };

    lib.add(game);
    save_library(&lib);

    // Create bottle from template
    let template_dir = match paths::template_bottle_dir() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving template bottle dir: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };
    let bottle_dir = match paths::bottle_dir(&slug) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving bottle dir: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };

    if template_dir.exists() {
        match bottle::create(&template_dir, &bottle_dir) {
            Ok(()) => println!("Bottle created for '{}'.", slug),
            Err(e) => eprintln!("Warning: could not create bottle: {}", e.user_message()),
        }
    } else {
        println!(
            "Note: No template bottle found at {}. Run wine setup first to initialize a template.",
            template_dir.display()
        );
    }

    // Copy icon if provided
    if let Some(icon_path) = icon {
        match library::set_game_icon(&slug, icon_path) {
            Ok(dest) => {
                // Update the game record with the icon path
                let mut lib2 = load_library();
                if let Some(g) = lib2.find_mut(&slug) {
                    g.icon_path = Some(dest.to_string_lossy().to_string());
                }
                save_library(&lib2);
                println!("Icon copied.");
            }
            Err(e) => eprintln!("Warning: could not copy icon: {}", e.user_message()),
        }
    }

    println!("Game '{}' added with slug '{}'.", name, slug);
}

fn cmd_remove(slug: &str) {
    let mut lib = load_library();

    if let Err(e) = lib.remove(slug) {
        eprintln!("Error: {}", e.user_message());
        std::process::exit(EXIT_USER_ERROR);
    }

    save_library(&lib);

    // Optionally delete the bottle
    let bottle_dir = match paths::bottle_dir(slug) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Warning: could not resolve bottle dir: {}", e.user_message());
            println!("Game '{}' removed from library.", slug);
            return;
        }
    };

    if bottle_dir.exists() {
        match bottle::delete(&bottle_dir) {
            Ok(()) => println!("Bottle deleted for '{}'.", slug),
            Err(e) => eprintln!("Warning: could not delete bottle: {}", e.user_message()),
        }
    }

    println!("Game '{}' removed.", slug);
}

async fn cmd_run(slug: &str, dxvk: bool) {
    let lib = load_library();

    let game = match lib.find(slug) {
        Some(g) => g,
        None => {
            eprintln!("Game '{}' not found. Use 'ogc list' to see available games.", slug);
            std::process::exit(EXIT_USER_ERROR);
        }
    };

    let wine_dir = match paths::wine_dir() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving wine dir: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };

    let wine_configs = match wine::discover(&wine_dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error discovering Wine installations: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };

    let wine_config = match wine::resolve(&wine_configs, &game.wine_config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e.user_message());
            std::process::exit(EXIT_USER_ERROR);
        }
    };

    let bottle_dir = match paths::bottle_dir(slug) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving bottle dir: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };

    let dxvk_enabled = dxvk || game.dxvk_enabled;
    let launch_config = LaunchConfig::new(&wine_config, &bottle_dir, &game.exe, &game.env, dxvk_enabled);

    println!(
        "Launching '{}' with Wine at {}...",
        game.name,
        launch_config.wine_binary.display()
    );

    let mut child = match runner::spawn(&launch_config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error spawning game: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };

    match child.wait().await {
        Ok(status) => {
            if status.success() {
                println!("Game exited successfully.");
            } else {
                eprintln!("Game exited with status: {}", status);
                std::process::exit(EXIT_SYSTEM_ERROR);
            }
        }
        Err(e) => {
            eprintln!("Error waiting for game process: {}", e);
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    }
}

fn cmd_wine() {
    let wine_dir = match paths::wine_dir() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving wine dir: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };

    let configs = match wine::discover(&wine_dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error discovering Wine installations: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };

    if configs.is_empty() {
        println!("No Wine installations found.");
        println!("Place Wine builds in: {}", wine_dir.display());
        return;
    }

    println!("{:<30} {}", "NAME", "BINARY PATH");
    println!("{}", "-".repeat(80));
    for cfg in &configs {
        println!("{:<30} {}", cfg.name, cfg.binary_path.display());
    }
}

fn cmd_bottles() {
    let bottles_dir = match paths::bottles_dir() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving bottles dir: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };

    let bottles = match bottle::list(&bottles_dir) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error listing bottles: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };

    if bottles.is_empty() {
        println!("No bottles found.");
        return;
    }

    println!("{:<30} {:<15} {}", "SLUG", "SIZE", "PATH");
    println!("{}", "-".repeat(90));
    for b in &bottles {
        let size_mb = b.size_bytes as f64 / (1024.0 * 1024.0);
        println!("{:<30} {:<15} {}", b.slug, format!("{:.1} MB", size_mb), b.path.display());
    }
}

fn cmd_reset_bottle(slug: &str) {
    // Validate the game exists before attempting reset
    let lib = load_library();
    if lib.find(slug).is_none() {
        eprintln!("Game '{}' not found. Use 'ogc list' to see available games.", slug);
        std::process::exit(EXIT_USER_ERROR);
    }

    let template_dir = match paths::template_bottle_dir() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving template bottle dir: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };

    if !template_dir.exists() {
        eprintln!(
            "Template bottle not found at {}. Run wine setup first.",
            template_dir.display()
        );
        std::process::exit(EXIT_USER_ERROR);
    }

    let bottle_dir = match paths::bottle_dir(slug) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving bottle dir: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };

    match bottle::reset(&template_dir, &bottle_dir) {
        Ok(()) => println!("Bottle for '{}' has been reset.", slug),
        Err(e) => {
            eprintln!("Error resetting bottle: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    }
}

fn cmd_export(path: &std::path::Path) {
    let lib = load_library();

    match library::export_library(&lib, path) {
        Ok(()) => println!(
            "Library exported to '{}' ({} games).",
            path.display(),
            lib.games.len()
        ),
        Err(e) => {
            eprintln!("Error exporting library: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    }
}

fn cmd_import(path: &std::path::Path) {
    if !path.exists() {
        eprintln!("Import file not found: {}", path.display());
        std::process::exit(EXIT_USER_ERROR);
    }

    let mut lib = load_library();

    let count = match library::import_library(&mut lib, path) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error importing library: {}", e.user_message());
            std::process::exit(EXIT_SYSTEM_ERROR);
        }
    };

    save_library(&lib);
    println!("Imported {} game(s) from '{}'.", count, path.display());
}

fn cmd_info() {
    macro_rules! print_path {
        ($label:expr, $fn:expr) => {
            match $fn {
                Ok(p) => println!("{:<20} {}", $label, p.display()),
                Err(e) => println!("{:<20} ERROR: {}", $label, e),
            }
        };
    }

    println!("OpenGameCore directories and configuration:");
    println!("{}", "-".repeat(60));
    print_path!("data_dir:", paths::data_dir());
    print_path!("config_path:", paths::config_path());
    print_path!("games_path:", paths::games_path());
    print_path!("bottles_dir:", paths::bottles_dir());
    print_path!("wine_dir:", paths::wine_dir());
}
