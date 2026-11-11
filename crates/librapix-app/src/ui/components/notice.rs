//! Bounded, overlay-friendly shell feedback with caller-localized labels.

use super::{
    badge::{BadgeVariant, badge},
    button::{ButtonSize, ButtonVariant, app_button},
    card::{CardVariant, card},
};
use crate::ui::{styles::text::text_primary, tokens::*};
use iced::{
    Alignment, Length,
    widget::{Container, column, row, text},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeTone {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellNotice {
    pub tone: NoticeTone,
    pub summary: String,
}

pub fn notice<'a, Message: Clone + 'a>(
    notice: &'a ShellNotice,
    tone_label: &'a str,
    dismiss: Option<(&'a str, Message)>,
) -> Container<'a, Message> {
    let variant = match notice.tone {
        NoticeTone::Info => BadgeVariant::Default,
        NoticeTone::Success => BadgeVariant::Success,
        NoticeTone::Warning => BadgeVariant::Warning,
        NoticeTone::Error => BadgeVariant::Destructive,
    };
    let mut header = row![badge(tone_label, variant)]
        .spacing(SPACE_SM)
        .align_y(Alignment::Center);
    if let Some((label, message)) = dismiss {
        header = header.push(app_button(
            text(label).size(FONT_CAPTION),
            Some(message),
            ButtonVariant::Ghost,
            ButtonSize::Small,
        ));
    }
    card(
        column![
            header,
            text(&notice.summary).size(FONT_BODY).style(text_primary)
        ]
        .spacing(SPACE_SM),
        CardVariant::Default,
    )
    .padding(SPACE_MD as u16)
    .width(Length::Shrink)
    .max_width(360)
}
