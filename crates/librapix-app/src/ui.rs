use iced::widget::{Container, Text, button, container, text, text_input};
use iced::{Background, Border, Color, Length, Radians, Shadow, Theme, Vector, gradient};

// ── Semantic palette ──
//
// A single `Palette` describes every chrome/text/accent surface. Two static
// instances (`DARK`, `LIGHT`) hold the concrete values; `palette()` selects one
// from the active theme. Style closures and the theme-aware text helpers read
// `palette(theme).*` so the whole UI follows Dark / Light / System automatically.

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg_base: Color,
    pub bg_layer: Color,
    pub bg_surface: Color,
    pub bg_card: Color,
    pub bg_hover: Color,
    pub bg_selected: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_subtle: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_disabled: Color,
    pub divider: Color,
    pub success: Color,
    pub warning: Color,
    pub focus_ring: Color,
    pub skeleton: Color,
}

const fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b, a: 1.0 }
}

const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color { r, g, b, a }
}

/// Near-black, content-first dark surface (the original Fluent values).
pub static DARK: Palette = Palette {
    bg_base: rgb(0.110, 0.110, 0.110),
    bg_layer: rgb(0.137, 0.137, 0.137),
    bg_surface: rgb(0.176, 0.176, 0.176),
    bg_card: rgb(0.220, 0.220, 0.220),
    bg_hover: rgb(0.259, 0.259, 0.259),
    bg_selected: rgb(0.055, 0.290, 0.478),
    accent: rgb(0.0, 0.471, 0.831),
    accent_hover: rgb(0.102, 0.533, 0.910),
    accent_subtle: rgb(0.0, 0.278, 0.502),
    text_primary: rgb(0.961, 0.961, 0.961),
    text_secondary: rgb(0.620, 0.620, 0.620),
    text_tertiary: rgb(0.431, 0.431, 0.431),
    text_disabled: rgb(0.306, 0.306, 0.306),
    divider: rgb(0.200, 0.200, 0.200),
    success: rgb(0.424, 0.796, 0.373),
    warning: rgb(1.0, 0.702, 0.278),
    focus_ring: rgba(0.0, 0.471, 0.831, 0.55),
    skeleton: rgb(0.286, 0.286, 0.286),
};

/// Soft off-white base with white surfaces; deeper accent + text for WCAG AA.
pub static LIGHT: Palette = Palette {
    bg_base: rgb(0.969, 0.969, 0.973),
    bg_layer: rgb(0.957, 0.957, 0.961),
    bg_surface: rgb(1.0, 1.0, 1.0),
    bg_card: rgb(0.929, 0.929, 0.937),
    bg_hover: rgb(0.898, 0.898, 0.910),
    bg_selected: rgb(0.847, 0.918, 0.984),
    accent: rgb(0.0, 0.357, 0.682),
    accent_hover: rgb(0.0, 0.408, 0.769),
    accent_subtle: rgb(0.788, 0.875, 0.961),
    text_primary: rgb(0.110, 0.110, 0.118),
    text_secondary: rgb(0.345, 0.345, 0.365),
    text_tertiary: rgb(0.420, 0.420, 0.443),
    text_disabled: rgb(0.620, 0.620, 0.635),
    divider: rgb(0.882, 0.882, 0.894),
    success: rgb(0.149, 0.553, 0.231),
    warning: rgb(0.706, 0.439, 0.0),
    focus_ring: rgba(0.0, 0.357, 0.682, 0.45),
    skeleton: rgb(0.910, 0.910, 0.918),
};

/// Resolve the palette for the active theme (System-aware via `theme()`).
pub fn palette(theme: &Theme) -> &'static Palette {
    if theme.extended_palette().is_dark {
        &DARK
    } else {
        &LIGHT
    }
}

// ── Elevation (soft shadows) ──
//
// Dark surfaces use a near-black shadow; light surfaces a softer cool-gray at
// reduced alpha. Helpers feed the `shadow` field of container/button styles to
// lift cards, controls, and dialogs off the base without changing layout.

