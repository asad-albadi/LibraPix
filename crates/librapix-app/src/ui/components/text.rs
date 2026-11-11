//! Reusable section typography with caller-owned localized content.
use crate::ui::{styles::text::text_tertiary, tokens::FONT_SECTION};
use iced::widget::{Text, text};

pub fn section_heading(label: &str) -> Text<'_> {
    text(label).size(FONT_SECTION).style(text_tertiary)
}
