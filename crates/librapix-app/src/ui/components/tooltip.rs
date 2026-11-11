use crate::ui::{styles::container::tooltip_style, tokens::*};
use iced::{
    Element,
    widget::{Tooltip, text, tooltip},
};

pub fn app_tooltip<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
) -> Tooltip<'a, Message> {
    tooltip(
        content,
        text(label).size(FONT_CAPTION),
        tooltip::Position::Bottom,
    )
    .gap(SPACE_XS as f32)
    .padding(SPACE_SM as f32)
    .style(tooltip_style)
}
