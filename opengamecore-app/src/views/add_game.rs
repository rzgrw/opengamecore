use iced::widget::{button, column, container, row, text, text_input};
use iced::{Background, Border, Element, Length};

use crate::app::Message;
use crate::theme;

#[derive(Debug, Clone, PartialEq)]
pub enum AddGameTab {
    Installer,
    Portable,
    FromFolder,
}

#[derive(Debug, Clone)]
pub struct AddGameState {
    pub tab: AddGameTab,
    pub name: String,
    pub path: Option<String>,
}

impl Default for AddGameState {
    fn default() -> Self {
        Self {
            tab: AddGameTab::Installer,
            name: String::new(),
            path: None,
        }
    }
}

pub fn view(state: &AddGameState) -> Element<'_, Message> {
    let tab_button =
        |label: &str, tab: AddGameTab, is_active: bool| -> Element<'static, Message> {
            let label = label.to_string();
            button(text(label).size(14).color(if is_active {
                theme::ACCENT
            } else {
                theme::TEXT_SECONDARY
            }))
            .on_press(Message::AddGameTabChanged(tab))
            .padding([8, 16])
            .style(move |_theme, _status| button::Style {
                background: if is_active {
                    Some(Background::Color(iced::Color::from_rgba(
                        1.0, 1.0, 1.0, 0.1,
                    )))
                } else {
                    None
                },
                text_color: theme::TEXT_PRIMARY,
                border: Border::default().rounded(4),
                ..button::Style::default()
            })
            .into()
        };

    let tabs = row![
        tab_button(
            "Installer",
            AddGameTab::Installer,
            state.tab == AddGameTab::Installer
        ),
        tab_button(
            "Portable",
            AddGameTab::Portable,
            state.tab == AddGameTab::Portable
        ),
        tab_button(
            "From Folder",
            AddGameTab::FromFolder,
            state.tab == AddGameTab::FromFolder
        ),
    ]
    .spacing(4);

    let tab_hint = match state.tab {
        AddGameTab::Installer => "Select a Windows installer (.exe/.msi)",
        AddGameTab::Portable => "Select a portable game executable",
        AddGameTab::FromFolder => "Select an existing game folder",
    };

    let path_display = state
        .path
        .as_deref()
        .unwrap_or("No file selected");

    let browse_row = row![
        container(text(path_display).size(13).color(theme::TEXT_SECONDARY))
            .padding([8, 12])
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(iced::Color::from_rgba(
                    0.0, 0.0, 0.0, 0.3
                ))),
                border: Border::default().rounded(4),
                ..container::Style::default()
            }),
        button(text("Browse").size(14).color(theme::TEXT_PRIMARY))
            .on_press(Message::AddGameBrowse)
            .padding([8, 16])
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(iced::Color::from_rgba(
                    1.0, 1.0, 1.0, 0.1
                ))),
                text_color: theme::TEXT_PRIMARY,
                border: Border::default().rounded(4),
                ..button::Style::default()
            })
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let name_input = text_input("Game name", &state.name)
        .on_input(Message::AddGameNameChanged)
        .padding(8)
        .size(14);

    let can_add = !state.name.is_empty() && state.path.is_some();

    let mut add_btn = button(
        text("Add Game").size(14).color(theme::BUTTON_GREEN_TEXT),
    )
    .padding([8, 20])
    .style(|_theme, _status| button::Style {
        background: Some(Background::Color(theme::BUTTON_GREEN)),
        text_color: theme::BUTTON_GREEN_TEXT,
        border: Border::default().rounded(6),
        ..button::Style::default()
    });

    if can_add {
        add_btn = add_btn.on_press(Message::ConfirmAddGame);
    }

    let cancel_btn = button(
        text("Cancel").size(14).color(theme::TEXT_SECONDARY),
    )
    .on_press(Message::CloseAddGame)
    .padding([8, 20])
    .style(|_theme, _status| button::Style {
        background: None,
        text_color: theme::TEXT_SECONDARY,
        border: Border::default(),
        ..button::Style::default()
    });

    let action_row = row![cancel_btn, add_btn].spacing(12);

    let dialog = container(
        column![
            text("Add Game").size(20).color(theme::TEXT_PRIMARY),
            tabs,
            text(tab_hint).size(13).color(theme::TEXT_SECONDARY),
            browse_row,
            text("Game Name").size(13).color(theme::TEXT_SECONDARY),
            name_input,
            action_row,
        ]
        .spacing(12)
        .padding(24)
        .width(400),
    )
    .style(|_theme| container::Style {
        background: Some(Background::Color(theme::BG_SIDEBAR)),
        border: Border::default().rounded(12),
        ..container::Style::default()
    });

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(iced::Color::from_rgba(
                0.0, 0.0, 0.0, 0.5,
            ))),
            ..container::Style::default()
        })
        .into()
}
