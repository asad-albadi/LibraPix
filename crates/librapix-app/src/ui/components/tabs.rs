use super::button::{ButtonSize, ButtonVariant, app_button};
use crate::ui::{styles::button::tab_style, tokens::*};
use iced::{
    Alignment, Element,
    widget::{Button, Row},
};

pub fn tab_list<'a, Message: 'a>(
    triggers: impl IntoIterator<Item = Element<'a, Message>>,
) -> Row<'a, Message> {
    Row::with_children(triggers)
        .spacing(SPACE_XS)
        .align_y(Alignment::Center)
}

pub fn tab_trigger<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    on_press: Option<Message>,
    active: bool,
) -> Button<'a, Message> {
    app_button(content, on_press, ButtonVariant::Ghost, ButtonSize::Small)
        .style(move |theme, status| tab_style(theme, status, active))
}
