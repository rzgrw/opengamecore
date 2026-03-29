use std::collections::HashSet;

use iced::widget::{button, column, container, image, row, text, Scrollable};
use iced::{Background, Border, Element, Length};

use opengamecore_lib::store_detect::DetectedGame;
use opengamecore_lib::Game;

use crate::app::Message;
use crate::theme;

pub fn view<'a>(
    games: &'a [Game],
    running_games: &'a HashSet<String>,
    detected_games: &'a [DetectedGame],
) -> Element<'a, Message> {
    let header = row![
        text("All Games").size(24).color(theme::TEXT_PRIMARY),
        iced::widget::horizontal_space(),
        button(text("+ Add Game").size(14).color(theme::BUTTON_GREEN_TEXT))
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

    let has_games = !games.is_empty();

    // Filter detected games to only show those NOT already in library
    let library_slugs: HashSet<&str> = games.iter().map(|g| g.slug.as_str()).collect();
    let new_detected: Vec<&DetectedGame> = detected_games
        .iter()
        .filter(|d| {
            let slug = opengamecore_lib::library::slugify(&d.name);
            !library_slugs.contains(slug.as_str())
        })
        .collect();

    let has_detected = !new_detected.is_empty();

    if !has_games && !has_detected {
        let empty = container(
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
        .center_y(Length::Fill);

        return column![header, empty]
            .spacing(16)
            .padding(24)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    let mut all_cards = column![].spacing(12);

    // Library games
    for game in games {
        all_cards = all_cards.push(game_card(game, running_games));
    }

    // Detected games section
    if has_detected {
        if has_games {
            all_cards = all_cards.push(
                text("Detected from Steam / GOG")
                    .size(16)
                    .color(theme::TEXT_SECONDARY),
            );
        }

        for detected in &new_detected {
            all_cards = all_cards.push(detected_card(detected));
        }
    }

    let content: Element<'_, Message> = Scrollable::new(all_cards).into();

    column![header, content]
        .spacing(16)
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn game_card<'a>(game: &'a Game, running_games: &'a HashSet<String>) -> Element<'a, Message> {
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
        _ => container(text("G").size(20).color(theme::ACCENT))
            .width(64)
            .height(64)
            .center_x(64)
            .center_y(64)
            .style(|_theme| container::Style {
                background: Some(Background::Color(iced::Color::from_rgba(
                    1.0, 1.0, 1.0, 0.05,
                ))),
                border: Border::default().rounded(8),
                ..container::Style::default()
            })
            .into(),
    };

    let is_gptk = game.wine_config.contains("gptk") || game.use_gptk;

    let mut badge_row = row![text(format!("Wine: {}", &game.wine_config))
        .size(12)
        .color(theme::TEXT_SECONDARY),]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    if game.dxvk_enabled {
        badge_row = badge_row.push(
            container(text("DXVK").size(10).color(theme::ACCENT))
                .padding([2, 6])
                .style(|_theme| container::Style {
                    background: Some(Background::Color(iced::Color::from_rgba(
                        0.39, 1.0, 0.855, 0.12,
                    ))),
                    border: Border::default().rounded(4),
                    ..container::Style::default()
                }),
        );
    }

    if is_gptk {
        badge_row = badge_row.push(
            container(text("GPTK").size(10).color(theme::BADGE_GPTK))
                .padding([2, 6])
                .style(|_theme| container::Style {
                    background: Some(Background::Color(iced::Color::from_rgba(
                        1.0, 0.76, 0.03, 0.12,
                    ))),
                    border: Border::default().rounded(4),
                    ..container::Style::default()
                }),
        );
    }

    let wine_row: Element<'_, Message> = badge_row.into();

    let is_running = running_games.contains(&slug);
    let slug_for_remove = slug.clone();
    let play_widget: Element<'_, Message> = if is_running {
        container(text("Running...").size(14).color(theme::TEXT_SECONDARY))
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

    let remove_btn = button(text("X").size(12).color(theme::TEXT_SECONDARY))
        .on_press(Message::RemoveGame(slug_for_remove))
        .padding([6, 10])
        .style(|_theme, _status| button::Style {
            background: Some(Background::Color(iced::Color::from_rgba(
                1.0, 0.2, 0.2, 0.15,
            ))),
            text_color: theme::TEXT_SECONDARY,
            border: Border::default().rounded(4),
            ..button::Style::default()
        });

    container(
        row![
            icon_widget,
            column![
                text(&game.name).size(16).color(theme::TEXT_PRIMARY),
                wine_row,
            ]
            .spacing(4),
            iced::widget::horizontal_space(),
            play_widget,
            remove_btn,
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
    })
    .into()
}

fn detected_card(detected: &DetectedGame) -> Element<'_, Message> {
    let store_label = match detected.store {
        opengamecore_lib::store_detect::GameStore::Steam => "Steam",
        opengamecore_lib::store_detect::GameStore::Gog => "GOG",
    };

    let rating_text = detected
        .rating
        .as_ref()
        .map(|r| r.label().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let rating_color = detected
        .rating
        .as_ref()
        .map_or(theme::TEXT_SECONDARY, |r| match r {
            opengamecore_lib::CompatRating::Platinum => iced::Color::from_rgb(0.0, 0.85, 0.45),
            opengamecore_lib::CompatRating::Gold => iced::Color::from_rgb(1.0, 0.84, 0.0),
            opengamecore_lib::CompatRating::Silver => iced::Color::from_rgb(0.75, 0.75, 0.75),
            opengamecore_lib::CompatRating::Bronze => iced::Color::from_rgb(0.8, 0.5, 0.2),
            opengamecore_lib::CompatRating::Borked => iced::Color::from_rgb(0.9, 0.2, 0.2),
        });

    let icon = container(
        text(store_label.chars().next().unwrap_or('?'))
            .size(20)
            .color(theme::ACCENT),
    )
    .width(64)
    .height(64)
    .center_x(64)
    .center_y(64)
    .style(|_theme| container::Style {
        background: Some(Background::Color(iced::Color::from_rgba(
            1.0, 1.0, 1.0, 0.03,
        ))),
        border: Border::default().rounded(8),
        ..container::Style::default()
    });

    let info = column![
        text(&detected.name).size(16).color(theme::TEXT_PRIMARY),
        row![
            text(store_label).size(12).color(theme::TEXT_SECONDARY),
            container(text(rating_text.clone()).size(10).color(rating_color))
                .padding([2, 6])
                .style(move |_theme| container::Style {
                    background: Some(Background::Color(iced::Color::from_rgba(
                        1.0, 1.0, 1.0, 0.05,
                    ))),
                    border: Border::default().rounded(4),
                    ..container::Style::default()
                }),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(4);

    let slug = opengamecore_lib::library::slugify(&detected.name);
    let action: Element<'_, Message> = if detected.bundle_available {
        button(text("Set Up").size(14).color(theme::BUTTON_GREEN_TEXT))
            .on_press(Message::SetupFromDatabase(slug))
            .padding([8, 16])
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(theme::BUTTON_GREEN)),
                text_color: theme::BUTTON_GREEN_TEXT,
                border: Border::default().rounded(6),
                ..button::Style::default()
            })
            .into()
    } else {
        button(text("Add Manually").size(14).color(theme::TEXT_SECONDARY))
            .on_press(Message::OpenAddGame)
            .padding([8, 16])
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(iced::Color::from_rgba(
                    1.0, 1.0, 1.0, 0.05,
                ))),
                text_color: theme::TEXT_SECONDARY,
                border: Border::default().rounded(6),
                ..button::Style::default()
            })
            .into()
    };

    container(
        row![icon, info, iced::widget::horizontal_space(), action,]
            .spacing(12)
            .align_y(iced::Alignment::Center),
    )
    .padding(12)
    .width(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(Background::Color(iced::Color::from_rgba(
            1.0, 1.0, 1.0, 0.02,
        ))),
        border: Border {
            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}
