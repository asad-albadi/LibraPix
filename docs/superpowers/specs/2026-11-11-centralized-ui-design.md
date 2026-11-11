# Centralized LibraPix UI Design

## Intent

Create a centralized, app-local Iced design system for LibraPix, inspired by the
clarity and composability of shadcn/ui without importing web, React, Tailwind, or
Radix architecture into a native desktop application.

The work should make LibraPix visually consistent, reduce repeated widget
composition, and establish one documented component vocabulary for every
existing screen and dialog. The resulting UI should remain recognizably
LibraPix: media-first, compact, desktop-native, and optimized for browsing
screenshots and recordings.

## Why

The current application already has semantic light/dark palettes, spacing,
typography, radius, elevation, and widget style functions in
`crates/librapix-app/src/ui.rs`. However:

- `ui.rs` is a single 883-line style module rather than a component system.
- `main.rs` is approximately 14,000 lines and owns most composed controls,
  dialogs, sidebars, status treatments, and form patterns.
- Similar controls are assembled repeatedly with local size, padding, icon,
  tooltip, and state decisions.
- Visual consistency currently depends on each call site using low-level styles
  correctly.

This is an extraction and normalization project, not a rewrite from zero.

## Goals

- Establish one app-local UI module tree for tokens, styles, reusable components,
  and reusable product patterns.
- Migrate every current LibraPix screen and dialog to the centralized vocabulary.
- Adopt the approved media-first shadcn-inspired visual direction.
- Preserve the existing three-pane Library workflow.
- Make the left library pane and right details pane independently collapsible.
- Use the approved hybrid collapse treatment:
  - collapsed left pane becomes a compact icon navigation rail
  - collapsed right pane becomes a narrow labeled `Details` edge
- Persist both collapse states independently across application restarts.
- Preserve light, dark, and system theme behavior.
- Preserve all current functionality, background work, and non-destructive media
  behavior.
- Reduce presentation code in `main.rs` by moving reusable view composition into
  focused modules.
- Add targeted component and presentation-state tests where Iced permits stable
  assertions.

## Non-goals

- Do not use shadcn/ui source, React, Tailwind, Radix, HTML, or CSS in the app.
- Do not make LibraPix look like a generic web administration dashboard.
- Do not create a new `librapix-ui` workspace crate.
- Do not introduce a custom renderer or replace Iced.
- Do not change storage, indexing, search, thumbnail, or video-generation logic.
- Do not redesign the media grid, timeline algorithm, filtering semantics, or
  details content.
- Do not add new product features while migrating UI.
- Do not add speculative plugin, skin, or runtime theme systems.
- Do not create custom Iced widgets when composition of built-in widgets is
  sufficient.

## Anti-goals

The project is unsuccessful if it:

- moves domain or persistence behavior into UI components
- creates a second competing style path
- leaves old and new component conventions indefinitely mixed
- makes common controls harder to understand than direct Iced composition
- introduces generic abstractions that exist only to imitate a web framework
- reduces gallery space or media prominence
- changes keyboard shortcuts, dialogs, or action semantics unintentionally

## Approved Visual Direction

The approved direction is **Media-first shadcn**:

- restrained neutral surfaces and subtle one-pixel borders
- compact, consistent controls
- semantic component variants and predictable interaction states
- LibraPix blue retained for primary actions, active navigation, selection, and
  focus
- darker, quieter gallery canvas so media remains visually dominant
- minimal elevation; shadows communicate layering instead of decoration
- stable layouts during loading and state transitions
- purposeful density suitable for a desktop media manager

The approved sidebar behavior is **Hybrid**:

- expanded left pane keeps the current library navigation and management content
- collapsed left pane becomes an icon rail with tooltips and an explicit expand
  affordance
- expanded right pane keeps preview, metadata, tags, and actions
- collapsed right pane becomes a narrow, labeled `Details` edge with an explicit
  expand affordance
