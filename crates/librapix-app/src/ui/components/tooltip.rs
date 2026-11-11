use crate::ui::{
    styles::{container::tooltip_style, text::text_tertiary},
    tokens::*,
};
use iced::{
    Element,
    widget::{Tooltip, text, tooltip},
};
use std::borrow::Cow;

pub fn app_tooltip<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    label: impl Into<Cow<'a, str>>,
) -> Tooltip<'a, Message> {
    tooltip(
        content,
        text(label.into()).size(FONT_CAPTION),
        tooltip::Position::Bottom,
    )
    .gap(SPACE_XS as f32)
    .padding(SPACE_SM as f32)
    .style(tooltip_style)
}

/// Compact contextual help without changing the surrounding control interaction.
pub fn help_hint<'a, Message: 'a>(label: &'a str) -> Element<'a, Message> {
    app_tooltip(text("(?)").size(FONT_CAPTION).style(text_tertiary), label).into()
}
