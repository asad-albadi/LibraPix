# ADR 0025: Centralized app-local UI system

## Status

Accepted

## Context

LibraPix's themeable palette (ADR 0023) and unified Library (ADR 0024) already
provide a native Iced foundation. Repeated control and modal composition in
`main.rs` and a monolithic `ui.rs` made visual consistency and behavior-preserving
changes difficult. The approved direction is media-first, shadcn-inspired
clarity: restrained neutral surfaces, compact controls, quiet borders, blue
selection/focus, and media remaining the dominant content. This is visual and
compositional inspiration, not adoption of shadcn source or web architecture.

## Decision

Keep the design system inside `librapix-app`, using Iced 0.14 built-in widgets
and closure-based styles. Replace `ui.rs` with four ownership layers:

- `ui/tokens`: semantic light/dark palettes, deterministic chip tones, spacing,
  typography, sizing, radii, and elevation.
- `ui/styles`: widget appearance derived from tokens, semantic variants, and
  Iced interaction status.
- `ui/components`: small, message-generic composed controls receiving localized
  content and optional messages, without application services or state access.
- `ui/patterns`: reusable pane, dialog, field, and status structures; callers own
  validation, localization, state, and side effects.

`views/` composes these layers from product presentation state and emits app
messages. Shell, Library, details, gallery/timeline composition, scrubber, and all
seven dialogs have focused view modules. State/update/subscriptions, validation,
persistence, background scheduling, and layout/cache ownership remain in the
existing application layer; this decision does not claim to modularize that
remaining orchestration in `main.rs`.

Both panes collapse independently. The left rail contains only Library, Add
Library, Refresh, and Expand, with tooltips; root management returns on expansion.
The right edge retains a compact horizontal Details label and expansion action.
Two backward-compatible `[ui]` config booleans default to expanded. Toggles apply
immediately, reload config, and save only UI preferences through the config crate.
Save failure retains the session choice and logs an error, displaying a localized,
dismissible overlay notice. A permanent shell stack preserves media scroll state
as that notice appears or disappears.

Recurring controls use shared constructors. Specialized media overlays, semantic
text, scrollbars, and scrubber geometry may consume centralized styles directly
when a wrapper adds no useful behavior. Keep only modules/variants with actual
consumers: unused checkbox/select wrappers, placeholder motion/slider modules,
and legacy facades were removed rather than retained as speculative API.

## Alternatives considered

- Separate `librapix-ui` crate: rejected; one application consumes these helpers,
  so another package adds API/dependency overhead without a useful boundary.
- Styles-only extraction: rejected; it leaves repeated composed controls and
  dialogs in `main.rs` and does not centralize their interaction/layout vocabulary.
- Web UI or custom renderer/widget framework: rejected; neither is required for
  the approved native desktop experience or existing product behavior.

## Consequences

- Future controls reuse the owning token/style/component/pattern module; add an
  abstraction only for a real consumer. Do not recreate root compatibility facades.
- Product views may read app presentation state, but reusable UI must not import
  `Librapix`, storage/indexing services, or ffmpeg process logic.
- Loading components receive a presentation phase; they own no timers or work.
  Make Short uses one stage line and indeterminate dots, with a dedicated timer
  while the open dialog is running. Numeric progress requires real measured work.
- Shared dialogs bound and scroll their body while keeping actions visible;
  detailed ffmpeg errors belong in that body rather than the fixed footer.
- Existing keyboard routing, cancellation/results, source-media safety,
  background execution, viewport windowing, and layout caches are preserved.
- No dependency, MSRV, workspace crate, database schema, or video-processing
  change is needed. Config merging is sequential, not cross-process locking.
- Layout/event/render tests guard pane, control, dialog, and performance-sensitive
  behavior. They supplement, not replace, native Windows/Linux visual checks.

## Official references

- [Iced 0.14 Button composition, optional messages, and closure styling](https://docs.rs/iced/0.14.0/iced/widget/button/struct.Button.html)
- [Iced interval subscriptions](https://docs.rs/iced/0.14.0/iced/time/fn.every.html)

See `docs/architecture/ui.md`, `layers.md`, `media-ui.md`, and `message-flow.md`
for the delivered boundaries and interaction flow. This refines the presentation
boundary of ADRs 0011/0012/0023 without changing ADR 0024's projection strategy.