- each pane toggles independently
- collapse/expand does not change selection or active route
- the gallery recomputes against the newly available width through the existing
  responsive layout path
- pane state is persisted after each user toggle
- the collapsed left rail exposes global actions only: Library, Add Library,
  Refresh, and Expand; individual library roots and their Edit/Statistics actions
  remain available only in the expanded sidebar

The visual exploration is stored under `.superpowers/brainstorm/` and is design
reference only, not production source.

## Approaches Considered

### A. App-local design system (selected)

Keep the Iced-specific design system inside `librapix-app`. Split tokens, styles,
components, and product patterns into focused modules. This preserves the current
dependency direction and creates reuse exactly where it is needed.

### B. New `librapix-ui` crate

This would create a formal package boundary but add API and dependency overhead
for a component system currently consumed by one application. It is premature
and was rejected.

### C. Styles-only refactor

This would split `ui.rs` but leave repeated composed controls and dialogs in
`main.rs`. It does not satisfy the requirement for centralized components and
was rejected.

## Module Structure

Replace `crates/librapix-app/src/ui.rs` with this module tree:

```text
crates/librapix-app/src/ui/
  mod.rs
  tokens/
    mod.rs
    color.rs
    elevation.rs
    motion.rs
    radius.rs
    sizing.rs
    spacing.rs
    typography.rs
  styles/
    mod.rs
    button.rs
    checkbox.rs
    container.rs
    input.rs
    progress.rs
    scrollable.rs
    select.rs
    slider.rs
    text.rs
  components/
    mod.rs
    badge.rs
    button.rs
    card.rs
    checkbox.rs
    empty_state.rs
    icon_button.rs
    input.rs
    loading.rs
    notice.rs
    select.rs
    separator.rs
    tabs.rs
    tooltip.rs
  patterns/
    mod.rs
    details_pane.rs
    dialog.rs
    field.rs
    sidebar.rs
    status.rs
```

Product-level view modules should be extracted separately from the design system:

```text
crates/librapix-app/src/views/
  mod.rs
  shell.rs
  library.rs
  details.rs
  dialogs/
    mod.rs
    about.rs
    filters.rs
    library.rs
    library_statistics.rs
    make_short.rs
    new_media.rs
    settings.rs
```

`ui/` defines reusable visual language. `views/` composes that language using
LibraPix product state and messages. Domain behavior remains outside both.

## Layer Responsibilities

### Tokens

Tokens contain semantic values only:

- light and dark semantic colors
- spacing scale
- type scale
- control and icon sizes
- corner radii
- pane widths
- elevation levels
- short interaction-duration constants where Iced supports them

Colors should describe purpose, not appearance. Examples:

- `background`
- `surface`
- `surface_raised`
- `surface_hover`
- `border`
- `border_strong`
- `foreground`
- `foreground_muted`
- `primary`
- `primary_hover`
- `destructive`
- `warning`
- `success`
- `focus_ring`

Existing deterministic tag colors remain supported but move into the color token
module as a documented specialized palette.

### Styles

Styles translate semantic variants and Iced widget status into Iced style values.
They are the only layer that should normally construct `button::Style`,
`container::Style`, `text_input::Style`, and corresponding widget styles.

Use the current Iced 0.14 closure-based styling model. Do not enable `advanced`
or introduce a custom `Theme` catalog solely for this migration. Iced officially
supports widget `style` closures receiving `Theme` and widget status, which fits
the existing implementation and keeps the change bounded.

### Components

Components are small compositional helpers over built-in Iced widgets. They own
consistent dimensions, spacing, icon placement, labels, tooltip behavior, and
variant selection.

Components remain generic over the application message type and receive messages
or optional messages from callers. They must not import `Librapix`, storage
types, indexing types, or product orchestration functions.

Core variants:

