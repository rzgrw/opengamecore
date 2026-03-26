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
                let config = opengamecore_lib::paths::config_path()
                    .ok()
                    .and_then(|p| AppConfig::load(&p).ok())
                    .unwrap_or_default();

                let library = opengamecore_lib::paths::games_path()
                    .ok()
                    .and_then(|p| GameLibrary::load(&p).ok())
                    .unwrap_or_default();

                Box::new(AppState { config, library })
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
                if first_run {
                    self.screen = Screen::FirstRun;
                }
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

        let game_count = self.library.games.len();

        let sidebar = container(text("Sidebar"))
            .width(200)
            .height(Length::Fill);

        let content_text = match self.screen {
            Screen::FirstRun => String::from("Welcome to OpenGameCore"),
            Screen::Library => format!("{} games", game_count),
            Screen::Bottles => String::from("Bottles"),
            Screen::Settings => String::from("Settings"),
        };

        let main_content = container(
            column![text(content_text).size(24)]
                .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        container(row![sidebar, main_content])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
