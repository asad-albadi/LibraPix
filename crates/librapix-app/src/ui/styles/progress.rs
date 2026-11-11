use crate::ui::tokens::{RADIUS_PILL, palette};
use iced::{Theme, widget::container};

pub fn loading_dot_style(theme: &Theme, active: bool) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(if active { p.focus_ring } else { p.skeleton }.into()),
        border: iced::border::rounded(RADIUS_PILL),
        ..container::Style::default()
    }
}
