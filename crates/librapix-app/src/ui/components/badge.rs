pub use crate::ui::styles::container::BadgeVariant;
use crate::ui::{styles::container::badge_style, tokens::*};
use iced::widget::{Container, container, text};

pub fn badge<'a, Message: 'a>(label: &'a str, variant: BadgeVariant) -> Container<'a, Message> {
    container(text(label).size(FONT_CAPTION))
        .padding([SPACE_2XS as u16, SPACE_SM as u16])
        .style(move |theme| badge_style(theme, variant))
}

/// Same single-container overlay as the media selection check, without a card wrapper.
pub fn selection_check<'a, Message: 'a>() -> Container<'a, Message> {
    container(
        text("\u{2713}")
            .size(FONT_CAPTION)
            .color(iced::Color::WHITE),
    )
    .padding([SPACE_2XS as u16, SPACE_XS as u16])
    .style(crate::ui::styles::container::check_badge_style)
}
