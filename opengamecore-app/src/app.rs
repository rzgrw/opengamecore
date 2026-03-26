use std::collections::HashMap;
use std::path::PathBuf;

use iced::widget::{container, row, text};
use iced::{Element, Length, Task, Theme};

use opengamecore_lib::bottle::BottleInfo;
use opengamecore_lib::{
    AppConfig, Game, GameLibrary, InstallType, LaunchConfig, WineConfig,
};
use crate::views;
use crate::views::add_game::{AddGameState, AddGameTab};
use crate::views::first_run::FirstRunPhase;

#[derive(Debug, Clone)]
pub enum Screen {
    FirstRun,
    Library,
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
    GameExited(String),

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

    // First Run
    StartFirstRun,
    SkipFirstRun,
    FinishFirstRun,
    FirstRunProgress(f32, String),
    FirstRunTemplateCreating,
    FirstRunComplete,
    FirstRunError(String),
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub library: GameLibrary,
    pub wine_configs: Vec<WineConfig>,
    pub bottles: Vec<BottleInfo>,
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
        };

        let task = Task::perform(
            async {
                let config = opengamecore_lib::paths::config_path()
                    .ok()
                    .and_then(|p| AppConfig::load(&p).ok())
                    .unwrap_or_default();

                let library = opengamecore_lib::paths::games_path()
                    .ok()
                    .and_then(|p| GameLibrary::load(&p).ok())
                    .unwrap_or_default();

                let wine_configs = opengamecore_lib::paths::wine_dir()
                    .ok()
                    .and_then(|p| opengamecore_lib::wine::discover(&p).ok())
                    .unwrap_or_default();

                let bottles = opengamecore_lib::paths::bottles_dir()
                    .ok()
                    .and_then(|p| opengamecore_lib::bottle::list(&p).ok())
                    .unwrap_or_default();

                Box::new(AppState {
                    config,
                    library,
                    wine_configs,
                    bottles,
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
                if first_run {
                    self.screen = Screen::FirstRun;
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
                            // Auto-fill name from filename
                            if let Some(stem) = std::path::Path::new(p)
                                .file_stem()
                                .and_then(|s| s.to_str())
                            {
                                state.name = stem.to_string();
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
                if let Some(state) = self.add_game.take() {
                    let slug = opengamecore_lib::library::slugify(&state.name);
                    let install_type = match state.tab {
                        AddGameTab::Installer => InstallType::Installer,
                        AddGameTab::Portable => InstallType::Portable,
                        AddGameTab::FromFolder => InstallType::FolderInstall,
                    };

                    let exe = state.path.unwrap_or_default();

                    let icon_path = state.icon_path.and_then(|ip| {
                        opengamecore_lib::library::set_game_icon(
                            &slug,
                            std::path::Path::new(&ip),
                        )
                        .ok()
                        .map(|p| p.to_string_lossy().to_string())
                    });

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
                    };

                    self.library.add(game);

                    // Save library
                    if let Ok(path) = opengamecore_lib::paths::games_path() {
                        let _ = self.library.save(&path);
                    }

                    // Create bottle from template
                    if let (Ok(template), Ok(bottle)) = (
                        opengamecore_lib::paths::template_bottle_dir(),
                        opengamecore_lib::paths::bottle_dir(&slug),
                    ) {
                        let _ = opengamecore_lib::bottle::create(&template, &bottle);
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
                    let wine = opengamecore_lib::wine::resolve(
                        &self.wine_configs,
                        &game.wine_config,
                    );

                    if let Ok(wine) = wine {
                        if let Ok(bottle_dir) = opengamecore_lib::paths::bottle_dir(&slug) {
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
                                    let _ = self.library.save(&path);
                                }
                            }

                            return Task::perform(
                                async move {
                                    match opengamecore_lib::runner::spawn(&config) {
                                        Ok(mut child) => {
                                            let _ = child.wait().await;
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to launch: {}", e);
                                        }
                                    }
                                    slug_clone
                                },
                                Message::GameExited,
                            );
                        }
                    }
                }
            }
            Message::GameExited(_slug) => {
                // Game process finished, could refresh state
            }

            // Bottle actions
            Message::ResetBottle(slug) => {
                if let (Ok(template), Ok(bottle)) = (
                    opengamecore_lib::paths::template_bottle_dir(),
                    opengamecore_lib::paths::bottle_dir(&slug),
                ) {
                    let _ = opengamecore_lib::bottle::reset(&template, &bottle);
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
                    let _ = opengamecore_lib::bottle::delete(&bottle);
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
                    let _ = self.config.save(&path);
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
                        let _ = self.library.save(&path);
                    }
                }
            }
            Message::DownloadDxvk => {
                let url = self.config.wine.dxvk_download_url.clone();
                return Task::perform(
                    async move {
                        let data_dir = match opengamecore_lib::paths::wine_dir() {
                            Ok(d) => d,
                            Err(_) => return None,
                        };
                        opengamecore_lib::dxvk::download_and_extract(&url, &data_dir)
                            .await
                            .ok()
                    },
                    Message::DxvkDownloaded,
                );
            }
            Message::DxvkDownloaded(path) => {
                self.dxvk_dir = path;
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

                        let extracted = opengamecore_lib::wine::download_and_extract(&url, &wine_dir)
                            .await
                            .map_err(|e| e.to_string())?;

                        // Find the wine binary in extracted dir
                        let configs = opengamecore_lib::wine::discover(&wine_dir)
                            .map_err(|e| e.to_string())?;

                        if let Some(wine) = configs.first() {
                            let template = opengamecore_lib::paths::template_bottle_dir()
                                .map_err(|e| e.to_string())?;
                            opengamecore_lib::bottle::create_template(
                                &wine.binary_path,
                                &template,
                            )
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
                    let _ = self.config.save(&path);
                }
                self.screen = Screen::Library;
            }
            Message::FinishFirstRun => {
                self.config.app.first_run_complete = true;
                if let Ok(path) = opengamecore_lib::paths::config_path() {
                    let _ = self.config.save(&path);
                }
                self.screen = Screen::Library;

                // Reload wine configs
                if let Ok(wine_dir) = opengamecore_lib::paths::wine_dir() {
                    if let Ok(configs) = opengamecore_lib::wine::discover(&wine_dir) {
                        self.wine_configs = configs;
                        if let Some(first) = self.wine_configs.first() {
                            self.config.wine.default = first.name.clone();
                        }
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
                self.first_run_phase = FirstRunPhase::Done;
            }
            Message::FirstRunError(err) => {
                self.first_run_phase = FirstRunPhase::Error(err);
            }
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

        let main_content: Element<'_, Message> = match self.screen {
            Screen::Library => views::game_grid::view(&self.library.games),
            Screen::Bottles => views::bottle_detail::view(&self.bottles),
            Screen::Settings => views::settings::view(
                &self.wine_configs,
                &self.config.wine.download_urls,
                &self.config.wine.default,
                self.dxvk_dir.as_deref(),
            ),
            Screen::FirstRun => unreachable!(),
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
