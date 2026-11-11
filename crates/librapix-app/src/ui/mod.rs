//! App-local design tokens, widget styles, and compatibility layout helpers.

// Primitives are intentionally staged before view migration (Tasks 4-7).
#[allow(dead_code)]
pub mod components;
pub mod patterns;
pub mod styles;
pub mod tokens;

pub use styles::*;
pub use tokens::*;

use iced::widget::{Text, text};

// ── Layout Helpers ──

pub fn section_heading(label: &str) -> Text<'_> {
    text(label).size(FONT_SECTION).style(text_tertiary)
}
