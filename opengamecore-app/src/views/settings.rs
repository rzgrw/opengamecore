use iced::widget::{button, column, container, row, text, Scrollable};
use iced::{Background, Border, Element, Length};

use opengamecore_lib::WineConfig;

use crate::app::Message;
use crate::theme;

pub fn view<'a>(
    wine_configs: &'a [WineConfig],
    download_urls: &'a [String],
    default_wine: &'a str,
) -> Element<'a, Message> {
    let header = text("Settings").size(24).color(theme::TEXT_PRIMARY);

    // Wine installations section
    let wine_header = text("Wine Installations")
        .size(18)
        .color(theme::TEXT_PRIMARY);

    let mut wine_list = column![].spacing(8);

    if wine_configs.is_empty() {
        wine_list = wine_list.push(
            text("No Wine installations found")
                .size(14)
                .color(theme::TEXT_SECONDARY),
        );
    } else {
        for config in wine_configs {
            let name = config.name.clone();
            let is_default = config.name == default_wine;

            let mut set_default_btn = button(
                text(if is_default { "Default" } else { "Set Default" })
                    .size(13)
                    .color(if is_default {
                        theme::ACCENT
                    } else {
                        theme::TEXT_SECONDARY
                    }),
            )
            .padding([6, 14])
            .style(move |_theme, _status| button::Style {
                background: if is_default {
                    Some(Background::Color(iced::Color::from_rgba(
                        0.39, 1.0, 0.855, 0.1,
                    )))
                } else {
                    Some(Background::Color(iced::Color::from_rgba(
                        1.0, 1.0, 1.0, 0.05,
                    )))
                },
                text_color: theme::TEXT_SECONDARY,
                border: Border::default().rounded(4),
                ..button::Style::default()
            });

            if !is_default {
                set_default_btn = set_default_btn.on_press(Message::SetDefaultWine(name));
            }

            let card = container(
                row![
                    column![
                        text(&config.name).size(15).color(theme::TEXT_PRIMARY),
                        text(config.binary_path.display().to_string())
                            .size(12)
                            .color(theme::TEXT_SECONDARY),
                    ]
                    .spacing(4),
                    iced::widget::horizontal_space(),
                    set_default_btn,
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

            wine_list = wine_list.push(card);
        }
    }

    let add_wine_btn = button(
        text("+ Add Custom Wine Path")
            .size(14)
            .color(theme::ACCENT),
    )
    .on_press(Message::AddCustomWinePath)
    .padding([8, 16])
    .style(|_theme, _status| button::Style {
        background: Some(Background::Color(iced::Color::from_rgba(
            1.0, 1.0, 1.0, 0.05,
        ))),
        text_color: theme::ACCENT,
        border: Border::default().rounded(6),
        ..button::Style::default()
    });

    // Download sources section
    let sources_header = text("Download Sources")
        .size(18)
        .color(theme::TEXT_PRIMARY);

    let mut sources_list = column![].spacing(4);
    for url in download_urls {
        sources_list = sources_list.push(
            container(
                text(url).size(12).color(theme::TEXT_SECONDARY),
            )
            .padding([8, 12])
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(iced::Color::from_rgba(
                    0.0, 0.0, 0.0, 0.2,
                ))),
                border: Border::default().rounded(4),
                ..container::Style::default()
            }),
        );
    }

    let content = column![
        wine_header,
        wine_list,
        add_wine_btn,
        sources_header,
        sources_list,
    ]
    .spacing(16);

    let scrollable = Scrollable::new(
        column![header, content]
            .spacing(20)
            .padding(24)
            .width(Length::Fill),
    );

    container(scrollable)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
