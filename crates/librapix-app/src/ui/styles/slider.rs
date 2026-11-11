//! Timeline rail styling; the native slider retains all interaction semantics.
use crate::ui::tokens::palette;
use iced::{Border, Theme, widget::slider};

pub fn timeline_slider_style(theme: &Theme, status: slider::Status) -> slider::Style {
    let p = palette(theme);
    let interacting = !matches!(status, slider::Status::Active);
    slider::Style {
        rail: slider::Rail {
            backgrounds: (p.border_strong.into(), p.border_strong.into()),
            width: 2.0,
            border: Border::default().rounded(1),
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 5.0 },
            background: if interacting {
                p.accent_hover
            } else {
                p.accent
            }
            .into(),
            border_width: if interacting { 2.0 } else { 1.0 },
            border_color: if interacting {
                p.focus_ring
            } else {
                p.bg_surface
            },
        },
    }
}
