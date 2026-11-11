use crate::ui::tokens::*;
use iced::widget::button;
use iced::{Background, Border, Color, Theme};

use super::container::{CardVariant, card_surface_style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Variants are staged before their view call sites (Tasks 4-7).
pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Link,
}

pub fn button_style(
    theme: &Theme,
    status: button::Status,
    variant: ButtonVariant,
) -> button::Style {
    let p = palette(theme);
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let (background, text_color) = if status == button::Status::Disabled {
        (p.bg_card, p.text_disabled)
    } else {
        match variant {
            ButtonVariant::Primary => (
                if hovered { p.accent_hover } else { p.accent },
                Color::WHITE,
            ),
            ButtonVariant::Destructive => (
                if hovered {
                    p.destructive_hover
                } else {
                    p.destructive
                },
                Color::WHITE,
            ),
            ButtonVariant::Secondary => {
                (if hovered { p.bg_hover } else { p.bg_card }, p.text_primary)
            }
            ButtonVariant::Outline | ButtonVariant::Ghost => (
                if hovered {
                    p.bg_hover
                } else {
                    Color::TRANSPARENT
                },
                p.text_primary,
            ),
            ButtonVariant::Link => (Color::TRANSPARENT, p.accent_text),
        }
    };
    button::Style {
        background: Some(background.into()),
        text_color,
        border: Border {
            color: if variant == ButtonVariant::Outline {
                p.border_strong
            } else {
                Color::TRANSPARENT
            },
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..button::Style::default()
    }
}

pub fn card_action_style(
    theme: &Theme,
    status: button::Status,
    variant: CardVariant,
) -> button::Style {
    let mut surface = card_surface_style(theme, variant);
    let p = palette(theme);
    if variant == CardVariant::Interactive
        && matches!(status, button::Status::Hovered | button::Status::Pressed)
    {
        surface.background = Some(p.bg_hover.into());
        surface.border.color = p.border_strong;
    }
    button::Style {
        background: surface.background,
        text_color: if status == button::Status::Disabled {
            p.text_disabled
        } else {
            p.text_primary
        },
        border: surface.border,
        ..button::Style::default()
    }
}

pub fn tab_style(theme: &Theme, status: button::Status, active: bool) -> button::Style {
    let p = palette(theme);
    let mut style = button_style(theme, status, ButtonVariant::Ghost);
    if active && status != button::Status::Disabled {
        style.background = Some(p.bg_selected.into());
        style.border.color = p.focus_ring;
    }
    style
}

// ── Button Styles ──

pub fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let (bg, text_color) = match status {
        button::Status::Active => (p.accent, Color::WHITE),
        button::Status::Hovered => (p.accent_hover, Color::WHITE),
        button::Status::Pressed => (p.accent_subtle, Color::WHITE),
        button::Status::Disabled => (p.bg_card, p.text_disabled),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: iced::border::rounded(RADIUS_MD),
        ..button::Style::default()
    }
}

pub fn subtle_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let (bg, text_color) = match status {
        button::Status::Active => (Color::TRANSPARENT, p.text_secondary),
        button::Status::Hovered => (p.bg_hover, p.text_primary),
        button::Status::Pressed => (p.bg_card, p.text_primary),
        button::Status::Disabled => (Color::TRANSPARENT, p.text_disabled),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: iced::border::rounded(RADIUS_MD),
        ..button::Style::default()
    }
}

pub fn action_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let (bg, text_color) = match status {
        button::Status::Active => (p.bg_card, p.text_primary),
        button::Status::Hovered => (p.bg_hover, p.text_primary),
        button::Status::Pressed => (p.bg_surface, p.text_primary),
        button::Status::Disabled => (p.bg_surface, p.text_disabled),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: iced::border::rounded(RADIUS_MD),
        ..button::Style::default()
    }
}

pub fn nav_button_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let (bg, text_color) = if active {
            (p.bg_surface, p.text_primary)
        } else {
            match status {
                button::Status::Hovered => (p.bg_hover, p.text_primary),
                _ => (Color::TRANSPARENT, p.text_secondary),
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: iced::border::rounded(RADIUS_MD),
            ..button::Style::default()
        }
    }
}

pub fn filter_chip_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let (bg, text_color) = if active {
            (p.accent, Color::WHITE)
        } else {
            match status {
                button::Status::Hovered => (p.bg_hover, p.text_primary),
                _ => (p.bg_surface, p.text_secondary),
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: iced::border::rounded(RADIUS_PILL),
            ..button::Style::default()
        }
    }
}

/// Segmented control button (Grid | Timeline toggle in the Library header).
pub fn segment_button_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        let (bg, text_color) = if active {
            (p.accent, Color::WHITE)
        } else {
            match status {
                button::Status::Hovered => (p.bg_hover, p.text_primary),
                _ => (Color::TRANSPARENT, p.text_secondary),
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: iced::border::rounded(RADIUS_MD),
            ..button::Style::default()
        }
    }
}

pub fn managed_chip_action_style(
    tone: ChipTone,
    destructive: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let text_color = if destructive {
            palette(theme).warning
        } else {
            tone.accent_text
        };
        let (bg, border_color) = match status {
            button::Status::Active => (Color::TRANSPARENT, Color::TRANSPARENT),
            button::Status::Hovered => (tone.background_hover, tone.border),
            button::Status::Pressed => (tone.background_hover, tone.border),
            button::Status::Disabled => (Color::TRANSPARENT, Color::TRANSPARENT),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                color: border_color,
                width: if matches!(status, button::Status::Active) {
                    0.0
                } else {
                    1.0
                },
                radius: RADIUS_PILL.into(),
            },
            ..button::Style::default()
        }
    }
}

pub fn card_button_style(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let p = palette(theme);
        // (background, border color, border width, shadow)
        let (bg, border_color, border_width, shadow) = if selected {
            (p.bg_selected, p.accent, 2.0, elevation_med(theme))
        } else {
            match status {
                button::Status::Hovered => (p.bg_hover, p.focus_ring, 1.0, elevation_med(theme)),
                _ => (p.bg_surface, Color::TRANSPARENT, 0.0, elevation_low(theme)),
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: p.text_primary,
            border: Border {
                color: border_color,
                width: border_width,
                radius: RADIUS_LG.into(),
            },
            shadow,
            ..button::Style::default()
        }
    }
}
