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

#[allow(dead_code)]
pub fn text_disabled(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).text_disabled),
    }
}

pub fn text_accent(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).accent_text),
    }
}

pub fn text_success(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).success),
    }
}

pub fn text_warning(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).warning),
    }
}

pub fn text_error(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).destructive),
    }
}
