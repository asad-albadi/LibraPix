use crate::ui::{
    components::{button::ButtonVariant, icon_button::icon_button, tooltip::app_tooltip},
    styles::{container::details_pane_style, scrollable::scrollbar_style, text::text_secondary},
    tokens::*,
};
use iced::{
    Alignment, Element, Length,
    widget::{Container, Space, button, column, container, row, scrollable, text},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailsPaneMode {
    Expanded,
    Edge,
}

impl DetailsPaneMode {
    pub fn from_collapsed(collapsed: bool) -> Self {
        if collapsed {
            Self::Edge
        } else {
            Self::Expanded
        }
    }
}

pub fn details_pane<'a, Message: Clone + 'a>(
    mode: DetailsPaneMode,
    title: &'a str,
    toggle_label: &'a str,
    toggle_message: Message,
    expanded_content: impl FnOnce() -> Element<'a, Message>,
) -> Container<'a, Message> {
    let (content, width, padding): (Element<'a, Message>, _, _) = match mode {
        DetailsPaneMode::Expanded => (
            column![
                row![
                    text(title).size(FONT_BODY).style(text_secondary),
                    Space::new().width(Length::Fill),
                    icon_button(
                        text("\u{203A}").size(ICON_LG).into(),
                        toggle_label,
                        Some(toggle_message),
                        ButtonVariant::Ghost,
                    ),
                ]
                .align_y(Alignment::Center),
                scrollable(expanded_content())
                    .style(scrollbar_style)
                    .direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::default().spacing(SPACE_XS),
                    ))
                    .height(Length::Fill),
            ]
            .spacing(SPACE_SM)
            .into(),
            DETAILS_WIDTH,
            SPACE_LG,
        ),
        DetailsPaneMode::Edge => (
            app_tooltip(
                button(container(super::rotated_label::rotated_label(title)).center(Length::Fill))
                    .on_press(toggle_message)
                    .style(|theme, status| {
                        crate::ui::styles::button::button_style(theme, status, ButtonVariant::Ghost)
                    })
                    .height(Length::Fill)
                    .width(Length::Fill),
                toggle_label,
            )
            .into(),
            DETAILS_EDGE_WIDTH,
            SPACE_XS,
        ),
    };
    container(content)
        .width(width)
        .height(Length::Fill)
        .padding(padding as u16)
        .style(details_pane_style)
}