- Button: `Primary`, `Secondary`, `Outline`, `Ghost`, `Destructive`, `Link`
- Button size: `Small`, `Default`, `Large`, `Icon`
- Badge: `Default`, `Secondary`, `Outline`, `Success`, `Warning`, `Destructive`
- Input: normal, focused, disabled, invalid
- Card: default, interactive, selected
- Loading: compact spinner, indeterminate working row, skeleton block
- Notice: shell-level informational, success, warning, and error feedback
- Tabs: list, trigger, active trigger

Components should expose concise constructors/builders rather than mirror every
Iced option. Uncommon layout remains at the view call site.

### Patterns

Patterns combine components into recurring LibraPix structures:

- labeled form field with optional help and validation
- standard modal frame with header, scrollable body, and footer actions
- application sidebar and compact navigation rail
- details pane and collapsed details edge
- activity/status row

Patterns may understand presentation state but not business behavior.

### Views

Views consume `Librapix` presentation state and emit the existing `Message`
variants. They compose UI components and patterns for a product surface.

Moving a render function from `main.rs` to `views/` must not move its update or
background-task logic with it.

## Component Inventory

The migration covers every existing recurring UI control:

- application surface and raised surface
- header
- sidebar and sidebar navigation item
- compact navigation rail item
- details pane and collapsed details edge
- primary, secondary, outline, ghost, destructive, and icon buttons
- search input and form input
- select/pick list
- checkbox/toggle treatment using the existing available Iced controls
- tabs/segmented controls
- filter chips and managed tags
- badges and media-kind indicators
- cards and selectable media cards
- tooltip/help badge
- horizontal and vertical separators
- scrollable treatment
- slider and timeline scrubber treatment
- progress/loading/skeleton treatments
- empty states
- status/activity treatments
- dialog shell, header, content, and footer
- labeled field, help text, and validation message

Specialized media layout, thumbnail rendering, timeline markers, and justified
row calculations remain product views rather than generic components.

## Sidebar State and Persistence

Add two settings to the existing configuration model:

```text
left_sidebar_collapsed: bool
details_sidebar_collapsed: bool
```

Requirements:

- both default to `false` for existing and new users
- deserialization must remain backward compatible when fields are absent
- toggling updates application state immediately
- persistence uses the existing config save path
- a persistence failure should not revert the visible toggle; report it through
  a shell-level non-blocking error notice and the existing runtime log
- state changes do not trigger indexing, projection rebuilds, or storage writes
- responsive media layout may naturally recalculate for the changed width

The left rail retains only global actions through icons with concise tooltips:
Library, Add Library, Refresh, and Expand. It does not render dynamic library
roots or per-root Edit/Statistics controls; expanding the pane restores those
controls. The right collapsed edge must clearly communicate `Details` and reopen
the inspector. The expanded details pane remains selection-sensitive as it is
today.

Preference-save failures use a small shell-level error notice that remains
visible even when the left pane is collapsed. The notice overlays rather than
resizes the media layout, is dismissible, and contains a concise localized
summary. This establishes a reusable notice component but does not broaden the
migration into a general notification-center feature.

## Accessibility and Interaction

- Preserve keyboard shortcuts and text-input routing.
- Every icon-only control requires a tooltip and an accessibility-friendly label
  where supported by Iced.
- Focus indication must be visible in light and dark themes.
- Disabled controls must remain visibly distinct and must not emit messages.
- Do not communicate warning, error, success, or selection using color alone.
- Maintain practical desktop hit targets even when visual controls are compact.
- Dialog content must remain keyboard reachable and layout-stable.
- Collapse affordances remain present in both expanded and collapsed states.

## Migration Strategy

The migration should be incremental and continuously buildable:

