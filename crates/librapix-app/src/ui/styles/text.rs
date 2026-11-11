use crate::ui::tokens::palette;
use iced::{Theme, widget::text};

// ── Theme-aware text styles ──

pub fn text_primary(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).text_primary),
    }
}

pub fn text_secondary(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).text_secondary),
    }
}

pub fn text_tertiary(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).text_tertiary),
    }
}

pub fn text_accent(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).accent_text),
    }
}

pub fn text_warning(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).warning),
    }
}

pub fn text_error(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).destructive_text),
    }
}
