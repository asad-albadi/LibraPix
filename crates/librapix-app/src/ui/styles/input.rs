use crate::ui::tokens::{RADIUS_MD, palette};
use iced::widget::text_input;
use iced::{Border, Theme};

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