fn shadow_color(theme: &Theme, alpha: f32) -> Color {
    if theme.extended_palette().is_dark {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: alpha,
        }
    } else {
        // Cool-gray, noticeably lighter so white surfaces don't get a harsh halo.
        Color {
            r: 0.20,
            g: 0.22,
            b: 0.28,
            a: alpha * 0.55,
        }
    }
}

/// Resting card lift.
pub fn elevation_low(theme: &Theme) -> Shadow {
    Shadow {
        color: shadow_color(theme, 0.30),
        offset: Vector::new(0.0, 1.0),
        blur_radius: 4.0,
    }
}

/// Hovered card / raised control.
pub fn elevation_med(theme: &Theme) -> Shadow {
    Shadow {
        color: shadow_color(theme, 0.40),
        offset: Vector::new(0.0, 3.0),
        blur_radius: 12.0,
    }
}

/// Modal dialogs.
pub fn elevation_high(theme: &Theme) -> Shadow {
    Shadow {
        color: shadow_color(theme, 0.48),
        offset: Vector::new(0.0, 10.0),
        blur_radius: 30.0,
    }
}

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

// ── Icon sizes ──

pub const ICON_XS: f32 = 14.0;
pub const ICON_SM: f32 = 16.0;
pub const ICON_MD: f32 = 18.0;
pub const ICON_LG: f32 = 20.0;
pub const ICON_XL: f32 = 40.0;

// ── Layout ──

pub const SIDEBAR_WIDTH: f32 = 240.0;
pub const DETAILS_WIDTH: f32 = 300.0;
pub const HEADER_HEIGHT: f32 = 52.0;
pub const GALLERY_GAP: u32 = 4;

pub const RADIUS_SM: f32 = 4.0;
pub const RADIUS_MD: f32 = 6.0;
pub const RADIUS_LG: f32 = 8.0;
pub const RADIUS_PILL: f32 = 16.0;

// ── Theme-aware text styles ──

pub fn text_primary(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).text_primary),
    }
}

pub fn text_secondary(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).text_secondary),
    }
}

pub fn text_tertiary(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).text_tertiary),
    }
}

#[allow(dead_code)]
pub fn text_disabled(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).text_disabled),
    }
}

pub fn text_accent(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).accent),
    }
}

pub fn text_success(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).success),
    }
}

pub fn text_warning(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).warning),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChipTone {
    pub background: Color,
    pub background_hover: Color,
    pub border: Color,
    pub text: Color,
    pub accent_text: Color,
}

