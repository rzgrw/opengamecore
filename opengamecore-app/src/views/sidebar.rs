use iced::widget::{button, column, container, text};
use iced::{Background, Border, Element, Length};

use crate::app::{Message, Screen};
use crate::theme;

pub fn view(current: &Screen) -> Element<'static, Message> {
    let header = text("OpenGameCore").size(20).color(theme::ACCENT);

    let nav_button = |label: &str, screen: Screen, is_active: bool| -> Element<'static, Message> {
        let label = label.to_string();
        let btn = button(text(label).size(14).color(if is_active {
            theme::ACCENT
        } else {
            theme::TEXT_PRIMARY
        }))
        .on_press(Message::NavigateTo(screen))
        .width(Length::Fill)
        .padding([8, 16])
        .style(move |_theme, _status| button::Style {
            background: if is_active {
                Some(Background::Color(iced::Color::from_rgba(
                    1.0, 1.0, 1.0, 0.05,
                )))
            } else {
                None
            },
            text_color: theme::TEXT_PRIMARY,
            border: Border::default().rounded(4),
            ..button::Style::default()
        });

        btn.into()
    };

    let is_library = matches!(current, Screen::Library);
    let is_database = matches!(current, Screen::Database);
    let is_bottles = matches!(current, Screen::Bottles);
    let is_settings = matches!(current, Screen::Settings);

    let content = column![
        header,
        nav_button("All Games", Screen::Library, is_library),
        nav_button("Game Database", Screen::Database, is_database),
        nav_button("Bottles", Screen::Bottles, is_bottles),
        nav_button("Settings", Screen::Settings, is_settings),
    ]
    .spacing(8)
    .padding(16);

    container(content)
        .width(200)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme::BG_SIDEBAR)),
            ..container::Style::default()
        })
        .into()
}
