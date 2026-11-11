use crate::ui::{styles::checkbox::checkbox_style, tokens::*};
use iced::widget::{Checkbox, checkbox};

pub fn app_checkbox<'a, Message: 'a>(
    label: &'a str,
    checked: bool,
    on_toggle: Option<impl Fn(bool) -> Message + 'a>,
) -> Checkbox<'a, Message> {
    checkbox(checked)
        .label(label)
        .on_toggle_maybe(on_toggle)
        .size(ICON_SM)
        .text_size(FONT_BODY)
        .spacing(SPACE_SM as f32)
        .style(checkbox_style)
}
