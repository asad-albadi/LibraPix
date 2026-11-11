pub use crate::ui::styles::container::CardVariant;
use crate::ui::styles::{button::card_action_style, container::card_surface_style};
use iced::{
    Element,
    widget::{Button, Container, button, container},
};

pub fn card<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    variant: CardVariant,
) -> Container<'a, Message> {
    container(content).style(move |theme| card_surface_style(theme, variant))
}

/// A single button, not a nested card wrapper, for the viewport-bounded media grid.
pub fn interactive_card<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    on_press: Option<Message>,
    selected: bool,
) -> Button<'a, Message> {
    let variant = if selected {
        CardVariant::Selected
    } else {
        CardVariant::Interactive
    };
    button(content)
        .on_press_maybe(on_press)
        .padding(0)
        .style(move |theme, status| card_action_style(theme, status, variant))
}
