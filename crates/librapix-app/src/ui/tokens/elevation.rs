use iced::{Color, Shadow, Theme, Vector};

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
