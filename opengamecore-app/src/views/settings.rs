use iced::widget::{button, column, container, row, text, text_input, Scrollable};
use iced::{Background, Border, Element, Length};

use opengamecore_lib::bottle::BottleInfo;
use opengamecore_lib::{GameLibrary, WineConfig};

use crate::app::Message;
use crate::theme;

#[allow(clippy::too_many_arguments)]
pub fn view<'a>(
    wine_configs: &'a [WineConfig],
    download_urls: &'a [String],
    default_wine: &'a str,
    dxvk_dir: Option<&'a std::path::Path>,
    installing_steam: bool,
    bottles: &'a [BottleInfo],
    library: &'a GameLibrary,
) -> Element<'a, Message> {
    let header = text("Settings").size(24).color(theme::TEXT_PRIMARY);

    // --- Wine section ---
    let wine_header = text("Wine").size(18).color(theme::TEXT_PRIMARY);

    let mut wine_list = column![].spacing(8);

    if wine_configs.is_empty() {
        wine_list = wine_list.push(
            column![
                text("No Wine installations found")
                    .size(14)
                    .color(theme::TEXT_SECONDARY),
                text("Wine is required to run Windows games. Download it below or add a custom path.")
                    .size(13)
                    .color(theme::TEXT_SECONDARY),
                button(
                    text("Download Wine")
                        .size(16)
                        .color(theme::BUTTON_GREEN_TEXT),
                )
                .on_press(Message::NavigateTo(crate::app::Screen::FirstRun))
                .padding([12, 32])
                .style(|_theme, _status| button::Style {
                    background: Some(Background::Color(theme::BUTTON_GREEN)),
                    text_color: theme::BUTTON_GREEN_TEXT,
                    border: Border::default().rounded(8),
                    ..button::Style::default()
                }),
            ]
            .spacing(10),
        );
    } else {
        for config in wine_configs {
            let name = config.name.clone();
            let is_default = config.name == default_wine;
            let path_str = config.binary_path.display().to_string();

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
                column![
                    row![
                        text(&config.name).size(15).color(theme::TEXT_PRIMARY),
                        iced::widget::horizontal_space(),
                        set_default_btn,
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                    text(path_str).size(12).color(theme::TEXT_SECONDARY),
                ]
                .spacing(4),
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

    let add_wine_btn = button(text("+ Add Custom Wine Path").size(14).color(theme::ACCENT))
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

    // --- Quick Setup section ---
    let quick_setup_header = text("Quick Setup").size(18).color(theme::TEXT_PRIMARY);

    let install_steam_label = if installing_steam {
        "Installing..."
    } else {
        "Install Steam"
    };

    let mut install_steam_btn = button(
        text(install_steam_label)
            .size(14)
            .color(theme::BUTTON_GREEN_TEXT),
    )
    .padding([8, 16])
    .style(|_theme, _status| button::Style {
        background: Some(Background::Color(theme::BUTTON_GREEN)),
        text_color: theme::BUTTON_GREEN_TEXT,
        border: Border::default().rounded(6),
        ..button::Style::default()
    });

    if !installing_steam {
        install_steam_btn = install_steam_btn.on_press(Message::InstallSteam);
    }

    let quick_setup_desc = text("Download and install Steam into a dedicated Wine bottle.")
        .size(13)
        .color(theme::TEXT_SECONDARY);

    let quick_setup_section = column![quick_setup_desc, install_steam_btn].spacing(8);

    // --- Library section ---
    let library_header = text("Library").size(18).color(theme::TEXT_PRIMARY);

    let export_btn = button(text("Export Library").size(14).color(theme::ACCENT))
        .on_press(Message::ExportLibrary)
        .padding([8, 16])
        .style(|_theme, _status| button::Style {
            background: Some(Background::Color(iced::Color::from_rgba(
                1.0, 1.0, 1.0, 0.05,
            ))),
            text_color: theme::ACCENT,
            border: Border::default().rounded(6),
            ..button::Style::default()
        });

    let import_btn = button(text("Import Library").size(14).color(theme::ACCENT))
        .on_press(Message::ImportLibrary)
        .padding([8, 16])
        .style(|_theme, _status| button::Style {
            background: Some(Background::Color(iced::Color::from_rgba(
                1.0, 1.0, 1.0, 0.05,
            ))),
            text_color: theme::ACCENT,
            border: Border::default().rounded(6),
            ..button::Style::default()
        });

    let library_section = column![row![export_btn, import_btn].spacing(8),].spacing(8);

    // --- Advanced section ---
    let advanced_header = text("Advanced").size(18).color(theme::TEXT_PRIMARY);

    // Bottles list
    let mut bottles_content = column![].spacing(12);
    if bottles.is_empty() {
        bottles_content = bottles_content.push(
            text("No game configurations yet.")
                .size(14)
                .color(theme::TEXT_SECONDARY),
        );
    } else {
        let bottles_label = text("Game Configurations")
            .size(15)
            .color(theme::TEXT_PRIMARY);
        bottles_content = bottles_content.push(bottles_label);
        for bottle in bottles {
            bottles_content = bottles_content.push(bottle_card(bottle, library, wine_configs));
        }
    }

    // GPTK info
    let gptk_header = text("Game Porting Toolkit")
        .size(15)
        .color(theme::TEXT_PRIMARY);

    let gptk_detected: Vec<&WineConfig> = wine_configs
        .iter()
        .filter(|c| c.name.contains("gptk"))
        .collect();

    let gptk_section: Element<'_, Message> = if gptk_detected.is_empty() {
        text("GPTK not detected.")
            .size(14)
            .color(theme::TEXT_SECONDARY)
            .into()
    } else {
        let mut items = column![].spacing(4);
        for cfg in &gptk_detected {
            items = items.push(
                text(format!("{} ({})", &cfg.name, cfg.binary_path.display()))
                    .size(13)
                    .color(theme::BADGE_GPTK),
            );
        }
        items.into()
    };

    // Download sources
    let sources_header = text("Download Sources").size(15).color(theme::TEXT_PRIMARY);

    let mut sources_list = column![].spacing(4);
    for url in download_urls {
        sources_list = sources_list.push(
            container(text(url).size(12).color(theme::TEXT_SECONDARY))
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

    // DXVK status
    let dxvk_header = text("DXVK / MoltenVK").size(15).color(theme::TEXT_PRIMARY);

    let dxvk_status_text = if dxvk_dir.is_some() {
        text("DXVK is downloaded and ready")
            .size(14)
            .color(theme::ACCENT)
    } else {
        text("DXVK is not downloaded")
            .size(14)
            .color(theme::TEXT_SECONDARY)
    };

    let download_dxvk_btn = button(
        text("Download DXVK")
            .size(14)
            .color(theme::BUTTON_GREEN_TEXT),
    )
    .on_press(Message::DownloadDxvk)
    .padding([8, 16])
    .style(|_theme, _status| button::Style {
        background: Some(Background::Color(theme::BUTTON_GREEN)),
        text_color: theme::BUTTON_GREEN_TEXT,
        border: Border::default().rounded(6),
        ..button::Style::default()
    });

    let dxvk_section = column![dxvk_status_text, download_dxvk_btn].spacing(8);

    let advanced_section = column![
        bottles_content,
        gptk_header,
        gptk_section,
        sources_header,
        sources_list,
        dxvk_header,
        dxvk_section,
    ]
    .spacing(16);

    let content = column![
        wine_header,
        wine_list,
        add_wine_btn,
        quick_setup_header,
        quick_setup_section,
        library_header,
        library_section,
        advanced_header,
        advanced_section,
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

fn bottle_card<'a>(
    bottle: &'a BottleInfo,
    library: &'a GameLibrary,
    wine_configs: &'a [WineConfig],
) -> Element<'a, Message> {
    let slug = bottle.slug.clone();
    let size_mb = bottle.size_bytes / (1024 * 1024);
    let game = library.find(&slug);

    let title = text(&bottle.slug).size(15).color(theme::TEXT_PRIMARY);
    let size_text = text(format!("{} MB", size_mb))
        .size(12)
        .color(theme::TEXT_SECONDARY);

    let mut card_content =
        column![row![title, iced::widget::horizontal_space(), size_text]
            .align_y(iced::Alignment::Center),]
        .spacing(6);

    if let Some(game) = game {
        let exe_slug = slug.clone();
        let exe_val = game.exe.clone();

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
    }

    // Action buttons
    let slug_reset = slug.clone();
    let slug_delete = slug.clone();
    let slug_finder = slug;

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
