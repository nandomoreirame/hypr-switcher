use iced::widget::{column, container, image, svg, text, Row, scrollable};
use iced::{Alignment, Element, Length, Padding};

use crate::hyprland::types::WindowEntry;
use crate::ui::style;

/// Renders the window list as horizontal cards with the selected item highlighted.
pub fn window_list_view<'a, M: Clone + 'a>(
    windows: &'a [WindowEntry],
    selected_index: usize,
) -> Element<'a, M> {
    let items: Vec<Element<'a, M>> = windows
        .iter()
        .enumerate()
        .map(|(i, entry)| window_card(entry, i == selected_index))
        .collect();

    let cards_row = Row::with_children(items)
        .spacing(style::ITEM_SPACING)
        .align_y(Alignment::Center);

    let scrollable_row = scrollable(cards_row)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new(),
        ))
        .width(Length::Shrink);

    container(scrollable_row)
        .max_width(style::CARD_MAX_WIDTH)
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
