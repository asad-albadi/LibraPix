use iced::widget::{Container, Text, button, container, text, text_input};
use iced::{Background, Border, Color, Length, Theme};

// ── Color Palette (Fluent-inspired dark theme) ──

pub const BG_BASE: Color = Color {
    r: 0.110,
    g: 0.110,
    b: 0.110,
    a: 1.0,
};
pub const BG_LAYER: Color = Color {
    r: 0.137,
    g: 0.137,
    b: 0.137,
    a: 1.0,
};
pub const BG_SURFACE: Color = Color {
    r: 0.176,
    g: 0.176,
    b: 0.176,
    a: 1.0,
};
pub const BG_CARD: Color = Color {
    r: 0.220,
    g: 0.220,
    b: 0.220,
    a: 1.0,
};
pub const BG_HOVER: Color = Color {
    r: 0.259,
    g: 0.259,
    b: 0.259,
    a: 1.0,
};
pub const BG_SELECTED: Color = Color {
    r: 0.055,
    g: 0.290,
    b: 0.478,
    a: 1.0,
};

pub const ACCENT: Color = Color {
    r: 0.0,
    g: 0.471,
    b: 0.831,
    a: 1.0,
};
pub const ACCENT_HOVER: Color = Color {
    r: 0.102,
    g: 0.533,
    b: 0.910,
    a: 1.0,
};
pub const ACCENT_SUBTLE: Color = Color {
    r: 0.0,
    g: 0.278,
    b: 0.502,
    a: 1.0,
};

pub const TEXT_PRIMARY: Color = Color {
    r: 0.961,
    g: 0.961,
    b: 0.961,
    a: 1.0,
};
pub const TEXT_SECONDARY: Color = Color {
    r: 0.620,
    g: 0.620,
    b: 0.620,
    a: 1.0,
};
pub const TEXT_TERTIARY: Color = Color {
    r: 0.431,
    g: 0.431,
    b: 0.431,
    a: 1.0,
};
pub const TEXT_DISABLED: Color = Color {
    r: 0.306,
    g: 0.306,
    b: 0.306,
    a: 1.0,
};

pub const DIVIDER_COLOR: Color = Color {
    r: 0.200,
    g: 0.200,
    b: 0.200,
    a: 1.0,
};
pub const SUCCESS_COLOR: Color = Color {
    r: 0.424,
    g: 0.796,
    b: 0.373,
    a: 1.0,
};
pub const WARNING_COLOR: Color = Color {
    r: 1.0,
    g: 0.702,
    b: 0.278,
    a: 1.0,
};

// ── Spacing ──

pub const SPACE_2XS: u32 = 2;
pub const SPACE_XS: u32 = 4;
pub const SPACE_SM: u32 = 8;
pub const SPACE_MD: u32 = 12;
pub const SPACE_LG: u32 = 16;
pub const SPACE_XL: u32 = 24;
pub const SPACE_2XL: u32 = 32;

// ── Typography ──

pub const FONT_DISPLAY: u32 = 28;
pub const FONT_TITLE: u32 = 20;
pub const FONT_SUBTITLE: u32 = 16;
pub const FONT_SECTION: u32 = 11;
pub const FONT_BODY: u32 = 13;
pub const FONT_CAPTION: u32 = 11;

// ── Layout ──

pub const SIDEBAR_WIDTH: f32 = 240.0;
pub const DETAILS_WIDTH: f32 = 300.0;
pub const HEADER_HEIGHT: f32 = 52.0;
pub const GALLERY_GAP: u32 = 4;

pub const RADIUS_SM: f32 = 4.0;
pub const RADIUS_MD: f32 = 6.0;
pub const RADIUS_LG: f32 = 8.0;
pub const RADIUS_PILL: f32 = 16.0;

// ── Container Styles ──

pub fn app_bg_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_BASE)),
        ..container::Style::default()
    }
}

pub fn header_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_LAYER)),
        border: Border {
            color: DIVIDER_COLOR,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn sidebar_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_LAYER)),
        ..container::Style::default()
    }
}

pub fn details_pane_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_LAYER)),
        ..container::Style::default()
    }
}

pub fn card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_SURFACE)),
        border: iced::border::rounded(RADIUS_LG),
        ..container::Style::default()
    }
}

pub fn thumb_placeholder_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_CARD)),
        border: iced::border::rounded(RADIUS_SM),
        ..container::Style::default()
    }
}

pub fn empty_state_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_SURFACE)),
        border: iced::border::rounded(RADIUS_LG),
        ..container::Style::default()
    }
}

pub fn scrubber_panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_SURFACE)),
        border: Border {
            color: DIVIDER_COLOR,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..container::Style::default()
    }
}

