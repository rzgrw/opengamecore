use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use iced::widget::{button, column, container, row, text};
use iced::{Background, Border, Element, Length, Task, Theme};

use crate::views;
use crate::views::add_game::{AddGameState, AddGameTab};
use crate::views::first_run::FirstRunPhase;
use opengamecore_lib::bottle::BottleInfo;
use opengamecore_lib::bundle::BundleConfig;
use opengamecore_lib::store_detect::DetectedGame;
use opengamecore_lib::{
    AppConfig, CompatDatabase, CompatRating, Game, GameLibrary, InstallType, LaunchConfig,
    WineConfig,
};

#[derive(Debug, Clone)]
pub enum Screen {
    FirstRun,
    Library,
    Database,
    Bottles,
    Settings,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    // Navigation
    NavigateTo(Screen),
    Loaded(Box<AppState>),

    // Add Game
    OpenAddGame,
    CloseAddGame,
    AddGameTabChanged(AddGameTab),
    AddGameNameChanged(String),
    AddGameBrowse,
    AddGamePathSelected(Option<String>),
    AddGameBrowseIcon,
    AddGameIconSelected(Option<String>),
    ConfirmAddGame,

    // Game actions
    PlayGame(String),
    GameExited(Box<opengamecore_lib::runner::RunResult>),

    // Bottle actions
    ResetBottle(String),
    DeleteBottle(String),
    BottlesLoaded(Vec<BottleInfo>),

    // Settings / Wine
    SetDefaultWine(String),
    AddCustomWinePath,
    CustomWinePathSelected(Option<String>),

    // DXVK
    ToggleGameDxvk(String),
    DownloadDxvk,
    DxvkDownloaded(Option<PathBuf>),

    // Library import/export
    ExportLibrary,
    ImportLibrary,
    ExportLibraryPath(Option<String>),
    ImportLibraryPath(Option<String>),
    LibraryImported(usize),

    // Errors
    ShowError(String),
    DismissError,

    // First Run
    StartFirstRun,
    SkipFirstRun,
    FinishFirstRun,
    FirstRunProgress(f32, String),
    FirstRunTemplateCreating,
    FirstRunComplete,
    FirstRunError(String),

    // Database
    SearchChanged(String),
    FilterRating(Option<CompatRating>),
    SetupFromDatabase(String),
    SetupFolderSelected(String, Option<String>),

    // Store detection
    DetectGames,
    GamesDetected(Vec<DetectedGame>),

    // Bundle
    ApplyBundle(String),
    BundleApplied(String),

    // Auto-detect in add game
    AutoDetectFolder,
    AutoDetectResult(Box<Option<BundleConfig>>),

    // Steam
    InstallSteam,
    SteamInstalled(Result<(), String>),

    // Data update
    DatabaseUpdated(bool),
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub library: GameLibrary,
    pub wine_configs: Vec<WineConfig>,
    pub bottles: Vec<BottleInfo>,
    pub load_warnings: Vec<String>,
    pub compat_db: Option<CompatDatabase>,
    pub bundles: HashMap<String, BundleConfig>,
    pub dxvk_dir: Option<PathBuf>,
}

