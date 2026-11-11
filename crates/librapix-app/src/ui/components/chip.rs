//! Deterministic managed tag/rule chip, independent of product state.
use crate::ui::{
    styles::{button::managed_chip_action_style, container::managed_chip_style},
    tokens::*,
};
use iced::{
    Element, Length,
    widget::{Space, button, column, container, row, text},
};

pub fn managed_chip<Message: Clone + 'static>(
    label: impl Into<String>,
    tone_key: &str,
    status_label: Option<String>,
    toggle: Option<(String, Message)>,
    edit: Option<(String, Message)>,
    remove: Option<Message>,
    dark: bool,
) -> Element<'static, Message> {
    let label = label.into();
    let tone = chip_tone_for_key(tone_key, dark);
    let mut controls = row![].spacing(SPACE_2XS).align_y(iced::Alignment::Center);
    if let Some((toggle_label, msg)) = toggle {
        controls = controls.push(
            button(text(toggle_label).size(FONT_CAPTION))
                .on_press(msg)
                .style(managed_chip_action_style(tone, false))
                .padding([SPACE_2XS as u16, SPACE_XS as u16]),
        );
    }
    if let Some((edit_label, msg)) = edit {
        controls = controls.push(
            button(text(edit_label).size(FONT_CAPTION))
                .on_press(msg)
                .style(managed_chip_action_style(tone, false))
                .padding([SPACE_2XS as u16, SPACE_XS as u16]),
        );
    }
    if let Some(msg) = remove {
        controls = controls.push(
            button(text("x").size(FONT_CAPTION))
                .on_press(msg)
                .style(managed_chip_action_style(tone, true))
                .padding([SPACE_2XS as u16, SPACE_XS as u16]),
        );
    }
    let status: Element<'static, Message> = if let Some(value) = status_label {
        text(value)
            .size(FONT_CAPTION)
            .color(tone.accent_text)
            .into()
    } else {
        Space::new().width(Length::Shrink).into()
    };
    container(
        row![
            column![text(label).size(FONT_BODY).color(tone.text), status,].spacing(SPACE_2XS),
            Space::new().width(Length::Fill),
            controls,
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding([SPACE_XS as u16, SPACE_SM as u16])
    .style(managed_chip_style(tone))
    .into()
}
