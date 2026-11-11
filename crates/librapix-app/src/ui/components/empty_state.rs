use super::card::{CardVariant, card};
use crate::ui::{
    styles::text::{text_primary, text_secondary},
    tokens::*,
};
use iced::{
    Alignment, Element, Length,
    widget::{Container, column, text},
};

pub fn empty_state<'a, Message: 'a>(
    icon: impl Into<Element<'a, Message>>,
    title: &'a str,
    help: Option<&'a str>,
    action: Option<Element<'a, Message>>,
) -> Container<'a, Message> {
    let mut content = column![
        icon.into(),
        text(title).size(FONT_SUBTITLE).style(text_primary)
    ]
    .spacing(SPACE_MD)
    .align_x(Alignment::Center);
    if let Some(help) = help {
        content = content.push(text(help).size(FONT_BODY).style(text_secondary));
    }
    if let Some(action) = action {
        content = content.push(action);
    }
    card(content, CardVariant::Default)
        .padding(SPACE_XL as u16)
        .center_x(Length::Fill)
}
