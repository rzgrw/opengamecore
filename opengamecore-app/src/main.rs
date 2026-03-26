mod app;
mod theme;
mod views;

use app::App;

fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .window(iced::window::Settings {
            size: iced::Size::new(1024.0, 700.0),
            min_size: Some(iced::Size::new(640.0, 480.0)),
            ..Default::default()
        })
        .run_with(App::new)
}
