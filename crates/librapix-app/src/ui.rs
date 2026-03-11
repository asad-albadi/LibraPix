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

#[derive(Debug, Clone, Copy)]
pub struct ChipTone {
    pub background: Color,
    pub background_hover: Color,
    pub border: Color,
    pub text: Color,
    pub accent_text: Color,
}

const CHIP_PALETTE: [ChipTone; 12] = [
    ChipTone {
        background: Color {
            r: 0.204,
            g: 0.165,
            b: 0.235,
            a: 1.0,
        },
        background_hover: Color {
            r: 0.243,
            g: 0.200,
            b: 0.278,
            a: 1.0,
        },
        border: Color {
            r: 0.463,
            g: 0.373,
            b: 0.565,
            a: 1.0,
        },
        text: TEXT_PRIMARY,
        accent_text: Color {
            r: 0.839,
            g: 0.722,
            b: 0.941,
            a: 1.0,
        },
    },
    ChipTone {
        background: Color {
            r: 0.169,
            g: 0.216,
            b: 0.267,
            a: 1.0,
        },
        background_hover: Color {
            r: 0.204,
            g: 0.255,
            b: 0.314,
            a: 1.0,
        },
        border: Color {
            r: 0.345,
            g: 0.486,
            b: 0.612,
            a: 1.0,
        },
        text: TEXT_PRIMARY,
        accent_text: Color {
            r: 0.702,
            g: 0.843,
            b: 0.980,
            a: 1.0,
        },
    },
    ChipTone {
        background: Color {
            r: 0.149,
            g: 0.224,
            b: 0.188,
            a: 1.0,
        },
        background_hover: Color {
            r: 0.184,
            g: 0.267,
            b: 0.224,
            a: 1.0,
        },
        border: Color {
            r: 0.341,
            g: 0.549,
            b: 0.451,
            a: 1.0,
        },
        text: TEXT_PRIMARY,
        accent_text: Color {
            r: 0.718,
            g: 0.941,
            b: 0.812,
            a: 1.0,
        },
    },
    ChipTone {
        background: Color {
            r: 0.239,
            g: 0.204,
            b: 0.145,
            a: 1.0,
        },
        background_hover: Color {
            r: 0.286,
            g: 0.243,
            b: 0.180,
            a: 1.0,
        },
        border: Color {
            r: 0.620,
            g: 0.494,
            b: 0.302,
            a: 1.0,
        },
        text: TEXT_PRIMARY,
        accent_text: Color {
            r: 0.980,
            g: 0.839,
            b: 0.663,
            a: 1.0,
        },
    },
    ChipTone {
        background: Color {
            r: 0.239,
            g: 0.173,
            b: 0.149,
            a: 1.0,
        },
        background_hover: Color {
            r: 0.282,
            g: 0.204,
            b: 0.176,
            a: 1.0,
        },
        border: Color {
            r: 0.643,
            g: 0.427,
            b: 0.373,
            a: 1.0,
        },
        text: TEXT_PRIMARY,
        accent_text: Color {
            r: 0.980,
            g: 0.776,
            b: 0.722,
            a: 1.0,
        },
    },
    ChipTone {
        background: Color {
            r: 0.247,
            g: 0.161,
            b: 0.184,
            a: 1.0,
        },
        background_hover: Color {
            r: 0.294,
            g: 0.196,
            b: 0.220,
            a: 1.0,
        },
        border: Color {
            r: 0.620,
            g: 0.349,
            b: 0.467,
            a: 1.0,
        },
        text: TEXT_PRIMARY,
        accent_text: Color {
            r: 0.961,
            g: 0.698,
            b: 0.812,
            a: 1.0,
        },
    },
    ChipTone {
        background: Color {
            r: 0.173,
            g: 0.188,
            b: 0.255,
            a: 1.0,
        },
        background_hover: Color {
            r: 0.208,
            g: 0.227,
            b: 0.306,
            a: 1.0,
        },
        border: Color {
            r: 0.408,
            g: 0.447,
            b: 0.678,
            a: 1.0,
        },
        text: TEXT_PRIMARY,
        accent_text: Color {
            r: 0.733,
            g: 0.761,
            b: 0.961,
            a: 1.0,
        },
    },
    ChipTone {
        background: Color {
            r: 0.149,
            g: 0.243,
            b: 0.251,
            a: 1.0,
        },
        background_hover: Color {
            r: 0.184,
            g: 0.290,
            b: 0.298,
            a: 1.0,
        },
        border: Color {
            r: 0.325,
            g: 0.588,
            b: 0.608,
            a: 1.0,
        },
        text: TEXT_PRIMARY,
        accent_text: Color {
            r: 0.698,
            g: 0.929,
            b: 0.961,
            a: 1.0,
        },
    },
    ChipTone {
        background: Color {
            r: 0.208,
            g: 0.231,
            b: 0.157,
            a: 1.0,
        },
        background_hover: Color {
            r: 0.247,
            g: 0.275,
            b: 0.188,
            a: 1.0,
        },
        border: Color {
            r: 0.502,
            g: 0.604,
            b: 0.357,
            a: 1.0,
        },
        text: TEXT_PRIMARY,
        accent_text: Color {
            r: 0.851,
            g: 0.937,
            b: 0.714,
            a: 1.0,
        },
    },
    ChipTone {
        background: Color {
            r: 0.251,
            g: 0.224,
            b: 0.161,
            a: 1.0,
        },
        background_hover: Color {
            r: 0.302,
            g: 0.271,
            b: 0.196,
            a: 1.0,
        },
        border: Color {
            r: 0.659,
            g: 0.565,
            b: 0.357,
            a: 1.0,
        },
        text: TEXT_PRIMARY,
        accent_text: Color {
            r: 0.949,
            g: 0.902,
            b: 0.725,
            a: 1.0,
        },
    },
    ChipTone {
        background: Color {
            r: 0.161,
            g: 0.161,
            b: 0.231,
            a: 1.0,
        },
        background_hover: Color {
            r: 0.196,
            g: 0.196,
            b: 0.278,
            a: 1.0,
        },
        border: Color {
            r: 0.380,
            g: 0.380,
            b: 0.620,
            a: 1.0,
        },
        text: TEXT_PRIMARY,
        accent_text: Color {
            r: 0.776,
            g: 0.776,
            b: 0.980,
            a: 1.0,
        },
    },
    ChipTone {
        background: Color {
            r: 0.231,
            g: 0.161,
            b: 0.208,
            a: 1.0,
        },
        background_hover: Color {
            r: 0.282,
            g: 0.196,
            b: 0.255,
            a: 1.0,
        },
        border: Color {
            r: 0.596,
            g: 0.380,
            b: 0.502,
            a: 1.0,
        },
        text: TEXT_PRIMARY,
        accent_text: Color {
            r: 0.929,
            g: 0.757,
            b: 0.871,
            a: 1.0,
        },
    },
];

fn stable_color_index(value: &str, size: usize) -> usize {
    if size == 0 {
        return 0;
    }
    // FNV-1a hash: deterministic across sessions/processes.
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (hash as usize) % size
}

pub fn chip_tone_for_key(key: &str) -> ChipTone {
    let index = stable_color_index(key.trim().to_ascii_lowercase().as_str(), CHIP_PALETTE.len());
    CHIP_PALETTE[index]
}

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

pub fn preview_loading_block_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            r: 0.286,
            g: 0.286,
            b: 0.286,
            a: 1.0,
        })),
        border: iced::border::rounded(RADIUS_MD),
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

pub fn modal_backdrop_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.60,
        })),
        ..container::Style::default()
    }
}

pub fn modal_dialog_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_LAYER)),
        border: Border {
            color: ACCENT_SUBTLE,
            width: 1.0,
            radius: RADIUS_LG.into(),
        },
        ..container::Style::default()
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

pub fn managed_chip_action_style(
    tone: ChipTone,
    destructive: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let text_color = if destructive {
            WARNING_COLOR
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
