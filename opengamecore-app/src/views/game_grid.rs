use std::collections::HashSet;

use iced::widget::{button, column, container, image, row, text, Scrollable};
use iced::{Background, Border, Element, Length};

use opengamecore_lib::Game;

use crate::app::Message;
use crate::theme;

pub fn view<'a>(games: &'a [Game], running_games: &'a HashSet<String>) -> Element<'a, Message> {
    let header = row![
        text("All Games").size(24).color(theme::TEXT_PRIMARY),
        iced::widget::horizontal_space(),
        button(
            text("+ Add Game").size(14).color(theme::BUTTON_GREEN_TEXT)
        )
        .on_press(Message::OpenAddGame)
        .padding([8, 16])
        .style(|_theme, _status| button::Style {
            background: Some(Background::Color(theme::BUTTON_GREEN)),
            text_color: theme::BUTTON_GREEN_TEXT,
            border: Border::default().rounded(6),
            ..button::Style::default()
        })
    ]
    .align_y(iced::Alignment::Center)
    .spacing(12);

    let content: Element<'_, Message> = if games.is_empty() {
        container(
            column![
                text("No games yet").size(18).color(theme::TEXT_SECONDARY),
                text("Click \"+ Add Game\" to get started")
                    .size(14)
                    .color(theme::TEXT_SECONDARY),
            ]
            .spacing(8)
            .align_x(iced::Alignment::Center),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        let mut cards = column![].spacing(12);
        for game in games {
            let slug = game.slug.clone();
            let icon_widget: Element<'_, Message> = match &game.icon_path {
                Some(path) if std::path::Path::new(path).exists() => {
                    container(image(path).width(64).height(64))
                        .width(64)
                        .height(64)
                        .style(|_theme| container::Style {
                            border: Border::default().rounded(8),
                            ..container::Style::default()
                        })
                        .into()
                }
                _ => {
                    container(text("G").size(20).color(theme::ACCENT))
                        .width(64)
                        .height(64)
                        .center_x(64)
                        .center_y(64)
                        .style(|_theme| container::Style {
                            background: Some(Background::Color(
                                iced::Color::from_rgba(1.0, 1.0, 1.0, 0.05),
                            )),
                            border: Border::default().rounded(8),
                            ..container::Style::default()
                        })
                        .into()
                }
            };

            let wine_row: Element<'_, Message> = if game.dxvk_enabled {
                row![
                    text(format!("Wine: {}", &game.wine_config))
                        .size(12)
                        .color(theme::TEXT_SECONDARY),
                    container(
                        text("DXVK").size(10).color(theme::ACCENT),
                    )
                    .padding([2, 6])
                    .style(|_theme| container::Style {
                        background: Some(Background::Color(iced::Color::from_rgba(
                            0.39, 1.0, 0.855, 0.12,
                        ))),
                        border: Border::default().rounded(4),
                        ..container::Style::default()
                    }),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center)
                .into()
            } else {
                text(format!("Wine: {}", &game.wine_config))
                    .size(12)
                    .color(theme::TEXT_SECONDARY)
                    .into()
            };

            let is_running = running_games.contains(&slug);
            let play_widget: Element<'_, Message> = if is_running {
                container(
                    text("Running...").size(14).color(theme::TEXT_SECONDARY),
                )
                .padding([8, 20])
                .into()
            } else {
                button(text("Play").size(14).color(theme::BUTTON_GREEN_TEXT))
                    .on_press(Message::PlayGame(slug))
                    .padding([8, 20])
                    .style(|_theme, _status| button::Style {
                        background: Some(Background::Color(theme::BUTTON_GREEN)),
                        text_color: theme::BUTTON_GREEN_TEXT,
                        border: Border::default().rounded(6),
                        ..button::Style::default()
                    })
                    .into()
            };

            let card = container(
                row![
                    icon_widget,
                    column![
                        text(&game.name).size(16).color(theme::TEXT_PRIMARY),
                        wine_row,
                    ]
                    .spacing(4),
                    iced::widget::horizontal_space(),
                    play_widget,
                ]
                .spacing(12)
                .align_y(iced::Alignment::Center),
            )
            .padding(12)
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(theme::BG_CARD)),
                border: Border::default().rounded(8),
                ..container::Style::default()
            });

            cards = cards.push(card);
        }
        Scrollable::new(cards).into()
    };

    column![header, content]
        .spacing(16)
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
