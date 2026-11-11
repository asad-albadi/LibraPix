use iced::{Color, Theme};

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
    pub accent_text: Color,
    pub accent_hover: Color,
    pub accent_subtle: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_disabled: Color,
    pub divider: Color,
    pub border_strong: Color,
    pub destructive: Color,
    pub destructive_hover: Color,
    pub destructive_text: Color,
    pub success: Color,
    pub warning: Color,
    pub focus_ring: Color,
    pub skeleton: Color,
}

const fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b, a: 1.0 }
}

pub(in crate::ui) const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color { r, g, b, a }
}

/// Quiet neutral chrome above a darker media canvas; blue remains the accent.
pub static DARK: Palette = Palette {
    bg_base: rgb(0.078, 0.086, 0.094),
    bg_layer: rgb(0.102, 0.110, 0.122),
    bg_surface: rgb(0.125, 0.133, 0.145),
    bg_card: rgb(0.153, 0.161, 0.173),
    bg_hover: rgb(0.180, 0.188, 0.204),
    bg_selected: rgb(0.075, 0.200, 0.310),
    accent: rgb(0.0, 0.400, 0.740),
    accent_text: rgb(0.350, 0.680, 1.0),
    accent_hover: rgb(0.0, 0.440, 0.800),
    accent_subtle: rgb(0.0, 0.300, 0.550),
    text_primary: rgb(0.961, 0.961, 0.961),
    text_secondary: rgb(0.780, 0.790, 0.810),
    text_tertiary: rgb(0.690, 0.710, 0.740),
    text_disabled: rgb(0.420, 0.440, 0.470),
    divider: rgb(0.230, 0.247, 0.271),
    border_strong: rgb(0.420, 0.450, 0.490),
    destructive: rgb(0.800, 0.220, 0.270),
    destructive_hover: rgb(0.730, 0.160, 0.220),
    destructive_text: rgb(1.0, 0.580, 0.620),
    success: rgb(0.550, 0.830, 0.580),
    warning: rgb(1.0, 0.702, 0.278),
    focus_ring: rgb(0.350, 0.680, 1.0),
    skeleton: rgb(0.230, 0.247, 0.271),
};

/// Soft off-white base with white surfaces; deeper accent + text for WCAG AA.
pub static LIGHT: Palette = Palette {
    bg_base: rgb(0.930, 0.940, 0.950),
    bg_layer: rgb(0.960, 0.965, 0.973),
    bg_surface: rgb(1.0, 1.0, 1.0),
    bg_card: rgb(0.980, 0.984, 0.988),
    bg_hover: rgb(0.900, 0.920, 0.940),
    bg_selected: rgb(0.847, 0.918, 0.984),
    accent: rgb(0.0, 0.357, 0.682),
    accent_text: rgb(0.0, 0.357, 0.682),
    accent_hover: rgb(0.0, 0.408, 0.769),
    accent_subtle: rgb(0.788, 0.875, 0.961),
    text_primary: rgb(0.110, 0.110, 0.118),
    text_secondary: rgb(0.290, 0.310, 0.340),
    text_tertiary: rgb(0.350, 0.370, 0.400),
    text_disabled: rgb(0.620, 0.620, 0.635),
    divider: rgb(0.820, 0.840, 0.870),
    border_strong: rgb(0.490, 0.520, 0.560),
    destructive: rgb(0.700, 0.100, 0.140),
    destructive_hover: rgb(0.620, 0.070, 0.110),
    destructive_text: rgb(0.700, 0.100, 0.140),
    success: rgb(0.120, 0.400, 0.190),
    warning: rgb(0.500, 0.300, 0.0),
    focus_ring: rgb(0.0, 0.357, 0.682),
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

#[cfg(test)]
mod tests {
    use crate::ui::tokens::color::{
        CHIP_PALETTE_DARK, CHIP_PALETTE_LIGHT, ChipTone, DARK, LIGHT, chip_tone_for_key, palette,
    };
    use iced::Theme;

    fn assert_tone_eq(actual: ChipTone, expected: ChipTone) {
        assert_eq!(actual.background, expected.background);
        assert_eq!(actual.background_hover, expected.background_hover);
        assert_eq!(actual.border, expected.border);
        assert_eq!(actual.text, expected.text);
        assert_eq!(actual.accent_text, expected.accent_text);
    }

    #[test]
    fn repeated_and_normalized_keys_keep_the_same_tone() {
        for dark in [false, true] {
            let expected = chip_tone_for_key("landscape", dark);
            assert_tone_eq(chip_tone_for_key("landscape", dark), expected);
            assert_tone_eq(chip_tone_for_key("  LANDSCAPE  ", dark), expected);
        }
    }

    #[test]
    fn representative_keys_keep_their_existing_palette_entries() {
        for (key, index) in [("landscape", 2), ("portrait", 10), ("gameplay", 1)] {
            assert_tone_eq(chip_tone_for_key(key, true), CHIP_PALETTE_DARK[index]);
            assert_tone_eq(chip_tone_for_key(key, false), CHIP_PALETTE_LIGHT[index]);
        }
    }

    #[test]
    fn theme_selects_the_corresponding_semantic_palette() {
        assert!(std::ptr::eq(palette(&Theme::Dark), &DARK));
        assert!(std::ptr::eq(palette(&Theme::Light), &LIGHT));
    }
}
