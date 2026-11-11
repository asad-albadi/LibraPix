//! Truthful status feedback: callers provide localized text and animation phase.

use crate::ui::{
    components::{
        badge::{BadgeVariant, badge},
        loading::working_row,
    },
    tokens::*,
};
use iced::{Element, Length, widget::container};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusTone {
    Neutral,
    Working,
    Success,
    Warning,
    Error,
}

/// No numeric progress input: Working is always indeterminate, driven by the
/// caller's existing timer rather than a subscription or task owned by the UI.
pub fn status_row<'a, Message: 'a>(
    stage: &'a str,
    tone: StatusTone,
    phase: usize,
) -> Element<'a, Message> {
    let content: Element<'a, Message> = match tone {
        StatusTone::Working => working_row(stage, phase).into(),
        StatusTone::Neutral => badge(stage, BadgeVariant::Secondary).into(),
        StatusTone::Success => badge(stage, BadgeVariant::Success).into(),
        StatusTone::Warning => badge(stage, BadgeVariant::Warning).into(),
        StatusTone::Error => badge(stage, BadgeVariant::Destructive).into(),
    };
    container(content)
        .width(Length::Fill)
        .center_y(CONTROL_HEIGHT_MD)
        .into()
}
