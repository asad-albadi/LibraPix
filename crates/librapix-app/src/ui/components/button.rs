use crate::ui::{styles::button::button_style, tokens::*};
use iced::{
    Element, Length,
    widget::{Button, button, container},
};

pub use crate::ui::styles::button::ButtonVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Small,
    Default,
    Large,
    Icon,
}

pub fn app_button<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    on_press: Option<Message>,
    variant: ButtonVariant,
    size: ButtonSize,
) -> Button<'a, Message> {
    let (height, horizontal) = match size {
        ButtonSize::Small => (CONTROL_HEIGHT_SM, SPACE_SM),
        ButtonSize::Default => (CONTROL_HEIGHT_MD, SPACE_MD),
        ButtonSize::Large => (CONTROL_HEIGHT_LG, SPACE_LG),
        ButtonSize::Icon => (CONTROL_HEIGHT_MD, SPACE_XS),
    };
    let control = button(
        container(content)
            .center_x(Length::Shrink)
            .center_y(Length::Fill),
    )
    .on_press_maybe(on_press)
    .height(height)
    .padding([0, horizontal as u16])
    .style(move |theme, status| button_style(theme, status, variant));
    if size == ButtonSize::Icon {
        control.width(height)
    } else {
        control
    }
}
