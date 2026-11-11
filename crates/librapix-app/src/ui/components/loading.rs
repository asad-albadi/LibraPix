use crate::ui::{
    styles::{container::preview_loading_block_style, progress::loading_dot_style},
    tokens::*,
};
use iced::{
    Alignment, Length,
    widget::{Container, Row, Space, container, row, text},
};

/// A fixed-size, three-dot indeterminate indicator. The caller supplies its timer
/// phase; this component does not own subscriptions or invent numeric progress.
pub fn loading_indicator<'a, Message: 'a>(phase: usize) -> Row<'a, Message> {
    Row::with_children((0..3).map(|dot| {
        container(Space::new().width(6).height(6))
            .style(move |theme| loading_dot_style(theme, dot == phase % 3))
            .into()
    }))
    .spacing(SPACE_XS)
    .align_y(Alignment::Center)
}

pub fn working_row<'a, Message: 'a>(stage: &'a str, phase: usize) -> Row<'a, Message> {
    row![
        loading_indicator(phase),
        text(stage)
            .size(FONT_BODY)
            .style(crate::ui::styles::text::text_primary)
    ]
    .spacing(SPACE_SM)
    .align_y(Alignment::Center)
}

pub fn skeleton<'a, Message: 'a>(
    width: impl Into<Length>,
    height: impl Into<Length>,
) -> Container<'a, Message> {
    container(Space::new())
        .width(width)
        .height(height)
        .style(preview_loading_block_style)
}
