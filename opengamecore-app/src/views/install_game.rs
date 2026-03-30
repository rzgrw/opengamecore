use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Background, Border, Element, Length};

use opengamecore_lib::{CompatDatabase, CompatEntry, CompatRating};

use crate::app::Message;
use crate::theme;

fn rating_color(rating: &CompatRating) -> iced::Color {
    match rating {
        CompatRating::Platinum => iced::Color::from_rgb(0.0, 0.85, 0.45),
        CompatRating::Gold => iced::Color::from_rgb(1.0, 0.84, 0.0),
        CompatRating::Silver => iced::Color::from_rgb(0.75, 0.75, 0.75),
        CompatRating::Bronze => iced::Color::from_rgb(0.8, 0.5, 0.2),
        CompatRating::Borked => iced::Color::from_rgb(0.9, 0.2, 0.2),
    }
}

fn game_row(entry: &CompatEntry) -> Element<'_, Message> {
    let rating_badge_color = rating_color(&entry.rating);
    let rating_label = entry.rating.label().to_string();
    let slug = entry.slug.clone();

    let name = text(entry.name.clone())
        .size(14)
        .color(theme::TEXT_PRIMARY)
        .width(Length::Fill);

    let badge = container(text(rating_label).size(12).color(iced::Color::BLACK))
        .padding([2, 8])
        .style(move |_theme| container::Style {
            background: Some(Background::Color(rating_badge_color)),
            border: Border::default().rounded(4),
            ..container::Style::default()
        });

    let mut row_content = row![name, badge]
        .spacing(12)
        .align_y(iced::Alignment::Center);

    if entry.bundle_available {
        let install_btn = button(text("Install").size(12).color(theme::BUTTON_GREEN_TEXT))
            .on_press(Message::SetupFromDatabase(slug))
            .padding([4, 12])
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(theme::BUTTON_GREEN)),
                text_color: theme::BUTTON_GREEN_TEXT,
                border: Border::default().rounded(4),
                ..button::Style::default()
            });
        row_content = row_content.push(install_btn);
    }

    container(row_content)
        .padding([8, 16])
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(iced::Color::from_rgba(
                1.0, 1.0, 1.0, 0.03,
            ))),
            border: Border::default().rounded(4),
            ..container::Style::default()
        })
        .into()
}

pub fn view<'a>(
    db: Option<&'a CompatDatabase>,
    search_query: &str,
    filter_rating: &Option<CompatRating>,
    custom_game_name: &str,
    custom_game_path: &'a Option<String>,
) -> Element<'a, Message> {
    let title = text("Install a Game").size(24).color(theme::TEXT_PRIMARY);

    let search = text_input("Search compatible games...", search_query)
        .on_input(Message::SearchChanged)
        .padding(8)
        .size(14)
        .width(Length::Fill);

    let filter_btn =
        |label: &str, rating: Option<CompatRating>, is_active: bool| -> Element<'static, Message> {
            let label = label.to_string();
            button(text(label).size(12).color(if is_active {
                theme::ACCENT
            } else {
                theme::TEXT_SECONDARY
            }))
            .on_press(Message::FilterRating(rating))
            .padding([4, 10])
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

    let filters = row![
        filter_btn("All", None, filter_rating.is_none()),
        filter_btn(
            "Platinum",
            Some(CompatRating::Platinum),
            *filter_rating == Some(CompatRating::Platinum)
        ),
        filter_btn(
            "Gold",
            Some(CompatRating::Gold),
            *filter_rating == Some(CompatRating::Gold)
        ),
        filter_btn(
            "Silver",
            Some(CompatRating::Silver),
            *filter_rating == Some(CompatRating::Silver)
        ),
        filter_btn(
            "Bronze",
            Some(CompatRating::Bronze),
            *filter_rating == Some(CompatRating::Bronze)
        ),
    ]
    .spacing(4);

    let header = column![search, filters].spacing(12);

    let mut content = column![title, header].spacing(16).padding(24);

    // Database game list
    match db {
        Some(db) => {
            let entries: Vec<&CompatEntry> = db
                .games
                .iter()
                .filter(|e| {
                    let matches_search = search_query.is_empty()
                        || e.name.to_lowercase().contains(&search_query.to_lowercase())
                        || e.slug.contains(&search_query.to_lowercase());
                    let matches_filter = filter_rating.as_ref().is_none_or(|r| e.rating == *r);
                    matches_search && matches_filter
                })
                .collect();

            if entries.is_empty() {
                content = content.push(
                    text("No games match your search.")
                        .size(14)
                        .color(theme::TEXT_SECONDARY),
                );
            } else {
                let count_text = text(format!("{} compatible game(s)", entries.len()))
                    .size(12)
                    .color(theme::TEXT_SECONDARY);
                content = content.push(count_text);

                let mut list = column![].spacing(4);
                for entry in entries {
                    list = list.push(game_row(entry));
                }
                content = content.push(scrollable(list).height(Length::Fill));
            }
        }
        None => {
            content = content.push(
                column![
                    text("Compatibility database is not available.")
                        .size(15)
                        .color(theme::TEXT_SECONDARY),
                    text("You can still add games manually using the section below.")
                        .size(13)
                        .color(theme::TEXT_SECONDARY),
                ]
                .spacing(6),
            );
        }
    }

    // Custom game section at the bottom
    let custom_header = text("Add Custom Game").size(18).color(theme::TEXT_PRIMARY);

    let custom_desc =
        text("Have a game folder not in the database? Browse to the folder, name it, and confirm.")
            .size(13)
            .color(theme::TEXT_SECONDARY);

    let path_display = custom_game_path
        .as_deref()
        .unwrap_or("No folder selected \u{2014} click Browse to pick a game folder");

    let browse_row = row![
        container(text(path_display).size(13).color(theme::TEXT_SECONDARY))
            .padding([8, 12])
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(iced::Color::from_rgba(
                    0.0, 0.0, 0.0, 0.3,
                ))),
                border: Border::default().rounded(4),
                ..container::Style::default()
            }),
        button(text("Browse").size(14).color(theme::TEXT_PRIMARY))
            .on_press(Message::InstallCustomGame)
            .padding([8, 16])
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(iced::Color::from_rgba(
                    1.0, 1.0, 1.0, 0.1,
                ))),
                text_color: theme::TEXT_PRIMARY,
                border: Border::default().rounded(4),
                ..button::Style::default()
            })
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let mut custom_section = column![custom_header, custom_desc, browse_row].spacing(8);

    if custom_game_path.is_some() {
        let name_input = text_input("Game name", custom_game_name)
            .on_input(Message::CustomGameNameChanged)
            .padding(8)
            .size(14);

        let can_add = !custom_game_name.trim().is_empty();
        let mut add_btn = button(text("Add Game").size(14).color(theme::BUTTON_GREEN_TEXT))
            .padding([8, 20])
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(theme::BUTTON_GREEN)),
                text_color: theme::BUTTON_GREEN_TEXT,
                border: Border::default().rounded(6),
                ..button::Style::default()
            });

        if can_add {
            add_btn = add_btn.on_press(Message::ConfirmCustomGame);
        }

        custom_section = custom_section.push(name_input).push(add_btn);
    }

    let custom_container = container(custom_section.padding(16))
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
        });

    content = content.push(custom_container);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