1. Create module skeleton and move existing tokens/styles without visual change.
2. Add component variants and tests alongside current call sites.
3. Migrate shell, header, buttons, inputs, separators, tooltips, and status UI.
4. Implement and persist hybrid sidebar collapse behavior.
5. Migrate dialogs onto shared dialog and field patterns.
6. Migrate cards, chips, tabs, badges, loading, and empty states.
7. Extract product view functions from `main.rs` into `views/`.
8. Remove superseded style helpers and direct recurring widget composition.
9. Perform final visual normalization in light and dark themes.

Do not combine broad visual changes with structural extraction in the same commit
where avoidable. First preserve behavior, then apply the approved visual tokens.

## Testing

### Unit tests

- palette semantic values select correctly for light and dark themes
- component variant/style mapping covers active, hovered, pressed, focused,
  invalid, and disabled states as applicable
- deterministic tag colors remain stable
- sidebar config defaults are backward compatible
- sidebar config values round-trip through TOML
- collapse messages update only intended presentation state
- disabled components do not receive actionable messages where testable

### Presentation tests

- left expanded and collapsed compositions expose the appropriate controls
- right expanded and collapsed compositions expose the appropriate affordances
- dialog pattern supports title, body, validation, and footer actions
- smooth-warning, Make Short loading/cancel/success/failure presentation remains
  intact after migration
- video-only actions remain video-only
- all key option tooltips remain present

Prefer tests over component data/variant decisions and state transitions rather
than brittle pixel snapshots. If practical, add a development-only component
gallery view or test harness only when it does not ship as product functionality;
otherwise verify visually through existing screens.

### Validation loop

Run serially:

```text
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Manual checks must cover:

- light, dark, and system themes
- all current dialogs
- left pane expanded/collapsed
- right pane expanded/collapsed
- persisted pane state after restart
- gallery and timeline resizing after pane toggles
- keyboard shortcuts and focus behavior
- loading, success, failure, warning, disabled, and empty states
- common window sizes on Windows and Linux

## Documentation Impact

This is a meaningful presentation architecture change. Update in the same
workstream:

- `CHANGELOG.md`
- `docs/architecture/layers.md`
- `docs/architecture/media-ui.md`
- `docs/architecture/message-flow.md` for pane toggle/config flow
- `docs/DEPENDENCIES.md` only if a dependency changes; no dependency is expected
- a new ADR documenting the app-local design-system boundary and why a separate
  crate was rejected
- `docs/TROUBLESHOOTING.md` only for confirmed issues discovered during migration

## Risks and Controls

### Large migration surface

Control: migrate by component family and keep every commit buildable.

### Behavior regression hidden by visual refactoring

Control: preserve existing messages and update paths; add state-level tests before
moving each major view.

### Over-generic component APIs

Control: implement only variants used by LibraPix. Add variants when a real call
site needs them.

### Iced lifetime and generic complexity

Control: prefer functions returning `Element<'a, Message>` and simple typed
options. Keep direct Iced composition when a wrapper would obscure ownership.

### Gallery performance regression

Control: do not wrap individual media cards in extra dynamic abstractions; retain
viewport-bounded rendering and justified-layout caches; compare existing runtime
diagnostics before and after migration.

## Completion Criteria

- `ui.rs` is replaced by the documented module tree.
- All reusable controls use centralized tokens, styles, and components.
- Existing product views are separated from the component library.
- Both sidebars collapse independently using the approved hybrid treatment.
- Both sidebar states persist across restarts.
- Light, dark, and system themes remain coherent.
- No product behavior or background execution path changes unintentionally.
- No new UI crate or unnecessary dependency is introduced.
- Architecture documentation and ADR are current.
- Formatting, checks, clippy, and tests pass.
- Manual visual verification is completed on supported desktop platforms.

## Official References Consulted

- Iced 0.14 theming and closure-based styling:
  <https://docs.iced.rs/iced/index.html#theming>
- Iced button status-aware styling:
  <https://docs.iced.rs/iced/widget/struct.Button.html>
- Iced container styling and composition:
  <https://docs.iced.rs/iced/widget/struct.Container.html>

