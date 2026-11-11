use crate::ui::tokens::{RADIUS_PILL, palette};
use iced::{Theme, widget::container};

pub fn activity_progress_style(theme: &Theme) -> iced::widget::progress_bar::Style {
    let p = palette(theme);
    iced::widget::progress_bar::Style {
        background: p.skeleton.into(),
        bar: p.focus_ring.into(),
        border: iced::border::rounded(RADIUS_PILL),
    }
}

pub fn loading_dot_style(theme: &Theme, active: bool) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(if active { p.focus_ring } else { p.skeleton }.into()),
        border: iced::border::rounded(RADIUS_PILL),
        ..container::Style::default()
    }
}
