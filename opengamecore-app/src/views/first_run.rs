use iced::widget::{button, column, container, progress_bar, text};
use iced::{Background, Border, Element, Length};

use crate::app::Message;
use crate::theme;

#[derive(Debug, Clone)]
pub enum FirstRunPhase {
    Welcome,
    Downloading { progress: f32, status: String },
    CreatingTemplate,
    Done,
    Error(String),
}

impl Default for FirstRunPhase {
    fn default() -> Self {
        Self::Welcome
    }
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
