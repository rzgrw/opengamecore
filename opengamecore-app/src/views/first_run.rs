use iced::widget::{button, column, container, progress_bar, scrollable, row, text};
use iced::{Background, Border, Element, Length};

use opengamecore_lib::store_detect::DetectedGame;

use crate::app::Message;
use crate::theme;

#[derive(Debug, Clone, Default)]
pub enum FirstRunPhase {
    #[default]
    Welcome,
    Downloading { progress: f32, status: String },
    CreatingTemplate,
    Done,
    DetectingGames,
    GamesFound { detected: Vec<DetectedGame> },
    Error(String),
}


pub fn view(phase: &FirstRunPhase) -> Element<'_, Message> {
    let content: Element<'_, Message> = match phase {
        FirstRunPhase::Welcome => {
            let title = text("Welcome to OpenGameCore")
                .size(28)
                .color(theme::ACCENT);

            let subtitle = text("To get started, we need to download a Wine build.")
                .size(14)
                .color(theme::TEXT_SECONDARY);

            let download_btn = button(
                text("Download Wine").size(16).color(theme::BUTTON_GREEN_TEXT),
            )
            .on_press(Message::StartFirstRun)
            .padding([12, 32])
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(theme::BUTTON_GREEN)),
                text_color: theme::BUTTON_GREEN_TEXT,
                border: Border::default().rounded(8),
                ..button::Style::default()
            });

            let skip = button(
                text("Skip for now").size(13).color(theme::TEXT_SECONDARY),
            )
            .on_press(Message::SkipFirstRun)
            .style(|_theme, _status| button::Style {
                background: None,
                text_color: theme::TEXT_SECONDARY,
                border: Border::default(),
                ..button::Style::default()
            });

            column![title, subtitle, download_btn, skip]
                .spacing(16)
                .align_x(iced::Alignment::Center)
                .into()
        }
        FirstRunPhase::Downloading { progress, status } => {
            let title = text("Downloading Wine...")
                .size(24)
                .color(theme::TEXT_PRIMARY);

            let bar = progress_bar(0.0..=100.0, *progress)
                .width(400)
                .height(8);

            let status_text = text(status)
                .size(13)
                .color(theme::TEXT_SECONDARY);

            column![title, bar, status_text]
                .spacing(12)
                .align_x(iced::Alignment::Center)
                .into()
        }
        FirstRunPhase::CreatingTemplate => {
            let title = text("Creating template bottle...")
                .size(24)
                .color(theme::TEXT_PRIMARY);

            let subtitle = text("This may take a moment.")
                .size(14)
                .color(theme::TEXT_SECONDARY);

            column![title, subtitle]
                .spacing(12)
                .align_x(iced::Alignment::Center)
                .into()
        }
        FirstRunPhase::Done => {
            let title = text("All set!")
                .size(28)
                .color(theme::ACCENT);

            let subtitle = text("Wine is installed and ready to go.")
                .size(14)
                .color(theme::TEXT_SECONDARY);

            let go_btn = button(
                text("Go to Library").size(16).color(theme::BUTTON_GREEN_TEXT),
            )
            .on_press(Message::FinishFirstRun)
            .padding([12, 32])
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(theme::BUTTON_GREEN)),
                text_color: theme::BUTTON_GREEN_TEXT,
                border: Border::default().rounded(8),
                ..button::Style::default()
            });

            column![title, subtitle, go_btn]
                .spacing(16)
                .align_x(iced::Alignment::Center)
                .into()
        }
        FirstRunPhase::DetectingGames => {
            let title = text("Scanning for installed games...")
                .size(24)
                .color(theme::TEXT_PRIMARY);

            let subtitle = text("Checking Steam and GOG libraries")
                .size(14)
                .color(theme::TEXT_SECONDARY);

            column![title, subtitle]
                .spacing(12)
                .align_x(iced::Alignment::Center)
                .into()
        }
        FirstRunPhase::GamesFound { detected } => {
            let title = text("Games Found!")
                .size(28)
                .color(theme::ACCENT);

            let subtitle = text(format!("Found {} installed game(s)", detected.len()))
                .size(14)
                .color(theme::TEXT_SECONDARY);

            let mut list = column![].spacing(4);
            for game in detected {
                let rating_text = game
                    .rating
                    .as_ref()
                    .map(|r| r.label().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let game_row = container(
                    row![
                        text(&game.name)
                            .size(14)
                            .color(theme::TEXT_PRIMARY)
                            .width(Length::Fill),
                        text(rating_text)
                            .size(12)
                            .color(theme::TEXT_SECONDARY),
                        {
                            let action_element: Element<'_, Message> = if game.bundle_available {
                                button(
                                    text("Set Up").size(12).color(theme::BUTTON_GREEN_TEXT),
                                )
                                .on_press(Message::ApplyBundle(game.name.clone()))
                                .padding([4, 12])
                                .style(|_theme, _status| button::Style {
                                    background: Some(Background::Color(theme::BUTTON_GREEN)),
                                    text_color: theme::BUTTON_GREEN_TEXT,
                                    border: Border::default().rounded(4),
                                    ..button::Style::default()
                                })
                                .into()
                            } else {
                                text("Manual setup needed")
                                    .size(12)
                                    .color(theme::TEXT_SECONDARY)
                                    .into()
                            };
                            action_element
                        },
                    ]
                    .spacing(12)
                    .align_y(iced::Alignment::Center),
                )
                .padding([8, 16])
                .width(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(iced::Color::from_rgba(
                        1.0, 1.0, 1.0, 0.03,
                    ))),
                    border: Border::default().rounded(4),
                    ..container::Style::default()
                });
                list = list.push(game_row);
            }

            let go_btn = button(
                text("Continue to Library").size(16).color(theme::BUTTON_GREEN_TEXT),
            )
            .on_press(Message::FinishFirstRun)
            .padding([12, 32])
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(theme::BUTTON_GREEN)),
                text_color: theme::BUTTON_GREEN_TEXT,
                border: Border::default().rounded(8),
                ..button::Style::default()
            });

            let scroll = scrollable(list).height(300);

            column![title, subtitle, scroll, go_btn]
                .spacing(16)
                .align_x(iced::Alignment::Center)
                .width(600)
                .into()
        }
        FirstRunPhase::Error(msg) => {
            let title = text("Something went wrong")
                .size(24)
                .color(iced::Color::from_rgb(0.9, 0.3, 0.3));

            let detail = text(msg)
                .size(14)
                .color(theme::TEXT_SECONDARY);

            let settings_btn = button(
                text("Go to Settings").size(14).color(theme::TEXT_PRIMARY),
            )
            .on_press(Message::NavigateTo(crate::app::Screen::Settings))
            .padding([10, 24])
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(iced::Color::from_rgba(
                    1.0, 1.0, 1.0, 0.1,
                ))),
                text_color: theme::TEXT_PRIMARY,
                border: Border::default().rounded(6),
                ..button::Style::default()
            });

            column![title, detail, settings_btn]
                .spacing(16)
                .align_x(iced::Alignment::Center)
                .into()
        }
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
