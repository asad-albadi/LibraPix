use crate::ui::tokens::{RADIUS_MD, palette};
use iced::widget::text_input;
use iced::{Background, Border, Theme};

pub fn input_style(theme: &Theme, status: text_input::Status, invalid: bool) -> text_input::Style {
    let p = palette(theme);
    let disabled = status == text_input::Status::Disabled;
    let border = if disabled {
        p.divider
    } else if invalid {
        p.destructive
    } else {
        match status {
            text_input::Status::Focused { .. } => p.focus_ring,
            text_input::Status::Hovered => p.border_strong,
            _ => p.divider,
        }
    };
    text_input::Style {
        background: if disabled { p.bg_card } else { p.bg_surface }.into(),
        border: Border {
            color: border,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        icon: p.text_tertiary,
        placeholder: p.text_tertiary,
        value: if disabled {
            p.text_disabled
        } else {
            p.text_primary
        },
        selection: p.accent_subtle,
    }
}

// ── Text Input Styles ──

pub fn search_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let p = palette(theme);
    let (border_color, border_width) = match status {
        text_input::Status::Active => (p.bg_card, 1.0),
        text_input::Status::Hovered => (p.bg_hover, 1.0),
        text_input::Status::Focused { .. } => (p.focus_ring, 2.0),
        text_input::Status::Disabled => (p.bg_surface, 1.0),
    };
    text_input::Style {
        background: Background::Color(p.bg_surface),
        border: Border {
            color: border_color,
            width: border_width,
            radius: RADIUS_MD.into(),
        },
        icon: p.text_tertiary,
        placeholder: p.text_tertiary,
        value: p.text_primary,
        selection: p.accent_subtle,
    }
}

pub fn field_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let p = palette(theme);
    let (border_color, border_width) = match status {
        text_input::Status::Active => (p.bg_card, 1.0),
        text_input::Status::Hovered => (p.bg_hover, 1.0),
        text_input::Status::Focused { .. } => (p.focus_ring, 2.0),
        text_input::Status::Disabled => (p.bg_surface, 1.0),
    };
    text_input::Style {
        background: Background::Color(p.bg_surface),
        border: Border {
            color: border_color,
            width: border_width,
            radius: RADIUS_MD.into(),
        },
        icon: p.text_tertiary,
        placeholder: p.text_tertiary,
        value: p.text_primary,
        selection: p.accent_subtle,
    }
}
