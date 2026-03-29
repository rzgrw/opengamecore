use iced::widget::{button, column, container, row, text, text_input, Scrollable};
use iced::{Background, Border, Element, Length};

use opengamecore_lib::bottle::BottleInfo;
use opengamecore_lib::{GameLibrary, WineConfig};

use crate::app::Message;
use crate::theme;

pub fn view<'a>(
    bottles: &'a [BottleInfo],
    library: &'a GameLibrary,
    wine_configs: &'a [WineConfig],
) -> Element<'a, Message> {
    let header = text("Bottles & Game Settings")
        .size(24)
        .color(theme::TEXT_PRIMARY);

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
        let mut list = column![].spacing(12);
        for bottle in bottles {
            list = list.push(bottle_card(bottle, library, wine_configs));
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

fn bottle_card<'a>(
    bottle: &'a BottleInfo,
    library: &'a GameLibrary,
    wine_configs: &'a [WineConfig],
) -> Element<'a, Message> {
    let slug = bottle.slug.clone();
    let size_mb = bottle.size_bytes / (1024 * 1024);
    let game = library.find(&slug);

    // Header row: name, size, path
    let title = text(&bottle.slug).size(18).color(theme::TEXT_PRIMARY);
    let size_text = text(format!("{} MB", size_mb))
        .size(12)
        .color(theme::TEXT_SECONDARY);
    let path_text = text(bottle.path.display().to_string())
        .size(11)
        .color(theme::TEXT_SECONDARY);

    let mut card_content = column![
        row![title, iced::widget::horizontal_space(), size_text].align_y(iced::Alignment::Center),
        path_text,
    ]
    .spacing(6);

    // Game settings (if a matching game exists in library)
    if let Some(game) = game {
        let exe_slug = slug.clone();
        let exe_val = game.exe.clone();

        // Exe path
        let exe_row = row![
            text("Executable:")
                .size(13)
                .color(theme::TEXT_SECONDARY)
                .width(100),
            text_input("drive_c/path/to/game.exe", &exe_val)
                .on_input(move |val| Message::ChangeExePath(exe_slug.clone(), val))
                .padding(6)
                .size(13)
                .width(Length::Fill),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        // Wine config selector
        let wine_slug = slug.clone();
        let current_wine = game.wine_config.clone();
        let mut wine_row = row![text("Wine:")
            .size(13)
            .color(theme::TEXT_SECONDARY)
            .width(100),]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        for wc in wine_configs {
            let is_selected = wc.name == current_wine;
            let ws = wine_slug.clone();
            let wname = wc.name.clone();
            wine_row = wine_row.push(
                button(text(&wc.name).size(12).color(if is_selected {
                    theme::ACCENT
                } else {
                    theme::TEXT_SECONDARY
                }))
                .on_press(Message::ChangeWineConfig(ws, wname))
                .padding([4, 10])
                .style(move |_theme, _status| button::Style {
                    background: if is_selected {
                        Some(Background::Color(iced::Color::from_rgba(
                            0.39, 1.0, 0.855, 0.12,
                        )))
                    } else {
                        Some(Background::Color(iced::Color::from_rgba(
                            1.0, 1.0, 1.0, 0.05,
                        )))
                    },
                    text_color: theme::TEXT_SECONDARY,
                    border: Border::default().rounded(4),
                    ..button::Style::default()
                }),
            );
        }

        // Toggles
        let dxvk_slug = slug.clone();
        let gptk_slug = slug.clone();
        let dxvk_btn = toggle_button("DXVK", game.dxvk_enabled, Message::ToggleDxvk(dxvk_slug));
        let gptk_btn = toggle_button("GPTK", game.use_gptk, Message::ToggleGptk(gptk_slug));

        let toggles_row = row![
            text("Features:")
                .size(13)
                .color(theme::TEXT_SECONDARY)
                .width(100),
            dxvk_btn,
            gptk_btn,
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        card_content = card_content
            .push(
                container(text("").size(1))
                    .width(Length::Fill)
                    .height(1)
                    .style(|_theme| container::Style {
                        background: Some(Background::Color(iced::Color::from_rgba(
                            1.0, 1.0, 1.0, 0.08,
                        ))),
                        ..container::Style::default()
                    }),
            )
            .push(exe_row)
            .push(wine_row)
            .push(toggles_row);
    } else {
        card_content = card_content.push(
            text("No game linked to this bottle")
                .size(13)
                .color(theme::TEXT_SECONDARY),
        );
    }

    // Action buttons
    let slug_reset = slug.clone();
    let slug_delete = slug.clone();
    let slug_finder = slug.clone();

    let actions = row![
        button(text("Open in Finder").size(12).color(theme::TEXT_SECONDARY))
            .on_press(Message::OpenInFinder(slug_finder))
            .padding([6, 12])
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(iced::Color::from_rgba(
                    1.0, 1.0, 1.0, 0.05,
                ))),
                text_color: theme::TEXT_SECONDARY,
                border: Border::default().rounded(4),
                ..button::Style::default()
            }),
        button(text("Reset").size(12).color(theme::TEXT_SECONDARY))
            .on_press(Message::ResetBottle(slug_reset))
            .padding([6, 12])
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
                .size(12)
                .color(iced::Color::from_rgb(0.9, 0.3, 0.3)),
        )
        .on_press(Message::DeleteBottle(slug_delete))
        .padding([6, 12])
        .style(|_theme, _status| button::Style {
            background: Some(Background::Color(iced::Color::from_rgba(
                0.9, 0.3, 0.3, 0.15,
            ))),
            text_color: iced::Color::from_rgb(0.9, 0.3, 0.3),
            border: Border::default().rounded(4),
            ..button::Style::default()
        }),
    ]
    .spacing(8);

    card_content = card_content.push(actions);

    container(card_content)
        .padding(16)
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme::BG_CARD)),
            border: Border::default().rounded(8),
            ..container::Style::default()
        })
        .into()
}

fn toggle_button(label: &str, enabled: bool, msg: Message) -> Element<'_, Message> {
    let display = format!("{}: {}", label, if enabled { "ON" } else { "OFF" });
    let color = if enabled {
        theme::ACCENT
    } else {
        theme::TEXT_SECONDARY
    };
    button(text(display).size(12).color(color))
        .on_press(msg)
        .padding([4, 10])
        .style(move |_theme, _status| button::Style {
            background: Some(Background::Color(if enabled {
                iced::Color::from_rgba(0.39, 1.0, 0.855, 0.12)
            } else {
                iced::Color::from_rgba(1.0, 1.0, 1.0, 0.05)
            })),
            text_color: color,
            border: Border::default().rounded(4),
            ..button::Style::default()
        })
        .into()
}
