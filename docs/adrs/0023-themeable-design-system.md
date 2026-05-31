# ADR 0023: Themeable design system (Light / Dark / System)

## Status

Accepted

## Context

ADR 0011 (app shell + design tokens) and ADR 0012 (Fluent-inspired design system) established a dark-only visual language: ~16 hardcoded `Color` constants in `ui.rs`, untokenized icon sizes and `.color(CONST)` text calls, a single inline color, and a "System" theme preference that mapped to a fixed dark theme (`TokyoNight`) rather than following the OS. There was also no in-app way to change the theme. The visual presentation needed a real, switchable theme and a token layer that lets the whole UI follow Light/Dark/System without scattered conditionals.

## Decision

- Replace the module-level color constants with a semantic `Palette` struct and two `static` instances, `DARK` and `LIGHT`; `palette(&Theme) -> &'static Palette` selects one via `theme.extended_palette().is_dark`.
- Every style closure and a set of theme-aware text helpers (`text_primary/secondary/tertiary/...`) read `palette(theme).*`, so call sites stay theme-agnostic.
- Add a real Light palette (off-white base, white surfaces, hairline dividers, deeper accent) meeting WCAG AA for text/accent pairs.
- `ThemePreference::System` follows the OS via the `dark-light` crate (see `docs/DEPENDENCIES.md`). The detector is **not** called on the render path: `theme()` reads it at most once per second via a cached `Cell<bool>` + last-checked `Instant`; `Librapix::is_dark_theme()` reads the same cache for icon/chip variant selection.
- Add an in-app theme toggle (System / Light / Dark) in Settings → Appearance, persisted to `config.theme` (`SetThemePreference` message → `load_from_path`/`save_to_path`).
- Theme-aware assets: UI icons swap white (dark) / black (light); the brand logo stays the single blue SVG across themes.
- Visual layer additions, all token-driven and theme-aware: soft elevation helpers (`elevation_low/med/high` → `Shadow`), translucent "glass" dialog surfaces (linear-gradient + edge highlight + shadow), thin translucent overlay scrollbars, a focus-ring token, a tile selection check + thumbnail scrim, and icon-size tokens (`ICON_XS`..`ICON_XL`).
- Centralize all modal dialogs through `render_dialog_frame(content, max_width, max_height)` so they share the glass surface, padding, scroll, and content-hugging-up-to-a-cap sizing.

## Alternatives considered

- Keep dark-only and just retint: rejected — a real Light/System theme was a stated goal.
- True backdrop-blur "liquid glass": rejected — iced 0.14 has no backdrop-filter primitive; it would require a custom wgpu pass sampling the scene behind each panel. Translucent gradient over the dimmed modal scrim achieves a glass read without blur.
- Per-platform OS-appearance detection by hand: rejected in favor of the small `dark-light` crate.
- A runtime theme config file (JSON/TOML tokens): rejected as unnecessary indirection; typed Rust `Palette` statics are simpler and directly consumable (consistent with ADR 0012).

## Consequences

- The entire UI follows Light / Dark / System with no per-call-site branching; new components automatically theme correctly by reading `palette(theme)`.
- Adding or tuning a theme is a localized change in `ui.rs` (`DARK`/`LIGHT` and the elevation/glass helpers).
- The OS detector is isolated behind a throttled cache, so System mode adds no per-frame system call and preserves rendering performance.
- Dialog look/feel is consistent and defined in one place (`render_dialog_frame` + `modal_dialog_style`).
- A new dependency (`dark-light`) is introduced; its behavior, fallback, and call-path constraints are documented in `docs/DEPENDENCIES.md`.
