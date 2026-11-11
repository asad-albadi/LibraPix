//! Bounded dialog frame. The caller owns modality and backdrop/keyboard actions.

use crate::ui::{
    components::{
        button::{ButtonSize, ButtonVariant, app_button},
        separator::horizontal,
    },
    styles::{container::modal_dialog_style, scrollable::scrollbar_style, text::text_primary},
    tokens::*,
};
use iced::{
    Alignment, Element, Length,
    widget::{Space, column, container, row, scrollable, text},
};

pub struct DialogSpec<'a, Message> {
    pub title: &'a str,
    /// Localized label and existing close message; None omits the close action.
    pub close_message: Option<(&'a str, Message)>,
    pub body: Element<'a, Message>,
    pub footer: Option<Element<'a, Message>>,
    pub max_width: f32,
    pub max_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogParts {
    pub has_close: bool,
    pub has_footer: bool,
}

impl DialogParts {
    pub fn from_spec<Message>(spec: &DialogSpec<'_, Message>) -> Self {
        Self {
            has_close: spec.close_message.is_some(),
            has_footer: spec.footer.is_some(),
        }
    }
}

pub fn dialog<'a, Message: Clone + 'a>(spec: DialogSpec<'a, Message>) -> Element<'a, Message> {
    let parts = DialogParts::from_spec(&spec);
    let mut header = row![
        text(spec.title).size(FONT_TITLE).style(text_primary),
        Space::new().width(Length::Fill)
    ]
    .spacing(SPACE_SM)
    .align_y(Alignment::Center);
    if parts.has_close {
        if let Some((label, message)) = spec.close_message {
            header = header.push(app_button(
                text(label).size(FONT_BODY),
                Some(message),
                ButtonVariant::Ghost,
                ButtonSize::Default,
            ));
        }
    }
    // Fill allocates the remaining height to the body only, after measuring the
    // fixed header/footer. Long content cannot push actions outside the frame.
    let mut content = column![
        header,
        horizontal(),
        scrollable(spec.body)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(scrollbar_style)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::default().spacing(SPACE_XS)
            )),
    ]
    .spacing(SPACE_LG)
    .height(Length::Fill);
    if parts.has_footer {
        if let Some(footer) = spec.footer {
            content = content.push(horizontal()).push(footer);
        }
    }
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .max_width(spec.max_width)
        .max_height(spec.max_height)
        .padding(SPACE_LG as u16)
        .style(modal_dialog_style)
        .into()
}
