use iced::widget::{button, column, container, row, text, Scrollable};
use iced::{Background, Border, Element, Length};

use opengamecore_lib::bottle::BottleInfo;

use crate::app::Message;
use crate::theme;

pub fn view(bottles: &[BottleInfo]) -> Element<'_, Message> {
    let header = text("Bottles").size(24).color(theme::TEXT_PRIMARY);

    let content: Element<'_, Message> = if bottles.is_empty() {
        container(
            text("No bottles yet. Bottles are created when you add games.")
                .size(14)
                .color(theme::TEXT_SECONDARY),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        let mut list = column![].spacing(8);
        for bottle in bottles {
            let slug = bottle.slug.clone();
            let slug2 = bottle.slug.clone();
            let size_mb = bottle.size_bytes / (1024 * 1024);

            let card = container(
                row![
                    column![
                        text(&bottle.slug).size(16).color(theme::TEXT_PRIMARY),
                        text(format!("{} MB", size_mb))
                            .size(12)
                            .color(theme::TEXT_SECONDARY),
                    ]
                    .spacing(4),
                    iced::widget::horizontal_space(),
                    button(text("Reset").size(13).color(theme::TEXT_SECONDARY))
                        .on_press(Message::ResetBottle(slug))
                        .padding([6, 14])
                        .style(|_theme, _status| button::Style {
                            background: Some(Background::Color(iced::Color::from_rgba(
                                1.0, 1.0, 1.0, 0.05,
                            ))),
                            text_color: theme::TEXT_SECONDARY,
                            border: Border::default().rounded(4),
                            ..button::Style::default()
                        }),
                    button(
                        text("Delete")
                            .size(13)
                            .color(iced::Color::from_rgb(0.9, 0.3, 0.3)),
                    )
                    .on_press(Message::DeleteBottle(slug2))
                    .padding([6, 14])
                    .style(|_theme, _status| button::Style {
                        background: Some(Background::Color(iced::Color::from_rgba(
                            0.9, 0.3, 0.3, 0.15,
                        ))),
                        text_color: iced::Color::from_rgb(0.9, 0.3, 0.3),
                        border: Border::default().rounded(4),
                        ..button::Style::default()
                    })
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            )
            .padding(12)
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(theme::BG_CARD)),
                border: Border::default().rounded(8),
                ..container::Style::default()
            });

            list = list.push(card);
        }
        Scrollable::new(list).into()
    };

    column![header, content]
        .spacing(16)
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
