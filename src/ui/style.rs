#![allow(dead_code)]

use iced::Color;

// Layout dimensions
pub const CARD_MAX_WIDTH: f32 = 900.0;
pub const CARD_ITEM_WIDTH: f32 = 140.0;
pub const ICON_SIZE: f32 = 32.0;
pub const ITEM_HEIGHT: f32 = 48.0;
pub const ITEM_PADDING: f32 = 12.0;
pub const ITEM_SPACING: f32 = 6.0;
pub const CARD_PADDING: f32 = 10.0;
pub const CARD_BORDER_RADIUS: f32 = 12.0;
pub const WORKSPACE_BADGE_PADDING: f32 = 6.0;

// Colors
pub const BG_COLOR: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.5,
};

pub const CARD_BG_COLOR: Color = Color {
    r: 0.12,
    g: 0.12,
    b: 0.14,
    a: 0.95,
};

/// Accent color: #7c3aed (purple)
pub const ACCENT_COLOR: Color = Color {
    r: 0.486,
    g: 0.227,
    b: 0.929,
    a: 1.0,
};

pub const TEXT_COLOR: Color = Color {
    r: 0.93,
    g: 0.93,
    b: 0.95,
    a: 1.0,
};

pub const TEXT_DIM_COLOR: Color = Color {
    r: 0.6,
    g: 0.6,
    b: 0.65,
    a: 1.0,
};

pub const SELECTED_BG_COLOR: Color = Color {
    r: 0.486,
    g: 0.227,
    b: 0.929,
    a: 0.25,
};

pub const ITEM_BG_COLOR: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 0.05,
};

pub const BADGE_BG_COLOR: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 0.1,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accent_color_is_purple() {
        // #7c3aed = rgb(124, 58, 237) = (0.486, 0.227, 0.929)
        assert!(ACCENT_COLOR.r > 0.4 && ACCENT_COLOR.r < 0.5);
        assert!(ACCENT_COLOR.g > 0.2 && ACCENT_COLOR.g < 0.3);
        assert!(ACCENT_COLOR.b > 0.9 && ACCENT_COLOR.b < 1.0);
    }

    #[test]
    fn test_bg_is_semi_transparent() {
        assert_eq!(BG_COLOR.a, 0.5);
    }

    #[test]
    fn test_card_dimensions() {
        assert_eq!(CARD_MAX_WIDTH, 900.0);
        assert_eq!(CARD_ITEM_WIDTH, 140.0);
        assert_eq!(ICON_SIZE, 32.0);
    }
}
