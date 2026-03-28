//! Integration tests for full OpenGameCore workflows.
//! These test complete user journeys using the lib crate.

use std::collections::HashMap;
use std::path::Path;

use opengamecore_lib::bottle;
use opengamecore_lib::config::AppConfig;
use opengamecore_lib::dxvk;
use opengamecore_lib::library::{
    export_library, import_library, slugify, Game, GameLibrary, InstallType,
};
use opengamecore_lib::runner::LaunchConfig;
use opengamecore_lib::wine;
use opengamecore_lib::WineConfig;

use tempfile::TempDir;

fn make_test_game(name: &str) -> Game {
    Game {
        name: name.into(),
        slug: slugify(name),
        exe: "drive_c/game/game.exe".into(),
        install_type: InstallType::Portable,
        wine_config: "default".into(),
        env: HashMap::new(),
        added_at: chrono::Utc::now(),
        last_played: None,
        icon_path: None,
        dxvk_enabled: false,
        use_gptk: false,
    }
}

/// Full workflow: add game → create bottle → verify → remove game → delete bottle
#[test]
fn full_game_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let games_path = tmp.path().join("games.toml");
    let bottles_dir = tmp.path().join("bottles");
    let template_dir = bottles_dir.join("_template");

    // Create a fake template bottle
    std::fs::create_dir_all(template_dir.join("drive_c/windows/system32")).unwrap();
    std::fs::write(template_dir.join("system.reg"), "fake registry").unwrap();

    // Add game to library
    let mut library = GameLibrary::default();
    let game = make_test_game("Test Game");
    let slug = game.slug.clone();
    library.add(game).unwrap();
    library.save(&games_path).unwrap();

    // Create bottle for the game
    let bottle_dir = bottles_dir.join(&slug);
    bottle::create(&template_dir, &bottle_dir).unwrap();
    assert!(bottle_dir.join("drive_c").exists());
    assert!(bottle_dir.join("system.reg").exists());

    // Verify game is in library
    let loaded = GameLibrary::load(&games_path).unwrap();
    assert_eq!(loaded.games.len(), 1);
    assert_eq!(loaded.find(&slug).unwrap().name, "Test Game");

    // List bottles
    let bottles = bottle::list(&bottles_dir).unwrap();
    assert_eq!(bottles.len(), 1);
    assert_eq!(bottles[0].slug, slug);

    // Remove game
    library.remove(&slug).unwrap();
    library.save(&games_path).unwrap();

    // Delete bottle
    bottle::delete(&bottle_dir).unwrap();
    assert!(!bottle_dir.exists());

    // Verify clean state
    let loaded = GameLibrary::load(&games_path).unwrap();
    assert!(loaded.games.is_empty());
    let bottles = bottle::list(&bottles_dir).unwrap();
    assert_eq!(bottles.len(), 0);
}

/// Workflow: bottle reset preserves template but wipes game data
#[test]
fn bottle_reset_workflow() {
    let tmp = TempDir::new().unwrap();
    let template = tmp.path().join("_template");
    std::fs::create_dir_all(template.join("drive_c")).unwrap();
    std::fs::write(template.join("system.reg"), "clean registry").unwrap();

    let bottle = tmp.path().join("my-game");
    bottle::create(&template, &bottle).unwrap();

    // Simulate game installing files
    std::fs::write(bottle.join("drive_c/game_data.sav"), "save data").unwrap();
    std::fs::write(bottle.join("corrupted_file"), "bad data").unwrap();
    assert!(bottle.join("corrupted_file").exists());

    // Reset
    bottle::reset(&template, &bottle).unwrap();

    // Template files restored, game-added files gone
    assert!(bottle.join("drive_c").exists());
    assert!(bottle.join("system.reg").exists());
    assert!(!bottle.join("corrupted_file").exists());
    assert!(!bottle.join("drive_c/game_data.sav").exists());
}

