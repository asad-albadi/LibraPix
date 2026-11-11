pub use crate::ui::styles::container::BadgeVariant;
use crate::ui::{styles::container::badge_style, tokens::*};
use iced::widget::{Container, container, text};

pub fn badge<'a, Message: 'a>(label: &'a str, variant: BadgeVariant) -> Container<'a, Message> {
    container(text(label).size(FONT_CAPTION))
        .padding([SPACE_2XS as u16, SPACE_SM as u16])
        .style(move |theme| badge_style(theme, variant))
}
