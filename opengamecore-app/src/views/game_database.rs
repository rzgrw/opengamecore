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

fn game_row<'a>(entry: &CompatEntry) -> Element<'a, Message> {
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

    let backend = text(entry.recommended_backend.to_string())
        .size(12)
        .color(theme::TEXT_SECONDARY)
        .width(80);

    let confidence = text(format!("{:.0}%", entry.confidence * 100.0))
        .size(12)
        .color(theme::TEXT_SECONDARY)
        .width(50);

    let mut row_content = row![name, badge, backend, confidence]
        .spacing(12)
        .align_y(iced::Alignment::Center);

    if entry.bundle_available {
        let add_btn = button(text("Add").size(12).color(theme::BUTTON_GREEN_TEXT))
            .on_press(Message::SetupFromDatabase(slug))
            .padding([4, 12])
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(theme::BUTTON_GREEN)),
                text_color: theme::BUTTON_GREEN_TEXT,
                border: Border::default().rounded(4),
                ..button::Style::default()
            });
        row_content = row_content.push(add_btn);
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
) -> Element<'a, Message> {
    let title = text("Game Compatibility Database")
        .size(24)
        .color(theme::TEXT_PRIMARY);

    let search = text_input("Search games...", search_query)
        .on_input(Message::SearchChanged)
        .padding(8)
        .size(14)
        .width(300);

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

    let header = row![search, filters]
        .spacing(16)
        .align_y(iced::Alignment::Center);

    let mut content = column![title, header].spacing(16).padding(24);

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
                let count_text = text(format!("{} game(s)", entries.len()))
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
                text("Compatibility database not loaded.")
                    .size(14)
                    .color(theme::TEXT_SECONDARY),
            );
        }
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