/// Workflow: config persistence with defaults and modifications
#[test]
fn config_persistence_workflow() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    // Load non-existent config returns defaults
    let config = AppConfig::load(&config_path).unwrap();
    assert!(!config.app.first_run_complete);
    assert!(!config.wine.download_urls.is_empty());

    // First save: writes the file (no backup yet, file didn't exist)
    let mut config = config;
    config.app.first_run_complete = true;
    config.wine.default = "wine-9.0".into();
    config.save(&config_path).unwrap();

    // Second save: now the file exists so a backup is created
    config.save(&config_path).unwrap();

    // Verify backup was created
    assert!(config_path.with_extension("bak").exists());

    // Reload and verify
    let loaded = AppConfig::load(&config_path).unwrap();
    assert!(loaded.app.first_run_complete);
    assert_eq!(loaded.wine.default, "wine-9.0");
}

/// Workflow: export and import library between users
#[test]
fn library_sharing_workflow() {
    let tmp = TempDir::new().unwrap();

    // User A creates library with 3 games
    let mut user_a = GameLibrary::default();
    user_a.add(make_test_game("Game Alpha")).unwrap();
    user_a.add(make_test_game("Game Beta")).unwrap();
    user_a.add(make_test_game("Game Gamma")).unwrap();

    let export_path = tmp.path().join("shared-library.toml");
    export_library(&user_a, &export_path).unwrap();

    // User B has 1 overlapping game and 1 unique game
    let mut user_b = GameLibrary::default();
    user_b.add(make_test_game("Game Alpha")).unwrap(); // overlap
    user_b.add(make_test_game("Game Delta")).unwrap(); // unique

    let imported = import_library(&mut user_b, &export_path).unwrap();

    // Should import Beta and Gamma (2), skip Alpha (duplicate)
    assert_eq!(imported, 2);
    assert_eq!(user_b.games.len(), 4);
    assert!(user_b.find("game-alpha").is_some());
    assert!(user_b.find("game-beta").is_some());
    assert!(user_b.find("game-gamma").is_some());
    assert!(user_b.find("game-delta").is_some());
}

/// Workflow: Wine discovery finds local installations
#[test]
fn wine_discovery_workflow() {
    let tmp = TempDir::new().unwrap();
    let wine_dir = tmp.path().join("wine");

    // Create fake Wine installations
    let wine9 = wine_dir.join("wine-9.0/bin");
    std::fs::create_dir_all(&wine9).unwrap();
    std::fs::write(wine9.join("wine64"), "fake wine binary").unwrap();

    let wine8 = wine_dir.join("wine-8.0/bin");
    std::fs::create_dir_all(&wine8).unwrap();
    std::fs::write(wine8.join("wine"), "fake wine binary").unwrap();

    let configs = wine::discover(&wine_dir).unwrap();

    // Should find both installations
    let names: Vec<&str> = configs.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"wine-9.0"));
    assert!(names.contains(&"wine-8.0"));

    // Resolve default picks first
    let resolved = wine::resolve(&configs, "default").unwrap();
    assert!(!resolved.name.is_empty());

    // Resolve by name
    let resolved = wine::resolve(&configs, "wine-9.0").unwrap();
    assert_eq!(resolved.name, "wine-9.0");
}

