use iced::widget::{column, container, image, svg, text, Column, Row};
use iced::{Alignment, Element, Length, Padding};

use crate::hyprland::types::WindowEntry;
use crate::ui::style;

/// Calculates how many items fit in a single grid row for the given available width.
/// Each card occupies CARD_ITEM_WIDTH + 2*ITEM_PADDING on screen, plus ITEM_SPACING between cards.
pub fn calc_items_per_row(available_width: f32) -> usize {
    let width = available_width.min(style::GRID_MAX_WIDTH);
    let card_width = style::CARD_ITEM_WIDTH + 2.0 * style::ITEM_PADDING;
    let inner_width = width - 2.0 * style::CARD_PADDING;
    let result = ((inner_width + style::ITEM_SPACING) / (card_width + style::ITEM_SPACING)).floor() as usize;
    result.max(1)
}

/// Renders the window list as a grid of cards with the selected item highlighted.
pub fn window_list_view<'a, M: Clone + 'a>(
    windows: &'a [WindowEntry],
    selected_index: usize,
    items_per_row: usize,
) -> Element<'a, M> {
    let per_row = items_per_row.max(1);

    let rows: Vec<Element<'a, M>> = windows
        .chunks(per_row)
        .enumerate()
        .map(|(chunk_idx, chunk)| {
            let items: Vec<Element<'a, M>> = chunk
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let global_index = chunk_idx * per_row + i;
                    window_card(entry, global_index == selected_index)
                })
                .collect();

            Row::with_children(items)
                .spacing(style::ITEM_SPACING)
                .align_y(Alignment::Center)
                .into()
        })
        .collect();

    let grid = Column::with_children(rows)
        .spacing(style::GRID_ROW_SPACING)
        .align_x(Alignment::Center);

    container(grid)
        .max_width(style::GRID_MAX_WIDTH)
        .padding(style::CARD_PADDING)
        .style(card_container_style)
        .into()
}

/// Truncates a string to max_len characters, adding "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        let end = s.char_indices()
            .nth(max_len.saturating_sub(3))
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        format!("{}...", &s[..end])
    } else {
        s.to_string()
    }
}

/// Renders a single window card (vertical layout: icon + class + title + ws badge).
fn window_card<'a, M: Clone + 'a>(entry: &'a WindowEntry, selected: bool) -> Element<'a, M> {
    let mut content_items: Vec<Element<'a, M>> = Vec::new();

    // Icon (if available)
    if let Some(icon_path) = &entry.icon_path {
        let icon_element: Element<'a, M> = if icon_path.extension().is_some_and(|ext| ext == "svg") {
            svg(svg::Handle::from_path(icon_path))
                .width(style::ICON_SIZE)
                .height(style::ICON_SIZE)
                .into()
        } else {
            image(image::Handle::from_path(icon_path))
                .width(style::ICON_SIZE)
                .height(style::ICON_SIZE)
                .into()
        };

        content_items.push(
            container(icon_element)
                .center_x(Length::Fill)
                .into(),
        );
    }

    // App class name (prominent)
    let class_label = text(&entry.class)
        .size(13)
        .color(style::TEXT_COLOR);
    content_items.push(class_label.into());

    // Window title (dimmer, truncated)
    let title_text = truncate(&entry.title, 24);
    let title_label = text(title_text)
        .size(11)
        .color(style::TEXT_DIM_COLOR);
    content_items.push(title_label.into());

    // Workspace badge
    let badge = container(
        text(format!("ws {}", entry.workspace_name))
            .size(10)
            .color(style::TEXT_DIM_COLOR),
    )
    .padding(Padding::from([2.0, style::WORKSPACE_BADGE_PADDING]))
    .style(badge_style);
    content_items.push(badge.into());

    let content = column(content_items)
        .spacing(4)
        .align_x(Alignment::Center)
        .width(Length::Fixed(style::CARD_ITEM_WIDTH));

    container(content)
        .padding(Padding::from([style::ITEM_PADDING, style::ITEM_PADDING]))
        .center_x(Length::Shrink)
        .style(if selected {
            selected_item_style
        } else {
            item_style
        })
        .into()
}

fn card_container_style(theme: &iced::Theme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(iced::Background::Color(style::CARD_BG_COLOR)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: style::CARD_BORDER_RADIUS.into(),
        },
        ..Default::default()
    }
}

fn selected_item_style(theme: &iced::Theme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(iced::Background::Color(style::SELECTED_BG_COLOR)),
        border: iced::Border {
            color: style::ACCENT_COLOR,
            width: 1.5,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn item_style(theme: &iced::Theme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(iced::Background::Color(style::ITEM_BG_COLOR)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn badge_style(theme: &iced::Theme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(iced::Background::Color(style::BADGE_BG_COLOR)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_items_per_row_large_monitor() {
        // 1920px: capped to GRID_MAX_WIDTH=1800
        // card_width = 140 + 2*12 = 164, slot = 164 + 6 = 170
        // inner = 1800 - 2*10 = 1780, available = 1780 + 6 = 1786
        // floor(1786 / 170) = 10
        assert_eq!(calc_items_per_row(1920.0), 10);
    }

    #[test]
    fn test_calc_items_per_row_small_monitor() {
        // 1024px monitor: inner = 1024 - 20 = 1004, (1004+6)/170 = 5.94 → 5
        assert_eq!(calc_items_per_row(1024.0), 5);
    }

    #[test]
    fn test_calc_items_per_row_minimum() {
        // Very small width still returns at least 1
        assert_eq!(calc_items_per_row(100.0), 1);
    }
}
