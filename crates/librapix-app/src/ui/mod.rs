//! App-local design tokens, widget styles, and compatibility layout helpers.

pub mod styles;
pub mod tokens;

pub use styles::*;
pub use tokens::*;

use iced::Length;
use iced::widget::{Container, Text, container as view_container, text};

// ── Layout Helpers ──

pub fn section_heading(label: &str) -> Text<'_> {
    text(label).size(FONT_SECTION).style(text_tertiary)
}

pub fn h_divider<'a, Message: 'a>() -> Container<'a, Message> {
    view_container(text(""))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(divider_line_style)
}

/// 1px vertical hairline, used to separate the sidebar / content / details panes.
pub fn v_divider<'a, Message: 'a>() -> Container<'a, Message> {
    view_container(text(""))
        .width(Length::Fixed(1.0))
        .height(Length::Fill)
        .style(divider_line_style)
}