/// Workflow: DXVK install, verify, uninstall
#[test]
fn dxvk_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let bottle = tmp.path().join("bottle");
    std::fs::create_dir_all(bottle.join("drive_c/windows/system32")).unwrap();

    // Create fake DXVK directory
    let dxvk_dir = tmp.path().join("dxvk");
    let x64 = dxvk_dir.join("x64");
    std::fs::create_dir_all(&x64).unwrap();
    for dll in &["d3d9", "d3d10core", "d3d11", "dxgi"] {
        std::fs::write(x64.join(format!("{}.dll", dll)), "dxvk replacement").unwrap();
    }

    // Not installed initially (system32 is empty, no DLL files yet)
    assert!(!dxvk::is_installed(&bottle));

    // Place fake original DLLs (simulating a pre-existing Wine prefix)
    for dll in &["d3d9", "d3d10core", "d3d11", "dxgi"] {
        std::fs::write(
            bottle.join(format!("drive_c/windows/system32/{}.dll", dll)),
            "original wine dll",
        )
        .unwrap();
    }

    // Install
    dxvk::install(&dxvk_dir, &bottle).unwrap();
    assert!(dxvk::is_installed(&bottle));

    // Verify backups created
    for dll in &["d3d9", "d3d10core", "d3d11", "dxgi"] {
        let backup = bottle.join(format!("drive_c/windows/system32/{}.dll.orig", dll));
        assert!(backup.exists());
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), "original wine dll");
    }

    // Uninstall
    dxvk::uninstall(&bottle).unwrap();

    // Originals restored
    for dll in &["d3d9", "d3d10core", "d3d11", "dxgi"] {
        let path = bottle.join(format!("drive_c/windows/system32/{}.dll", dll));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "original wine dll");
    }
}

/// Workflow: LaunchConfig assembly with DXVK and custom env
#[test]
fn launch_config_assembly() {
    let wine = WineConfig {
        name: "wine-9.0".into(),
        binary_path: "/usr/local/bin/wine64".into(),
        env_overrides: HashMap::from([("WINEDEBUG".into(), "-all".into())]),
    };

    let game_env = HashMap::from([
        ("DXVK_HUD".into(), "fps".into()),
    ]);

    // Without DXVK
    let config = LaunchConfig::new(
        &wine,
        Path::new("/bottles/cyberpunk"),
        "drive_c/game/game.exe",
        &game_env,
        false,
    );
    assert_eq!(config.env.get("WINEDEBUG").unwrap(), "-all");
    assert_eq!(config.env.get("DXVK_HUD").unwrap(), "fps");
    assert!(config.env.get("WINEDLLOVERRIDES").is_none());

    // With DXVK
    let config = LaunchConfig::new(
        &wine,
        Path::new("/bottles/cyberpunk"),
        "drive_c/game/game.exe",
        &game_env,
        true,
    );
    assert!(config.env.get("WINEDLLOVERRIDES").is_some());
    assert!(config.env.get("WINEDLLOVERRIDES").unwrap().contains("d3d11=n"));
}

// ===== Viral features integration tests =====

/// Integration: load compatibility.json from repo data/, verify it parses and search works
#[test]
fn compat_database_from_repo() {
    let db_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("data/compatibility.json");
    let db = opengamecore_lib::compat::CompatDatabase::load(&db_path).unwrap();

    assert!(db.games.len() >= 3, "expected at least 3 seed games");
    assert_eq!(db.version, 1);

    // Search works
    let results = db.search("cyberpunk");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].slug, "cyberpunk-2077");

    // Lookup by steam appid
    let entry = db.find_by_steam_appid(413150);
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().name, "Stardew Valley");

    // Filter by rating
    let platinum = db.filter_by_rating(&opengamecore_lib::CompatRating::Platinum);
    assert!(platinum.len() >= 2);
}

/// Integration: load bundles from data/bundles/, match against fake folder, apply to library
#[test]
fn bundle_load_match_apply() {
    let bundles_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("data/bundles");
    let bundles = opengamecore_lib::bundle::load_bundles(&bundles_dir).unwrap();

    assert!(bundles.len() >= 3, "expected at least 3 seed bundles");
    assert!(bundles.contains_key("stardew-valley"));

    // Create a fake game folder with matching exe
    let tmp = TempDir::new().unwrap();
    let game_dir = tmp.path().join("Stardew Valley");
    std::fs::create_dir_all(&game_dir).unwrap();
    std::fs::write(game_dir.join("Stardew Valley.exe"), "fake").unwrap();

    // Match
    let matched = opengamecore_lib::bundle::match_bundle_for_folder(&game_dir, &bundles);
    assert!(matched.is_some());
    let bundle = matched.unwrap();
    assert_eq!(bundle.game.slug, "stardew-valley");

    // Apply
    let mut lib = GameLibrary::default();
    let slug = opengamecore_lib::bundle::apply_bundle(&bundle, &game_dir, &mut lib).unwrap();
    assert_eq!(slug, "stardew-valley");
    assert!(lib.find("stardew-valley").is_some());

    let game = lib.find("stardew-valley").unwrap();
    assert_eq!(game.wine_config, "default");
}

