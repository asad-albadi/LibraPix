use crate::ui::tokens::{RADIUS_PILL, palette};
use iced::widget::scrollable;
use iced::{Background, Border, Color, Theme};

/// Thin, rounded, translucent scrollbars: invisible rail, a pill scroller that
/// darkens on hover/drag. Reads as a modern overlay scrollbar in both themes.
pub fn scrollbar_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let p = palette(theme);
    let mut style = scrollable::default(theme, status);
    let scroller_color = match status {
        scrollable::Status::Dragged { .. } => p.text_secondary,
        scrollable::Status::Hovered {
            is_vertical_scrollbar_hovered: true,
            ..
        }
        | scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered: true,
            ..
        } => p.text_secondary,
        _ => p.text_tertiary.scale_alpha(0.7),
    };
    let rail = scrollable::Rail {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: RADIUS_PILL.into(),
        },
        scroller: scrollable::Scroller {
            background: Background::Color(scroller_color),
            border: iced::border::rounded(RADIUS_PILL),
        },
    };
    style.vertical_rail = rail;
    style.horizontal_rail = rail;
    style
}