pub fn scrubber_chip_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_CARD)),
        border: Border {
            color: ACCENT,
            width: 1.0,
            radius: RADIUS_PILL.into(),
        },
        ..container::Style::default()
    }
}

pub fn announcement_panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_SURFACE)),
        border: Border {
            color: ACCENT_SUBTLE,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..container::Style::default()
    }
}

pub fn media_kind_badge_style(is_video: bool) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let (background, border) = if is_video {
            (ACCENT_SUBTLE, ACCENT)
        } else {
            (BG_LAYER, TEXT_TERTIARY)
        };
        container::Style {
            background: Some(Background::Color(background)),
            border: Border {
                color: border,
                width: 1.0,
                radius: RADIUS_PILL.into(),
            },
            ..container::Style::default()
        }
    }
}

pub fn divider_line_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(DIVIDER_COLOR)),
        ..container::Style::default()
    }
}

// ── Button Styles ──

pub fn primary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Active => (ACCENT, Color::WHITE),
        button::Status::Hovered => (ACCENT_HOVER, Color::WHITE),
        button::Status::Pressed => (ACCENT_SUBTLE, Color::WHITE),
        button::Status::Disabled => (BG_CARD, TEXT_DISABLED),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: iced::border::rounded(RADIUS_MD),
        ..button::Style::default()
    }
}

pub fn subtle_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Active => (Color::TRANSPARENT, TEXT_SECONDARY),
        button::Status::Hovered => (BG_HOVER, TEXT_PRIMARY),
        button::Status::Pressed => (BG_CARD, TEXT_PRIMARY),
        button::Status::Disabled => (Color::TRANSPARENT, TEXT_DISABLED),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: iced::border::rounded(RADIUS_MD),
        ..button::Style::default()
    }
}

pub fn action_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Active => (BG_CARD, TEXT_PRIMARY),
        button::Status::Hovered => (BG_HOVER, TEXT_PRIMARY),
        button::Status::Pressed => (BG_SURFACE, TEXT_PRIMARY),
        button::Status::Disabled => (BG_SURFACE, TEXT_DISABLED),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: iced::border::rounded(RADIUS_MD),
        ..button::Style::default()
    }
}

pub fn nav_button_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (bg, text_color) = if active {
            (BG_SURFACE, TEXT_PRIMARY)
        } else {
            match status {
                button::Status::Hovered => (BG_HOVER, TEXT_PRIMARY),
                _ => (Color::TRANSPARENT, TEXT_SECONDARY),
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
    move |_theme, status| {
        let (bg, text_color) = if active {
            (ACCENT, Color::WHITE)
        } else {
            match status {
                button::Status::Hovered => (BG_HOVER, TEXT_PRIMARY),
                _ => (BG_SURFACE, TEXT_SECONDARY),
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

pub fn card_button_style(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (bg, border_color, border_width) = if selected {
            (BG_SELECTED, ACCENT, 2.0)
        } else {
            match status {
                button::Status::Hovered => (BG_HOVER, Color::TRANSPARENT, 0.0),
                _ => (BG_SURFACE, Color::TRANSPARENT, 0.0),
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: TEXT_PRIMARY,
            border: Border {
                color: border_color,
                width: border_width,
                radius: RADIUS_LG.into(),
            },
            ..button::Style::default()
        }
    }
}

// ── Text Input Styles ──

pub fn search_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Active => BG_CARD,
        text_input::Status::Hovered => BG_HOVER,
        text_input::Status::Focused { .. } => ACCENT,
        text_input::Status::Disabled => BG_SURFACE,
    };
    text_input::Style {
        background: Background::Color(BG_SURFACE),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        icon: TEXT_TERTIARY,
        placeholder: TEXT_TERTIARY,
        value: TEXT_PRIMARY,
        selection: ACCENT_SUBTLE,
    }
}

pub fn field_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Active => BG_CARD,
        text_input::Status::Hovered => BG_HOVER,
        text_input::Status::Focused { .. } => ACCENT,
        text_input::Status::Disabled => BG_SURFACE,
    };
    text_input::Style {
        background: Background::Color(BG_SURFACE),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        icon: TEXT_TERTIARY,
        placeholder: TEXT_TERTIARY,
        value: TEXT_PRIMARY,
        selection: ACCENT_SUBTLE,
    }
}

// ── Layout Helpers ──

pub fn section_heading(label: &str) -> Text<'_> {
    text(label).size(FONT_SECTION).color(TEXT_TERTIARY)
}

pub fn h_divider<'a, Message: 'a>() -> Container<'a, Message> {
    container(text(""))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(divider_line_style)
}