const CHIP_PALETTE_DARK: [ChipTone; 12] = [
    ChipTone {
        background: rgb(0.204, 0.165, 0.235),
        background_hover: rgb(0.243, 0.200, 0.278),
        border: rgb(0.463, 0.373, 0.565),
        text: DARK.text_primary,
        accent_text: rgb(0.839, 0.722, 0.941),
    },
    ChipTone {
        background: rgb(0.169, 0.216, 0.267),
        background_hover: rgb(0.204, 0.255, 0.314),
        border: rgb(0.345, 0.486, 0.612),
        text: DARK.text_primary,
        accent_text: rgb(0.702, 0.843, 0.980),
    },
    ChipTone {
        background: rgb(0.149, 0.224, 0.188),
        background_hover: rgb(0.184, 0.267, 0.224),
        border: rgb(0.341, 0.549, 0.451),
        text: DARK.text_primary,
        accent_text: rgb(0.718, 0.941, 0.812),
    },
    ChipTone {
        background: rgb(0.239, 0.204, 0.145),
        background_hover: rgb(0.286, 0.243, 0.180),
        border: rgb(0.620, 0.494, 0.302),
        text: DARK.text_primary,
        accent_text: rgb(0.980, 0.839, 0.663),
    },
    ChipTone {
        background: rgb(0.239, 0.173, 0.149),
        background_hover: rgb(0.282, 0.204, 0.176),
        border: rgb(0.643, 0.427, 0.373),
        text: DARK.text_primary,
        accent_text: rgb(0.980, 0.776, 0.722),
    },
    ChipTone {
        background: rgb(0.247, 0.161, 0.184),
        background_hover: rgb(0.294, 0.196, 0.220),
        border: rgb(0.620, 0.349, 0.467),
        text: DARK.text_primary,
        accent_text: rgb(0.961, 0.698, 0.812),
    },
    ChipTone {
        background: rgb(0.173, 0.188, 0.255),
        background_hover: rgb(0.208, 0.227, 0.306),
        border: rgb(0.408, 0.447, 0.678),
        text: DARK.text_primary,
        accent_text: rgb(0.733, 0.761, 0.961),
    },
    ChipTone {
        background: rgb(0.149, 0.243, 0.251),
        background_hover: rgb(0.184, 0.290, 0.298),
        border: rgb(0.325, 0.588, 0.608),
        text: DARK.text_primary,
        accent_text: rgb(0.698, 0.929, 0.961),
    },
    ChipTone {
        background: rgb(0.208, 0.231, 0.157),
        background_hover: rgb(0.247, 0.275, 0.188),
        border: rgb(0.502, 0.604, 0.357),
        text: DARK.text_primary,
        accent_text: rgb(0.851, 0.937, 0.714),
    },
    ChipTone {
        background: rgb(0.251, 0.224, 0.161),
        background_hover: rgb(0.302, 0.271, 0.196),
        border: rgb(0.659, 0.565, 0.357),
        text: DARK.text_primary,
        accent_text: rgb(0.949, 0.902, 0.725),
    },
    ChipTone {
        background: rgb(0.161, 0.161, 0.231),
        background_hover: rgb(0.196, 0.196, 0.278),
        border: rgb(0.380, 0.380, 0.620),
        text: DARK.text_primary,
        accent_text: rgb(0.776, 0.776, 0.980),
    },
    ChipTone {
        background: rgb(0.231, 0.161, 0.208),
        background_hover: rgb(0.282, 0.196, 0.255),
        border: rgb(0.596, 0.380, 0.502),
        text: DARK.text_primary,
        accent_text: rgb(0.929, 0.757, 0.871),
    },
];

// Light-theme chips: soft tinted fills, white-ish hover, saturated border, and a
// darkened accent text that meets AA over the pale fill.
const CHIP_PALETTE_LIGHT: [ChipTone; 12] = [
    ChipTone {
        background: rgb(0.929, 0.910, 0.965),
        background_hover: rgb(0.886, 0.851, 0.945),
        border: rgb(0.612, 0.518, 0.722),
        text: LIGHT.text_primary,
        accent_text: rgb(0.357, 0.235, 0.510),
    },
    ChipTone {
        background: rgb(0.902, 0.937, 0.973),
        background_hover: rgb(0.851, 0.910, 0.965),
        border: rgb(0.435, 0.580, 0.710),
        text: LIGHT.text_primary,
        accent_text: rgb(0.149, 0.310, 0.498),
    },
    ChipTone {
        background: rgb(0.894, 0.957, 0.918),
        background_hover: rgb(0.835, 0.937, 0.875),
        border: rgb(0.396, 0.620, 0.510),
        text: LIGHT.text_primary,
        accent_text: rgb(0.137, 0.420, 0.275),
    },
    ChipTone {
        background: rgb(0.969, 0.937, 0.875),
        background_hover: rgb(0.957, 0.906, 0.808),
        border: rgb(0.682, 0.553, 0.345),
        text: LIGHT.text_primary,
        accent_text: rgb(0.451, 0.337, 0.137),
    },
    ChipTone {
        background: rgb(0.973, 0.910, 0.898),
        background_hover: rgb(0.961, 0.859, 0.835),
        border: rgb(0.706, 0.471, 0.408),
        text: LIGHT.text_primary,
        accent_text: rgb(0.498, 0.255, 0.196),
    },
    ChipTone {
        background: rgb(0.973, 0.898, 0.922),
        background_hover: rgb(0.961, 0.843, 0.890),
        border: rgb(0.682, 0.392, 0.518),
        text: LIGHT.text_primary,
        accent_text: rgb(0.498, 0.176, 0.302),
    },
    ChipTone {
        background: rgb(0.910, 0.918, 0.973),
        background_hover: rgb(0.859, 0.871, 0.961),
        border: rgb(0.451, 0.490, 0.722),
        text: LIGHT.text_primary,
        accent_text: rgb(0.220, 0.247, 0.494),
    },
    ChipTone {
        background: rgb(0.890, 0.957, 0.961),
        background_hover: rgb(0.824, 0.937, 0.945),
        border: rgb(0.353, 0.620, 0.639),
        text: LIGHT.text_primary,
        accent_text: rgb(0.118, 0.396, 0.412),
    },
    ChipTone {
        background: rgb(0.937, 0.957, 0.882),
        background_hover: rgb(0.902, 0.937, 0.808),
        border: rgb(0.529, 0.631, 0.380),
        text: LIGHT.text_primary,
        accent_text: rgb(0.314, 0.388, 0.180),
    },
    ChipTone {
        background: rgb(0.965, 0.945, 0.875),
        background_hover: rgb(0.953, 0.918, 0.804),
        border: rgb(0.682, 0.588, 0.376),
        text: LIGHT.text_primary,
        accent_text: rgb(0.435, 0.365, 0.165),
    },
    ChipTone {
        background: rgb(0.910, 0.910, 0.965),
        background_hover: rgb(0.863, 0.863, 0.953),
        border: rgb(0.408, 0.408, 0.659),
        text: LIGHT.text_primary,
        accent_text: rgb(0.224, 0.224, 0.471),
    },
    ChipTone {
        background: rgb(0.965, 0.910, 0.953),
        background_hover: rgb(0.953, 0.859, 0.933),
        border: rgb(0.627, 0.408, 0.533),
        text: LIGHT.text_primary,
        accent_text: rgb(0.435, 0.224, 0.357),
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

pub fn chip_tone_for_key(key: &str, dark: bool) -> ChipTone {
    let palette = if dark {
        &CHIP_PALETTE_DARK
    } else {
        &CHIP_PALETTE_LIGHT
    };
    let index = stable_color_index(key.trim().to_ascii_lowercase().as_str(), palette.len());
    palette[index]
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

pub fn card_style(theme: &Theme) -> container::Style {
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

pub fn empty_state_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette(theme).bg_surface)),
        border: iced::border::rounded(RADIUS_LG),
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

pub fn divider_line_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette(theme).divider)),
        ..container::Style::default()
    }
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