/// Integration: mock Steam ACF → detect → match against compat DB
#[test]
fn store_detection_pipeline() {
    let db_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("data/compatibility.json");
    let db = opengamecore_lib::compat::CompatDatabase::load(&db_path).unwrap();

    // Create mock Steam directory
    let tmp = TempDir::new().unwrap();
    let steamapps = tmp.path().join("steamapps");
    std::fs::create_dir_all(steamapps.join("common/Cyberpunk 2077")).unwrap();
    std::fs::write(
        steamapps.join("appmanifest_1091500.acf"),
        r#"
"AppState"
{
	"appid"		"1091500"
	"name"		"Cyberpunk 2077"
	"installdir"		"Cyberpunk 2077"
}
"#,
    )
    .unwrap();

    let games =
        opengamecore_lib::store_detect::detect_steam_games(&steamapps, &db).unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].name, "Cyberpunk 2077");
    assert_eq!(
        games[0].rating,
        Some(opengamecore_lib::CompatRating::Gold)
    );
    assert!(games[0].bundle_available);
}

/// Integration: full pipeline — detect → find bundle → apply → verify library
#[test]
fn full_viral_pipeline() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let db = opengamecore_lib::compat::CompatDatabase::load(
        &repo_root.join("data/compatibility.json"),
    )
    .unwrap();
    let bundles =
        opengamecore_lib::bundle::load_bundles(&repo_root.join("data/bundles")).unwrap();

    // Create mock Steam directory
    let tmp = TempDir::new().unwrap();
    let steamapps = tmp.path().join("steamapps");
    let game_dir = steamapps.join("common/Stardew Valley");
    std::fs::create_dir_all(&game_dir).unwrap();
    std::fs::write(game_dir.join("Stardew Valley.exe"), "fake").unwrap();
    std::fs::write(
        steamapps.join("appmanifest_413150.acf"),
        r#"
"AppState"
{
	"appid"		"413150"
	"name"		"Stardew Valley"
	"installdir"		"Stardew Valley"
}
"#,
    )
    .unwrap();

    // Detect
    let detected =
        opengamecore_lib::store_detect::detect_steam_games(&steamapps, &db).unwrap();
    assert_eq!(detected.len(), 1);
    assert!(detected[0].bundle_available);

    // Find matching bundle
    let matched = opengamecore_lib::bundle::match_bundle_for_folder(&game_dir, &bundles);
    assert!(matched.is_some());

    // Apply
    let mut lib = GameLibrary::default();
    let slug = opengamecore_lib::bundle::apply_bundle(
        &matched.unwrap(),
        &game_dir,
        &mut lib,
    )
    .unwrap();

    // Verify
    let game = lib.find(&slug).unwrap();
    assert_eq!(game.name, "Stardew Valley");
    assert_eq!(game.slug, "stardew-valley");
}

/// Workflow: crash recovery — empty config file restored from backup
#[test]
fn crash_recovery_workflow() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    // First save: writes the file (no backup created yet)
    let mut config = AppConfig::default();
    config.app.first_run_complete = true;
    config.save(&config_path).unwrap();

    // Second save: file now exists, so a backup (.bak) is created before overwriting
    config.save(&config_path).unwrap();

    // Simulate crash: truncate the config file to empty
    std::fs::write(&config_path, "").unwrap();

    // Load should detect empty file and recover from backup
    let loaded = AppConfig::load(&config_path).unwrap();
    assert!(loaded.app.first_run_complete);
}
