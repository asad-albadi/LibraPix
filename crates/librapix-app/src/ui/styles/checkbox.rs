use crate::ui::tokens::{RADIUS_SM, palette};
use iced::{Border, Color, Theme, widget::checkbox};

pub fn checkbox_style(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let p = palette(theme);
    let (checked, hovered, disabled) = match status {
        checkbox::Status::Active { is_checked } => (is_checked, false, false),
        checkbox::Status::Hovered { is_checked } => (is_checked, true, false),
        checkbox::Status::Disabled { is_checked } => (is_checked, false, true),
    };
    checkbox::Style {
        background: if disabled {
            p.bg_card
        } else if checked {
            p.accent
        } else if hovered {
            p.bg_hover
        } else {
            p.bg_surface
        }
        .into(),
        icon_color: if disabled {
            p.text_disabled
        } else {
            Color::WHITE
        },
        border: Border {
            color: if hovered {
                p.focus_ring
            } else if checked && !disabled {
                p.accent
            } else {
                p.border_strong
            },
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
        text_color: Some(if disabled {
            p.text_disabled
        } else {
            p.text_primary
        }),
    }
}
