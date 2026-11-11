use crate::ui::tokens::{RADIUS_MD, elevation_low, palette};
use iced::{
    Border, Theme,
    widget::{overlay::menu, pick_list},
};

pub fn select_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let p = palette(theme);
    let edge = match status {
        pick_list::Status::Active => p.divider,
        pick_list::Status::Hovered => p.border_strong,
        pick_list::Status::Opened { .. } => p.focus_ring,
    };
    pick_list::Style {
        text_color: p.text_primary,
        placeholder_color: p.text_tertiary,
        handle_color: p.text_secondary,
        background: p.bg_surface.into(),
        border: Border {
            color: edge,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
    }
}

pub fn select_menu_style(theme: &Theme) -> menu::Style {
    let p = palette(theme);
    menu::Style {
        background: p.bg_surface.into(),
        border: Border {
            color: p.border_strong,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        text_color: p.text_primary,
        selected_text_color: p.text_primary,
        selected_background: p.bg_selected.into(),
        shadow: elevation_low(theme),
    }
}