pub struct App {
    screen: Screen,
    config: AppConfig,
    library: GameLibrary,
    loading: bool,
    add_game: Option<AddGameState>,
    bottles: Vec<BottleInfo>,
    wine_configs: Vec<WineConfig>,
    first_run_phase: FirstRunPhase,
    dxvk_dir: Option<PathBuf>,
    error_message: Option<String>,
    running_games: HashSet<String>,
    compat_db: Option<CompatDatabase>,
    bundles: HashMap<String, BundleConfig>,
    detected_games: Vec<DetectedGame>,
    db_search_query: String,
    db_filter_rating: Option<CompatRating>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            screen: Screen::Library,
            config: AppConfig::default(),
            library: GameLibrary::default(),
            loading: true,
            add_game: None,
            bottles: Vec::new(),
            wine_configs: Vec::new(),
            first_run_phase: FirstRunPhase::default(),
            dxvk_dir: None,
            error_message: None,
            running_games: HashSet::new(),
            compat_db: None,
            bundles: HashMap::new(),
            detected_games: Vec::new(),
            db_search_query: String::new(),
            db_filter_rating: None,
        };

        let task = Task::perform(
            async {
                let mut load_warnings = Vec::new();

                let config = match opengamecore_lib::paths::config_path()
                    .and_then(|p| AppConfig::load(&p))
                {
                    Ok(c) => c,
                    Err(e) => {
                        load_warnings.push(format!("Failed to load config: {}", e.user_message()));
                        AppConfig::default()
                    }
                };

                let library = match opengamecore_lib::paths::games_path()
                    .and_then(|p| GameLibrary::load(&p))
                {
                    Ok(l) => l,
                    Err(e) => {
                        load_warnings.push(format!("Failed to load library: {}", e.user_message()));
                        GameLibrary::default()
                    }
                };

                let wine_configs = match opengamecore_lib::paths::wine_dir()
                    .and_then(|p| opengamecore_lib::wine::discover(&p))
                {
                    Ok(c) => c,
                    Err(e) => {
                        load_warnings
                            .push(format!("Failed to discover Wine: {}", e.user_message()));
                        Vec::new()
                    }
                };

                let bottles = match opengamecore_lib::paths::bottles_dir()
                    .and_then(|p| opengamecore_lib::bottle::list(&p))
                {
                    Ok(b) => b,
                    Err(e) => {
                        load_warnings.push(format!("Failed to list bottles: {}", e.user_message()));
                        Vec::new()
                    }
                };

                let compat_db = opengamecore_lib::paths::compat_db_path()
                    .and_then(|p| CompatDatabase::load(&p))
                    .ok();

                let bundles = match opengamecore_lib::paths::bundles_dir() {
                    Ok(p) => opengamecore_lib::bundle::load_bundles(&p).unwrap_or_default(),
                    Err(_) => HashMap::new(),
                };

                // Detect existing DXVK installation
                let dxvk_dir = opengamecore_lib::paths::wine_dir().ok().and_then(|wine| {
                    let dxvk_parent = wine.join("dxvk");
                    if dxvk_parent.exists() {
                        std::fs::read_dir(&dxvk_parent)
                            .ok()?
                            .flatten()
                            .find_map(|e| {
                                let p = e.path();
                                if p.is_dir() && p.join("x64").exists() {
                                    Some(p)
                                } else {
                                    None
                                }
                            })
                    } else {
                        None
                    }
                });

                Box::new(AppState {
                    config,
                    library,
                    wine_configs,
                    bottles,
                    load_warnings,
                    compat_db,
                    bundles,
                    dxvk_dir,
                })
            },
            Message::Loaded,
        );

        (app, task)
    }

    pub fn title(&self) -> String {
        String::from("OpenGameCore")
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowError(msg) => {
                self.error_message = Some(msg);
            }
            Message::DismissError => {
                self.error_message = None;
            }
            Message::NavigateTo(screen) => {
                self.screen = screen;
            }
            Message::Loaded(state) => {
                self.loading = false;
                let first_run = !state.config.app.first_run_complete;
                self.config = state.config;
                self.library = state.library;
                self.wine_configs = state.wine_configs;
                self.bottles = state.bottles;
                self.compat_db = state.compat_db;
                self.bundles = state.bundles;
                self.dxvk_dir = state.dxvk_dir;
                if let Some(warning) = state.load_warnings.first() {
                    self.error_message = Some(warning.clone());
                }
                if first_run {
                    self.screen = Screen::FirstRun;
                }

                // Auto-create template bottle if Wine is found but template is missing
                if !self.wine_configs.is_empty() {
                    if let Ok(template) = opengamecore_lib::paths::template_bottle_dir() {
                        if !template.join("system.reg").exists() {
                            if let Err(e) = opengamecore_lib::bottle::ensure_template(
                                &self.wine_configs[0].binary_path,
                                &template,
                            ) {
                                if self.error_message.is_none() {
                                    self.error_message =
                                        Some(format!("Failed to create template bottle: {}", e));
                                }
                            }
                        }
                    }
                }
            }

            // Add Game
            Message::OpenAddGame => {
                self.add_game = Some(AddGameState::default());
            }
            Message::CloseAddGame => {
                self.add_game = None;
            }
            Message::AddGameTabChanged(tab) => {
                if let Some(ref mut state) = self.add_game {
                    state.tab = tab;
                    state.path = None;
                }
            }
            Message::AddGameNameChanged(name) => {
                if let Some(ref mut state) = self.add_game {
                    state.name = name;
                }
            }
            Message::AddGameBrowse => {
                let is_auto_detect = self
                    .add_game
                    .as_ref()
                    .is_some_and(|s| s.tab == AddGameTab::AutoDetect);

                if is_auto_detect {
                    return Task::perform(
                        async {
                            let handle = rfd::AsyncFileDialog::new()
                                .set_title("Select game folder")
                                .pick_folder()
                                .await;
                            handle.map(|h| h.path().to_string_lossy().to_string())
                        },
                        Message::AddGamePathSelected,
                    );
                }

                return Task::perform(
                    async {
                        let handle = rfd::AsyncFileDialog::new()
                            .set_title("Select file")
                            .pick_file()
                            .await;
                        handle.map(|h| h.path().to_string_lossy().to_string())
                    },
                    Message::AddGamePathSelected,
                );
            }
            Message::AddGamePathSelected(path) => {
                if let Some(ref mut state) = self.add_game {
                    if let Some(ref p) = path {
                        if state.name.is_empty() {
                            if let Some(stem) =
                                std::path::Path::new(p).file_stem().and_then(|s| s.to_str())
                            {
                                state.name = stem.to_string();
                            }
                        }
                        // Auto-detect bundle matching when in AutoDetect mode
                        if state.tab == AddGameTab::AutoDetect {
                            let folder = std::path::PathBuf::from(p);
                            state.matched_bundle =
                                opengamecore_lib::bundle::match_bundle_for_folder(
                                    &folder,
                                    &self.bundles,
                                );
                            if let Some(ref bundle) = state.matched_bundle {
                                state.name = bundle.game.name.clone();
                            }
                        }
                    }
                    state.path = path;
                }
            }
            Message::AddGameBrowseIcon => {
                return Task::perform(
                    async {
                        let handle = rfd::AsyncFileDialog::new()
                            .set_title("Select game icon")
                            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp"])
                            .pick_file()
                            .await;
                        handle.map(|h| h.path().to_string_lossy().to_string())
                    },
                    Message::AddGameIconSelected,
                );
            }
            Message::AddGameIconSelected(path) => {
                if let Some(ref mut state) = self.add_game {
                    state.icon_path = path;
                }
            }
            Message::ConfirmAddGame => {
                // Validate before proceeding
                if let Some(ref state) = self.add_game {
                    if state.name.trim().is_empty() {
                        self.error_message = Some("Game name is required.".into());
                        return Task::none();
                    }
                    if state.path.is_none() {
                        self.error_message =
                            Some("Please select a game executable or folder.".into());
                        return Task::none();
                    }
                    let slug = opengamecore_lib::library::slugify(&state.name);
                    if self.library.find(&slug).is_some() {
                        self.error_message =
                            Some(format!("A game named '{}' already exists.", state.name));
                        return Task::none();
                    }
                }

                self.error_message = None;

                if let Some(state) = self.add_game.take() {
                    // Handle auto-detect with bundle
                    if state.tab == AddGameTab::AutoDetect {
                        if let Some(bundle) = state.matched_bundle {
                            if let Some(install_path) = state.path {
                                let install_path = std::path::PathBuf::from(&install_path);
                                match opengamecore_lib::bundle::apply_bundle(
                                    &bundle,
                                    &install_path,
                                    &mut self.library,
                                ) {
                                    Ok(applied_slug) => {
                                        if let Ok(path) = opengamecore_lib::paths::games_path() {
                                            if let Err(e) = self.library.save(&path) {
                                                self.error_message =
                                                    Some(format!("Failed to save library: {}", e));
                                            }
                                        }
                                        // Create bottle
                                        let slug = applied_slug;
                                        if let (Ok(template), Ok(bottle)) = (
                                            opengamecore_lib::paths::template_bottle_dir(),
                                            opengamecore_lib::paths::bottle_dir(&slug),
                                        ) {
                                            if let Err(e) =
                                                opengamecore_lib::bottle::create(&template, &bottle)
                                            {
                                                self.error_message =
                                                    Some(format!("Failed to create bottle: {}", e));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        self.error_message =
                                            Some(format!("Failed to apply bundle: {}", e));
                                    }
                                }
                            }
                        }
                        return Task::perform(
                            async {
                                opengamecore_lib::paths::bottles_dir()
                                    .ok()
                                    .and_then(|p| opengamecore_lib::bottle::list(&p).ok())
                                    .unwrap_or_default()
                            },
                            Message::BottlesLoaded,
                        );
                    }

                    let slug = opengamecore_lib::library::slugify(&state.name);
                    let install_type = match state.tab {
                        AddGameTab::Installer => InstallType::Installer,
                        AddGameTab::Portable => InstallType::Portable,
                        AddGameTab::FromFolder | AddGameTab::AutoDetect => {
                            InstallType::FolderInstall
                        }
                    };

                    let exe = state.path.unwrap_or_default();

                    let icon_path =
                        state.icon_path.and_then(
                            |ip| match opengamecore_lib::library::set_game_icon(
                                &slug,
                                std::path::Path::new(&ip),
                            ) {
                                Ok(p) => Some(p.to_string_lossy().to_string()),
                                Err(e) => {
                                    eprintln!("Warning: could not set game icon: {}", e);
                                    None
                                }
                            },
                        );

                    let game = Game {
                        name: state.name,
                        slug: slug.clone(),
                        exe,
                        install_type,
                        wine_config: "default".into(),
                        env: HashMap::new(),
                        added_at: chrono::Utc::now(),
                        last_played: None,
                        icon_path,
                        dxvk_enabled: false,
                        use_gptk: false,
                    };

                    if let Err(e) = self.library.add(game) {
                        self.error_message = Some(format!("Failed to add game: {}", e));
                        return Task::none();
                    }

                    // Save library
                    if let Ok(path) = opengamecore_lib::paths::games_path() {
                        if let Err(e) = self.library.save(&path) {
                            self.error_message = Some(format!("Failed to save library: {}", e));
                        }
                    }

                    // Create bottle from template
                    if let (Ok(template), Ok(bottle)) = (
                        opengamecore_lib::paths::template_bottle_dir(),
                        opengamecore_lib::paths::bottle_dir(&slug),
                    ) {
                        if let Err(e) = opengamecore_lib::bottle::create(&template, &bottle) {
                            self.error_message =
                                Some(format!("Failed to create game bottle: {}", e));
                        }
                    }

                    // Reload bottles
                    return Task::perform(
                        async {
                            opengamecore_lib::paths::bottles_dir()
                                .ok()
                                .and_then(|p| opengamecore_lib::bottle::list(&p).ok())
                                .unwrap_or_default()
                        },
                        Message::BottlesLoaded,
                    );
                }
            }

            // Game actions
            Message::PlayGame(slug) => {
                if let Some(game) = self.library.find(&slug) {
                    let wine =
                        opengamecore_lib::wine::resolve(&self.wine_configs, &game.wine_config);

                    match wine {
                        Ok(wine) => {
                            match opengamecore_lib::paths::bottle_dir(&slug) {
                                Ok(bottle_dir) => {
                                    let config = LaunchConfig::new(
                                        &wine,
                                        &bottle_dir,
                                        &game.exe,
                                        &game.env,
                                        game.dxvk_enabled,
                                    );

                                    let slug_clone = slug.clone();
                                    // Update last_played
                                    if let Some(game_mut) = self.library.find_mut(&slug) {
                                        game_mut.last_played = Some(chrono::Utc::now());
                                        if let Ok(path) = opengamecore_lib::paths::games_path() {
                                            if let Err(e) = self.library.save(&path) {
                                                self.error_message = Some(format!(
                                                    "Failed to save play time: {}",
                                                    e
                                                ));
                                            }
                                        }
                                    }

                                    self.running_games.insert(slug.clone());

                                    return Task::perform(
                                        async move {
                                            match opengamecore_lib::runner::run_and_capture(
                                                &config,
                                                &slug_clone,
                                            )
                                            .await
                                            {
                                                Ok(result) => Box::new(result),
                                                Err(e) => {
                                                    Box::new(opengamecore_lib::runner::RunResult {
                                                        slug: slug_clone,
                                                        exit_code: None,
                                                        stdout: String::new(),
                                                        stderr: e.to_string(),
                                                        duration_secs: 0.0,
                                                    })
                                                }
                                            }
                                        },
                                        Message::GameExited,
                                    );
                                }
                                Err(e) => {
                                    self.error_message =
                                        Some(format!("Failed to resolve bottle directory: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            self.error_message = Some(e.user_message());
                        }
                    }
                }
            }
            Message::GameExited(result) => {
                self.running_games.remove(&result.slug);
                // Save log
                if let Err(e) = opengamecore_lib::runner::save_run_log(&result) {
                    self.error_message = Some(format!("Failed to save game log: {}", e));
                }
                // Show error if game crashed
                if result.exit_code.is_some_and(|c| c != 0) {
                    self.error_message = Some(format!(
                        "'{}' exited with code {}. Check logs for details.",
                        result.slug,
                        result.exit_code.unwrap_or(-1)
                    ));
                }
            }

            // Bottle actions
            Message::ResetBottle(slug) => {
                if let (Ok(template), Ok(bottle)) = (
                    opengamecore_lib::paths::template_bottle_dir(),
                    opengamecore_lib::paths::bottle_dir(&slug),
                ) {
                    if let Err(e) = opengamecore_lib::bottle::reset(&template, &bottle) {
                        self.error_message = Some(format!("Failed to reset bottle: {}", e));
                    }
                }
                return Task::perform(
                    async {
                        opengamecore_lib::paths::bottles_dir()
                            .ok()
                            .and_then(|p| opengamecore_lib::bottle::list(&p).ok())
                            .unwrap_or_default()
                    },
                    Message::BottlesLoaded,
                );
            }
            Message::DeleteBottle(slug) => {
                if let Ok(bottle) = opengamecore_lib::paths::bottle_dir(&slug) {
                    if let Err(e) = opengamecore_lib::bottle::delete(&bottle) {
                        self.error_message = Some(format!("Failed to delete bottle: {}", e));
                    }
                }
                return Task::perform(
                    async {
                        opengamecore_lib::paths::bottles_dir()
                            .ok()
                            .and_then(|p| opengamecore_lib::bottle::list(&p).ok())
                            .unwrap_or_default()
                    },
                    Message::BottlesLoaded,
                );
            }
            Message::BottlesLoaded(bottles) => {
                self.bottles = bottles;
            }

            // Settings / Wine
            Message::SetDefaultWine(name) => {
                self.config.wine.default = name;
                if let Ok(path) = opengamecore_lib::paths::config_path() {
                    if let Err(e) = self.config.save(&path) {
                        self.error_message = Some(format!("Failed to save settings: {}", e));
                    }
                }
            }
            Message::AddCustomWinePath => {
                return Task::perform(
                    async {
                        let handle = rfd::AsyncFileDialog::new()
                            .set_title("Select Wine binary")
                            .pick_file()
                            .await;
                        handle.map(|h| h.path().to_string_lossy().to_string())
                    },
                    Message::CustomWinePathSelected,
                );
            }
            Message::CustomWinePathSelected(path) => {
                if let Some(path) = path {
                    let name = std::path::Path::new(&path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("custom")
                        .to_string();

                    self.wine_configs.push(WineConfig {
                        name,
                        binary_path: path.into(),
                        env_overrides: HashMap::new(),
                    });
                }
            }

            // DXVK
            Message::ToggleGameDxvk(slug) => {
                if let Some(game) = self.library.find_mut(&slug) {
                    game.dxvk_enabled = !game.dxvk_enabled;
                    if let Ok(path) = opengamecore_lib::paths::games_path() {
                        if let Err(e) = self.library.save(&path) {
                            self.error_message =
                                Some(format!("Failed to save DXVK setting: {}", e));
                        }
                    }
                }
            }
            Message::DownloadDxvk => {
                let url = self.config.wine.dxvk_download_url.clone();
                return Task::perform(
                    async move {
                        let data_dir =
                            opengamecore_lib::paths::wine_dir().map_err(|e| e.to_string())?;
                        opengamecore_lib::dxvk::download_and_extract(&url, &data_dir)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    |result: Result<PathBuf, String>| match result {
                        Ok(path) => Message::DxvkDownloaded(Some(path)),
                        Err(e) => Message::ShowError(format!("Failed to download DXVK: {}", e)),
                    },
                );
            }
            Message::DxvkDownloaded(path) => {
                self.dxvk_dir = path;
            }

            // Library import/export
            Message::ExportLibrary => {
                return Task::perform(
                    async {
                        let handle = rfd::AsyncFileDialog::new()
                            .set_file_name("opengamecore-library.toml")
                            .save_file()
                            .await;
                        handle.map(|h| h.path().to_string_lossy().to_string())
                    },
                    Message::ExportLibraryPath,
                );
            }
            Message::ExportLibraryPath(Some(path)) => {
                if let Err(e) = opengamecore_lib::library::export_library(
                    &self.library,
                    std::path::Path::new(&path),
                ) {
                    self.error_message = Some(format!("Failed to export library: {}", e));
                }
            }
            Message::ExportLibraryPath(None) => {}
            Message::ImportLibrary => {
                return Task::perform(
                    async {
                        let handle = rfd::AsyncFileDialog::new()
                            .set_title("Import Game Library")
                            .add_filter("TOML", &["toml"])
                            .pick_file()
                            .await;
                        handle.map(|h| h.path().to_string_lossy().to_string())
                    },
                    Message::ImportLibraryPath,
                );
            }
            Message::ImportLibraryPath(Some(path)) => {
                match opengamecore_lib::library::import_library(
                    &mut self.library,
                    std::path::Path::new(&path),
                ) {
                    Ok(count) => {
                        if let Ok(games_path) = opengamecore_lib::paths::games_path() {
                            if let Err(e) = self.library.save(&games_path) {
                                self.error_message =
                                    Some(format!("Failed to save library after import: {}", e));
                            }
                        }
                        return Task::done(Message::LibraryImported(count));
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to import library: {}", e));
                    }
                }
            }
            Message::ImportLibraryPath(None) => {}
            Message::LibraryImported(count) => {
                self.error_message = Some(format!("Successfully imported {} game(s).", count));
            }

            // First Run
            Message::StartFirstRun => {
                self.first_run_phase = FirstRunPhase::Downloading {
                    progress: 0.0,
                    status: "Starting download...".into(),
                };

                let url = self
                    .config
                    .wine
                    .download_urls
                    .first()
                    .cloned()
                    .unwrap_or_default();

                return Task::perform(
                    async move {
                        let wine_dir = match opengamecore_lib::paths::wine_dir() {
                            Ok(d) => d,
                            Err(e) => return Err(e.to_string()),
                        };

                        let extracted =
                            opengamecore_lib::wine::download_and_extract(&url, &wine_dir)
                                .await
                                .map_err(|e| e.to_string())?;

                        // Find the wine binary in extracted dir
                        let configs = opengamecore_lib::wine::discover(&wine_dir)
                            .map_err(|e| e.to_string())?;

                        if let Some(wine) = configs.first() {
                            let template = opengamecore_lib::paths::template_bottle_dir()
                                .map_err(|e| e.to_string())?;
                            opengamecore_lib::bottle::create_template(&wine.binary_path, &template)
                                .map_err(|e| e.to_string())?;
                        }

                        let _ = extracted;
                        Ok(())
                    },
                    |result: Result<(), String>| match result {
                        Ok(()) => Message::FirstRunComplete,
                        Err(e) => Message::FirstRunError(e),
                    },
                );
            }
            Message::SkipFirstRun => {
                self.config.app.first_run_complete = true;
                if let Ok(path) = opengamecore_lib::paths::config_path() {
                    if let Err(e) = self.config.save(&path) {
                        self.error_message = Some(format!("Failed to save configuration: {}", e));
                    }
                }
                self.screen = Screen::Library;
            }
            Message::FinishFirstRun => {
                self.config.app.first_run_complete = true;
                if let Ok(path) = opengamecore_lib::paths::config_path() {
                    if let Err(e) = self.config.save(&path) {
                        self.error_message = Some(format!("Failed to save configuration: {}", e));
                    }
                }
                self.screen = Screen::Library;

                // Reload wine configs
                match opengamecore_lib::paths::wine_dir()
                    .and_then(|d| opengamecore_lib::wine::discover(&d))
                {
                    Ok(configs) if !configs.is_empty() => {
                        self.config.wine.default = configs[0].name.clone();
                        self.wine_configs = configs.clone();

                        // Ensure template bottle exists after Wine is configured
                        if let Ok(template) = opengamecore_lib::paths::template_bottle_dir() {
                            if let Err(e) = opengamecore_lib::bottle::ensure_template(
                                &configs[0].binary_path,
                                &template,
                            ) {
                                self.error_message =
                                    Some(format!("Failed to create template bottle: {}", e));
                            }
                        }
                    }
                    Ok(_) => {
                        self.error_message = Some(
                            "Wine was downloaded but no binary was found. Check Settings.".into(),
                        );
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to detect Wine: {}", e));
                    }
                }
            }
            Message::FirstRunProgress(progress, status) => {
                self.first_run_phase = FirstRunPhase::Downloading { progress, status };
            }
            Message::FirstRunTemplateCreating => {
                self.first_run_phase = FirstRunPhase::CreatingTemplate;
            }
            Message::FirstRunComplete => {
                self.first_run_phase = FirstRunPhase::DetectingGames;
                return Task::done(Message::DetectGames);
            }
            Message::FirstRunError(err) => {
                self.first_run_phase = FirstRunPhase::Error(err);
            }

            // Database
            Message::SearchChanged(query) => {
                self.db_search_query = query;
            }
            Message::FilterRating(rating) => {
                self.db_filter_rating = rating;
            }
            Message::SetupFromDatabase(slug) => {
                let slug_clone = slug.clone();
                return Task::perform(
                    async move {
                        let handle = rfd::AsyncFileDialog::new()
                            .set_title("Select game folder")
                            .pick_folder()
                            .await;
                        let path = handle.map(|h| h.path().to_string_lossy().to_string());
                        (slug_clone, path)
                    },
                    |(slug, path)| Message::SetupFolderSelected(slug, path),
                );
            }
            Message::SetupFolderSelected(slug, Some(path)) => {
                let install_path = std::path::PathBuf::from(&path);
                if let Some(bundle) = self.bundles.get(&slug).cloned() {
                    match opengamecore_lib::bundle::apply_bundle(
                        &bundle,
                        &install_path,
                        &mut self.library,
                    ) {
                        Ok(applied_slug) => {
                            let slug = applied_slug;
                            if let Ok(games_path) = opengamecore_lib::paths::games_path() {
                                if let Err(e) = self.library.save(&games_path) {
                                    self.error_message =
                                        Some(format!("Failed to save library: {}", e));
                                }
                            }
                            if let (Ok(template), Ok(bottle)) = (
                                opengamecore_lib::paths::template_bottle_dir(),
                                opengamecore_lib::paths::bottle_dir(&slug),
                            ) {
                                if let Err(e) = opengamecore_lib::bottle::create(&template, &bottle)
                                {
                                    self.error_message =
                                        Some(format!("Failed to create bottle: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed to apply bundle: {}", e));
                        }
                    }
                } else {
                    self.error_message = Some(format!("No bundle found for '{}'", slug));
                }
                return Task::perform(
                    async {
                        opengamecore_lib::paths::bottles_dir()
                            .ok()
                            .and_then(|p| opengamecore_lib::bottle::list(&p).ok())
                            .unwrap_or_default()
                    },
                    Message::BottlesLoaded,
                );
            }
            Message::SetupFolderSelected(_, None) => {}

            // Store detection
            Message::DetectGames => {
                let compat_db = self.compat_db.clone();
                return Task::perform(
                    async move {
                        match compat_db {
                            Some(db) => opengamecore_lib::store_detect::detect_installed_games(&db)
                                .unwrap_or_default(),
                            None => Vec::new(),
                        }
                    },
                    Message::GamesDetected,
                );
            }
            Message::GamesDetected(games) => {
                self.detected_games = games.clone();
                if matches!(self.first_run_phase, FirstRunPhase::DetectingGames) {
                    self.first_run_phase = FirstRunPhase::GamesFound { detected: games };
                }
            }

            // Bundle
            Message::ApplyBundle(_slug) => {
                // Handled via SetupFromDatabase flow
            }
            Message::BundleApplied(_slug) => {}

            // Auto-detect
            Message::AutoDetectFolder => {
                let bundles = self.bundles.clone();
                return Task::perform(
                    async move {
                        let handle = rfd::AsyncFileDialog::new()
                            .set_title("Select game folder")
                            .pick_folder()
                            .await;
                        match handle {
                            Some(h) => {
                                let path = h.path().to_path_buf();
                                opengamecore_lib::bundle::match_bundle_for_folder(&path, &bundles)
                            }
                            None => None,
                        }
                    },
                    |b| Message::AutoDetectResult(Box::new(b)),
                );
            }
            Message::AutoDetectResult(bundle) => {
                if let Some(ref mut state) = self.add_game {
                    state.matched_bundle = *bundle;
                }
            }

            // Steam
            Message::InstallSteam => {
                if let Some(wine) = self.wine_configs.first().cloned() {
                    let binary_path = wine.binary_path.clone();
                    return Task::perform(
                        async move {
                            let bottles_dir = opengamecore_lib::paths::bottles_dir()
                                .map_err(|e| e.to_string())?;
                            let steam_bottle = bottles_dir.join("steam");
                            opengamecore_lib::wine::install_steam(&binary_path, &steam_bottle)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        Message::SteamInstalled,
                    );
                } else {
                    self.error_message =
                        Some("No Wine installation found. Install Wine first.".into());
                }
            }
            Message::SteamInstalled(result) => match result {
                Ok(()) => {
                    self.error_message = Some("Steam installed successfully!".into());
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to install Steam: {}", e));
                }
            },

            // Data update
            Message::DatabaseUpdated(_) => {}
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.loading {
            return container(text("Loading..."))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        }

        // First run is a full-screen view without sidebar
        if matches!(self.screen, Screen::FirstRun) {
            return views::first_run::view(&self.first_run_phase);
        }

        let sidebar = views::sidebar::view(&self.screen);

        let screen_content: Element<'_, Message> = match self.screen {
            Screen::Library => views::game_grid::view(&self.library.games, &self.running_games),
            Screen::Database => views::game_database::view(
                self.compat_db.as_ref(),
                &self.db_search_query,
                &self.db_filter_rating,
            ),
            Screen::Bottles => views::bottle_detail::view(&self.bottles),
            Screen::Settings => views::settings::view(
                &self.wine_configs,
                &self.config.wine.download_urls,
                &self.config.wine.default,
                self.dxvk_dir.as_deref(),
            ),
            Screen::FirstRun => unreachable!(),
        };

        // Wrap main content with optional error banner
        let main_content: Element<'_, Message> = if let Some(ref msg) = self.error_message {
            let error_text = text(msg.clone()).size(14).color(iced::Color::WHITE);
            let dismiss_btn = button(text("X").size(14).color(iced::Color::WHITE))
                .on_press(Message::DismissError)
                .padding([2, 8])
                .style(|_theme, _status| button::Style {
                    background: None,
                    text_color: iced::Color::WHITE,
                    border: Border::default(),
                    ..button::Style::default()
                });

            let error_banner = container(
                row![error_text, dismiss_btn]
                    .spacing(12)
                    .align_y(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .padding([8, 16])
            .style(|_theme| container::Style {
                background: Some(Background::Color(iced::Color::from_rgb(0.8, 0.2, 0.15))),
                ..container::Style::default()
            });

            column![error_banner, screen_content]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            screen_content
        };

        let base = container(row![sidebar, main_content])
            .width(Length::Fill)
            .height(Length::Fill);

        if let Some(ref add_game_state) = self.add_game {
            let overlay = views::add_game::view(add_game_state);
            iced::widget::stack![base, overlay]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            base.into()
        }
    }
}
