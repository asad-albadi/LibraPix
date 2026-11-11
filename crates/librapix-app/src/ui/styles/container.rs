use crate::ui::tokens::color::rgba;
use crate::ui::tokens::*;
use iced::widget::container;
use iced::{Background, Border, Radians, Theme, gradient};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Secondary,
    Success,
    Destructive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardVariant {
    Default,
    Interactive,
    Selected,
}

pub fn badge_style(theme: &Theme, variant: BadgeVariant) -> container::Style {
    let p = palette(theme);
    let (background, text, border) = match variant {
        BadgeVariant::Secondary => (p.bg_card, p.text_secondary, p.divider),
        BadgeVariant::Success => (p.bg_surface, p.success, p.success),
        BadgeVariant::Destructive => (p.bg_surface, p.destructive_text, p.destructive),
    };
    container::Style {
        background: Some(background.into()),
        text_color: Some(text),
        border: Border {
            color: border,
            width: 1.0,
            radius: RADIUS_PILL.into(),
        },
        ..container::Style::default()
    }
}

pub fn card_surface_style(theme: &Theme, variant: CardVariant) -> container::Style {
    let p = palette(theme);
    let selected = variant == CardVariant::Selected;
    container::Style {
        background: Some(
            if selected {
                p.bg_selected
            } else {
                p.bg_surface
            }
            .into(),
        ),
        text_color: Some(p.text_primary),
        border: Border {
            color: if selected { p.focus_ring } else { p.divider },
            width: if selected { 2.0 } else { 1.0 },
            radius: RADIUS_LG.into(),
        },
        ..container::Style::default()
    }
}

pub fn tooltip_style(theme: &Theme) -> container::Style {
    let mut style = card_surface_style(theme, CardVariant::Default);
    style.border.color = palette(theme).border_strong;
    style.shadow = elevation_low(theme);
    style
}

pub fn separator_style(theme: &Theme) -> iced::widget::rule::Style {
    iced::widget::rule::Style {
        color: palette(theme).divider,
        radius: 0.0.into(),
        fill_mode: iced::widget::rule::FillMode::Full,
        snap: true,
    }
}

// ── Container Styles ──

pub fn app_bg_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette(theme).bg_base)),
        ..container::Style::default()
    }
}

pub fn header_style(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(Background::Color(p.bg_layer)),
        border: Border {
            color: p.divider,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn sidebar_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette(theme).bg_layer)),
        ..container::Style::default()
    }
}

pub fn details_pane_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette(theme).bg_layer)),
        ..container::Style::default()
    }
}

/// Borderless raised insets for the preview and appearance section, not media cards.
pub fn raised_surface_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette(theme).bg_surface)),
        border: iced::border::rounded(RADIUS_LG),
        shadow: elevation_low(theme),
        ..container::Style::default()
    }
}

pub fn thumb_placeholder_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette(theme).bg_card)),
        border: iced::border::rounded(RADIUS_SM),
        ..container::Style::default()
    }
}

pub fn preview_loading_block_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette(theme).skeleton)),
        border: iced::border::rounded(RADIUS_MD),
        ..container::Style::default()
    }
}

pub fn scrubber_panel_style(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(Background::Color(p.bg_surface)),
        border: Border {
            color: p.divider,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        shadow: elevation_low(theme),
        ..container::Style::default()
    }
}

pub fn scrubber_chip_style(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(Background::Color(p.bg_card)),
        border: Border {
            color: p.accent,
            width: 1.0,
            radius: RADIUS_PILL.into(),
        },
        ..container::Style::default()
    }
}

pub fn modal_backdrop_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(rgba(0.0, 0.0, 0.0, 0.60))),
        ..container::Style::default()
    }
}

/// Translucent "glass" dialog surface: a soft vertical gradient at < 1.0 alpha so
/// the dimmed backdrop reads through as frosted tint, a faint light edge
/// highlight, and a pronounced drop shadow. (No true backdrop blur — iced has no
/// backdrop-filter primitive — but over the modal scrim this reads as glass.)
pub fn modal_dialog_style(theme: &Theme) -> container::Style {
    let (top, bottom, edge) = if theme.extended_palette().is_dark {
        (
            rgba(0.22, 0.23, 0.27, 0.86),
            rgba(0.11, 0.12, 0.14, 0.92),
            rgba(1.0, 1.0, 1.0, 0.10),
        )
    } else {
        (
            rgba(1.0, 1.0, 1.0, 0.86),
            rgba(0.95, 0.96, 0.98, 0.92),
            rgba(1.0, 1.0, 1.0, 0.70),
        )
    };
    let fill = gradient::Linear::new(Radians(std::f32::consts::PI))
        .add_stop(0.0, top)
        .add_stop(1.0, bottom);
    container::Style {
        background: Some(Background::Gradient(fill.into())),
        border: Border {
            color: edge,
            width: 1.0,
            radius: RADIUS_LG.into(),
        },
        shadow: elevation_high(theme),
        ..container::Style::default()
    }
}

/// Top-down dark scrim over a thumbnail so overlaid badges (kind, selection)
/// stay legible on bright images, fading to transparent by mid-height.
pub fn thumb_scrim_style(_theme: &Theme) -> container::Style {
    let fill = gradient::Linear::new(Radians(std::f32::consts::PI))
        .add_stop(0.0, rgba(0.0, 0.0, 0.0, 0.45))
        .add_stop(0.4, rgba(0.0, 0.0, 0.0, 0.0));
    container::Style {
        background: Some(Background::Gradient(fill.into())),
        ..container::Style::default()
    }
}

/// Accent pill behind the selection checkmark on a selected tile.
pub fn check_badge_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette(theme).accent)),
        border: iced::border::rounded(RADIUS_PILL),
        ..container::Style::default()
    }
}

pub fn managed_chip_style(tone: ChipTone) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(tone.background)),
        border: Border {
            color: tone.border,
            width: 1.0,
            radius: RADIUS_PILL.into(),
        },
        ..container::Style::default()
    }
}
