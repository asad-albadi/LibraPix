use crate::ui::{styles::input::input_style, tokens::*};
use iced::widget::{TextInput, text_input};

/// Keep the Iced builder so views retain input IDs, focus routing and messages.
/// Without an `on_input` callback, Iced leaves this input disabled.
pub fn app_input<'a, Message: Clone + 'a>(
    placeholder: &'a str,
    value: &'a str,
    invalid: bool,
) -> TextInput<'a, Message> {
    text_input(placeholder, value)
        .size(FONT_BODY)
        .padding(SPACE_SM as u16)
        .style(move |theme, status| input_style(theme, status, invalid))
}
