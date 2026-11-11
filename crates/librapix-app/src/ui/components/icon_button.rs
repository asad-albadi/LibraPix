use super::{
    button::{ButtonSize, ButtonVariant, app_button},
    tooltip::app_tooltip,
};
use iced::Element;

/// Labels are supplied by the caller's locale, even for disabled actions.
pub fn icon_button<'a, Message: Clone + 'a>(
    icon: Element<'a, Message>,
    label: &'a str,
    on_press: Option<Message>,
    variant: ButtonVariant,
) -> Element<'a, Message> {
    app_tooltip(app_button(icon, on_press, variant, ButtonSize::Icon), label).into()
}
