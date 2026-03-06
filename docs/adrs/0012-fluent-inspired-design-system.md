# ADR 0012: Fluent-inspired design system

## Status

Accepted

## Context

The initial app-shell and design-token baseline (ADR 0011) established structural layout but the visual presentation still resembled an internal debug surface. The UI needed a comprehensive visual redesign to feel like a real desktop media application.

## Decision

- Adopt a Fluent-inspired dark theme color palette with neutral grays and Windows Fluent blue accent.
- Expand the design token system to cover colors, spacing, typography, radii, and layout dimensions.
- Implement custom style functions for buttons (primary, subtle, action, nav, card), text inputs (search, field), and containers (header, sidebar, details, cards, empty states).
- Replace the vertical item list with a responsive thumbnail grid for gallery browsing.
- Style navigation items as full-width buttons with active state highlighting.
- Show library root status using colored dot indicators instead of text labels.
- Use product-oriented language throughout i18n (e.g. "Search photos, videos, tags..." instead of "Run read-model query").
- Build all visual components using Iced primitives and centralized style functions rather than importing external UI kits.

## Alternatives considered

- Import a Fluent UI component library: rejected because no mature Iced-compatible Fluent kit exists, and building internally keeps the dependency footprint minimal.
- Use a CSS-like token file (JSON/TOML): rejected as unnecessary indirection; Rust constants in `ui.rs` are simpler, type-safe, and directly consumable.

## Consequences

- The UI now has a coherent visual language aligned with desktop media app expectations.
- All visual values are centralized in `ui.rs`, making future theme iterations straightforward.
- The gallery grid provides a more natural media browsing experience than the previous vertical list.
- Product-oriented i18n strings make the app feel less developer-oriented.
- The design system can be extended with additional component styles as needed.