// ── Text Input Styles ──

pub fn search_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let p = palette(theme);
    let (border_color, border_width) = match status {
        text_input::Status::Active => (p.bg_card, 1.0),
        text_input::Status::Hovered => (p.bg_hover, 1.0),
        text_input::Status::Focused { .. } => (p.focus_ring, 2.0),
        text_input::Status::Disabled => (p.bg_surface, 1.0),
    };
    text_input::Style {
        background: Background::Color(p.bg_surface),
        border: Border {
            color: border_color,
            width: border_width,
            radius: RADIUS_MD.into(),
        },
        icon: p.text_tertiary,
        placeholder: p.text_tertiary,
        value: p.text_primary,
        selection: p.accent_subtle,
    }
}

pub fn field_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let p = palette(theme);
    let (border_color, border_width) = match status {
        text_input::Status::Active => (p.bg_card, 1.0),
        text_input::Status::Hovered => (p.bg_hover, 1.0),
        text_input::Status::Focused { .. } => (p.focus_ring, 2.0),
        text_input::Status::Disabled => (p.bg_surface, 1.0),
    };
    text_input::Style {
        background: Background::Color(p.bg_surface),
        border: Border {
            color: border_color,
            width: border_width,
            radius: RADIUS_MD.into(),
        },
        icon: p.text_tertiary,
        placeholder: p.text_tertiary,
        value: p.text_primary,
        selection: p.accent_subtle,
    }
}

// ── Layout Helpers ──

pub fn section_heading(label: &str) -> Text<'_> {
    text(label).size(FONT_SECTION).style(text_tertiary)
}

pub fn h_divider<'a, Message: 'a>() -> Container<'a, Message> {
    container(text(""))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(divider_line_style)
}

/// 1px vertical hairline, used to separate the sidebar / content / details panes.
pub fn v_divider<'a, Message: 'a>() -> Container<'a, Message> {
    container(text(""))
        .width(Length::Fixed(1.0))
        .height(Length::Fill)
        .style(divider_line_style)
}
