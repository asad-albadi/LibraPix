# UI Architecture

Librapix UI uses a centralized, app-local, media-first shadcn-inspired design
system with a native Iced app-shell layout. ADR 0025 refines the earlier Fluent
direction; ADR 0023's Light/Dark/System policy and ADR 0024's unified Library remain.

## App shell baseline

### Presentation module ownership

- `ui/` owns tokens, widget styles, reusable components, and presentation patterns;
  consumers import the owning module rather than a legacy root-level facade.
- `views/` owns product composition, including gallery/timeline widget assembly
  and the timeline scrubber. Layout caches, row-window calculations, diagnostics,
  anchor generation, subscriptions, and background work remain in `main.rs`.
- Specialized media styles remain direct calls to `ui/styles`: thumbnail
  placeholders/scrims preserve image overlays; scrubber panel/chip styles preserve
  track geometry; modal backdrops preserve dismissal layering; shell header/base
  styles define app regions; scrollbars and semantic text styles are applied to
  built-in Iced widgets without wrapping them in another component.
- Determinate runtime activity styling is retained only for actual item counts,
  not for indeterminate Make Short generation. `raised_surface_style` preserves
  the borderless, elevated preview and appearance insets; these are not selectable
  media cards. `timeline_marker_style` preserves content-sized year labels and
  their quieter text/pressed feedback without the fixed height of action buttons.
- Library lifecycle dots/labels retain direct semantic palette colors because
  their active, unavailable, and deactivated states are product presentation,
  not generic button variants. The scrubber retains the native Iced slider and exact geometry, with centralized
  semantic rail/handle styling; no custom slider implementation is needed.
- Unconsumed staged checkbox/select components and styles, placeholder slider/
  motion modules, unused variants, and disabled-text/asset helpers were removed
  rather than kept under dead-code allowances. Existing dialogs keep their
  button/tab-based selections; no widget or action behavior changes.
- `ui/tokens` intentionally aggregates semantic values; component-level variant
  exports remain their supported constructor API. Only the obsolete root UI and
  style facades are removed. `ui/components/text.rs` owns section headings and
  `ui/components/tooltip.rs` owns contextual help hints.

### Shell regions

- Top header with product identity (brand-blue SVG logo + wordmark), integrated search bar, update-status chip, Settings button, About button, and GitHub link.
- Left sidebar with sectioned navigation: a single **Library** entry (the Grid/Timeline split is a segmented toggle in the content header, not separate tabs) plus library roots; operational controls (indexing, ignore rules, diagnostics) live in the Settings dialog.
- Main media pane for gallery grid, timeline groups, and search result cards.
- Right details pane for preview, file info, tags, and actions.
- Details pane shows an intentional empty state when no media is selected.

## Design system

Shared visual tokens and widget styles are centralized in `librapix-app/src/ui/`:

- `tokens/` owns the existing palette/chip tones, elevation, spacing, typography, sizing, and radii.
- `styles/` owns the existing Iced button, container, input, scrollbar, and text style functions.
- `components/` centralizes composed controls, including a single-button media card and message-generic managed chips. `patterns/` centralizes pane, dialog, field, and status composition.
- `views/library.rs` and `views/details.rs` expose product `view(&Librapix)` interfaces; `views/gallery.rs` and `views/timeline.rs` own widget assembly and the scrubber. Layout/window/cache helpers stay in the app. Consumers use explicit `ui/tokens`, `ui/styles`, component, and pattern imports; the legacy root facade and superseded helpers are removed.
- The migration applies the approved media-first component visuals without changing product/background behavior; the approved migration is recorded in `docs/superpowers/plans/2026-11-11-centralized-ui.md`.

### Color palette / theming
- Semantic `Palette` struct with two `static` instances, `DARK` and `LIGHT`; `palette(&Theme)` selects one via `theme.extended_palette().is_dark`.
- Background hierarchy: base, layer, surface, card, hover, selected. Accent (cool blue) for primary actions/selection. Text hierarchy: primary, secondary, tertiary, disabled. Semantic success/warning/divider, plus a `focus_ring` token.
- Style closures and theme-aware text helpers (`text_primary/secondary/...`) read `palette(theme).*`, so call sites are theme-agnostic.
- Theme preference is `System` / `Dark` / `Light` (`config.theme`). `System` follows the OS via the `dark-light` crate, resolved by `theme()` and cached/throttled off the render path (`Librapix::is_dark_theme()` reads the same cache for icon/chip variants). See `docs/DEPENDENCIES.md`.
- A second Light chip palette mirrors the dark one (`chip_tone_for_key(key, dark)`).

### Spacing and typography
- Spacing scale: 2xs through 2xl (2px to 32px).
- Typography: display, title, subtitle, section, body, caption sizes.
- Consistent padding and gap values across all surfaces.

### Component styles
- Action-button variants: Primary, Secondary, Outline, and Ghost. Tabs/navigation, interactive/selected cards, and managed chips have specialized centralized mappings; unused staged variants are not public API.
- Managed chips: deterministic color-mapped chip surfaces for tags/rules with inline edit/remove/toggle actions.
- Text input styles: search and field share the same rounded-corner language; the focused state uses a 2px `focus_ring` border.
- Container styles: header, sidebar, details pane, cards, empty states, thumbnail placeholders, dividers, timeline scrubber surfaces, media-kind badges, modal backdrop, and modal dialog surfaces.
- Icon-size tokens `ICON_XS`..`ICON_XL` replace raw pixel literals at icon/logo call sites.

### Elevation and glass
- Centralized elevation helpers (near-black on dark, soft gray on light) feed card and dialog shadows; quiet borders separate the other surfaces. Unconsumed elevation helpers were removed rather than retained as dormant API.
- Modal dialogs use a translucent "glass" surface: a vertical gradient at <1.0 alpha + a faint edge highlight + a strong shadow over the dimmed backdrop. There is **no** true backdrop blur (iced has no backdrop-filter primitive — see ADR 0023).

### Scrollbars
- `scrollbar_style` renders thin, rounded, translucent overlay scrollbars (invisible rail, pill scroller that darkens on hover/drag), applied to the gallery, sidebar, details pane, dialog frame, and diagnostics list.

### Dialogs
- `ui/patterns/dialog.rs` accepts a `DialogSpec` with a localized borrowed or owned title, optional labeled close message, body, optional footer, and size caps. A private app-local `dialog_layout` adapter measures the native column's natural height at the available width, then lays it out again at that height capped to the available space. The second pass reserves header/footer before allocating the scrolling body. Short frames shrink; overflowing bodies scroll without displacing actions. The adapter forwards the native widget tree, input, operations, overlays, and drawing unchanged; it is not a custom renderer.
- All seven product dialogs live in `views/dialogs/`: About, filters, library management, library statistics, Make Short, new media, and settings. Each module exposes one `view(&Librapix)` function and composes shared buttons, inputs, tabs, separators, fields, and status patterns. Dedicated wrapping, statistics, tooltip, and preview helpers live with their consuming dialog; `ui/components/chip.rs` supplies managed tag/rule chips to dialogs and details.
- `views/dialogs/mod.rs` owns the unchanged modal backdrop/content-click wrapper; `views/shell.rs` retains overlay ordering. Escape handling, state, validation decisions, request building, background tasks, cancellation, and lifecycle remain application-owned.
- Make Short shows one stage line and one indeterminate working indicator, Cancel while running, and a Smooth-only warning. Success, failure details, canceled feedback, validation, and option hover help remain distinct. Library path errors use the existing `root_status` decision in a shared field instead of being hidden behind the modal.
- Make Short owns a separate presentation phase driven by a 180 ms Iced timer only while its dialog is open and its run state is Running (including cancel-requested work). Preview loading has a separate timer/phase; animation never represents numeric generation progress. Terminal/hidden dialogs unsubscribe. Only the concise result summary and actions stay in the fixed footer; unrestricted failure details stay in the scrolling body.
- `ui/patterns/field.rs` composes caller-owned controls without changing IDs or message wiring. A validation error replaces help text rather than showing both.
- `ui/patterns/status.rs` provides neutral, working, success, warning, and error presentation. Working reuses the phase-driven indeterminate indicator; there is no percentage/progress-value input. Callers own localized stage text and timer subscriptions.

### Layout helpers
- `section_heading()`: small-caps section label.
- `ui/components/separator.rs`: horizontal / vertical hairlines separating regions and sections.

## Interaction model

- Selection is explicit app state (`selected_media_id`).
- Single click selects a media item and loads details.
- Double-click opens the media item in the OS default external app.
- Double-click detection tracks last-click media id and timestamp at app level.
- Gallery cards and timeline rows are clickable buttons with card styles.
- New-file detection during live filesystem refresh can surface an in-app modal announcement dialog with preview, metadata, and quick actions (view, open file, copy file, dismiss).
  - Dialog is centered within a full-screen backdrop and constrained with max width/height for product-like modal sizing.
- Timeline mode includes a fast right-side scrubber:
  - drag/click updates scrub value
  - scrub maps to projection anchors by nearest normalized anchor position
  - year markers are positioned along the scrub track from anchor normalized positions
  - a date chip is shown while dragging
  - entering scrub mode keeps scrubber lane geometry stable (no first-click lateral snap)
- Search is triggered via Enter key in the header search bar.
- Header update chip interaction:
  - `Checking...` while release status is in flight.
  - `Up to date` when current version matches the latest stable release.
  - `New release` when a newer stable release exists.
  - Click behavior is stateful:
    - opens latest release page when `New release` is active
    - otherwise triggers a manual check, rate-limited to once every 5 minutes
- Root selection uses styled nav buttons with status dot indicators; display names shown when set.
- Library management uses a unified add/edit dialog:
  - browse-first folder selection
  - optional manual path field toggle
  - display-name editing
  - root-level tag chip add/edit/remove
  - save + save-and-add-another flow for batch library registration
- Each library row has explicit `Edit` and `Stats` actions.
- Library statistics use a dedicated dialog (separate from Edit Library) with maintained persisted values per root.
- Copy shortcuts are supported through ignored keyboard events (no text-input conflicts):
  - `Cmd/Ctrl+C`: copy selected file
  - `Cmd/Ctrl+Shift+C`: copy selected path

## Filtering

- Media pane toolbar includes four filter axes:
  - library (All / per-root chips when multiple roots exist)
  - type (All / Images / Videos)
  - extension (PNG, JPG, GIF, WEBP, MP4, MOV, etc.)
  - tag (available indexed tags)
- Extension chip set adjusts based on active type filter: image extensions when type is Images, video extensions when type is Videos, both when All.
- Changing type resets the extension filter.
- Filters apply to gallery, timeline, and search projections simultaneously.
- Filter state lives in app state (`filter_media_kind`, `filter_extension`, `filter_tag`, `filter_source_root_id`); presentation is in the filter dialog.
- Filter logic is applied at the app orchestration layer, not inside widgets.
- `All` means no media-kind filter; it includes both images and videos.

## Media pane layout

- Media pane toolbar (title, refresh, total/images/videos stats, filter chips) is rendered outside the scrollable region.
- Only the browse content and search results scroll; the toolbar remains fixed at the top.
- This prevents the scrollbar from overlapping toolbar controls.
- The media scrollable uses embedded vertical scrollbar spacing, reserving a dedicated gutter so scrollbar chrome does not overlay media cards.
- Timeline mode keeps this layout and adds a side scrubber column inside the media pane body (without changing shell structure).

## Centralized media-view architecture

Gallery, timeline, and search views share a unified media-view architecture:

- **BrowseItem**: common data model with `media_id`, `title`, `media_kind`, `metadata_line`, `thumbnail_path`, `aspect_ratio`, `is_group_header`, and optional `group_image_count`, `group_video_count` for timeline headers.
- **`views::library::media_card()`**: shared card rendering primitive used by Grid, Timeline, and search.
- **resolve_thumbnail()**: unified thumbnail resolution for images (Lanczos3) and videos (ffmpeg).
- **populate_media_cache()**: caches read-model data alongside browse items to avoid per-click storage queries.
- **aspect_ratio_from()**: computes aspect ratio from stored dimensions (defaults to 1.5 for unknown).

## Gallery rendering

- Gallery uses a Google-Photos-style adaptive justified row layout.
- Uses Iced `responsive` widget to access available width and compute row heights dynamically.
- Row building algorithm accumulates items until the resulting row height drops to or below the target (200px).
- Each item receives `FillPortion` proportional to its aspect ratio for correct width distribution.
- Row heights are clamped between 100px and 350px.
- Images maintain their natural aspect ratios; no forced cropping unless the image is inherently mismatched.
- Thumbnails use `ContentFit::Cover` within their allocated card space.
- Selected cards show an accent-colored border.
- Cards include a top-right media-kind badge icon (image/video) for quick scanning.
- Metadata row under each thumbnail is compact and padded (`kind · size · dimensions`) to avoid clipping.
- When no thumbnail exists, a placeholder with the filename is shown.
- Gallery rendering does not apply a hidden hard item cap.

## Timeline rendering

- Timeline renders as date-grouped sections, each with a group header (date label plus image/video count chips) and a justified mini-grid.
- The mini-grid within each group uses the same justified row algorithm as the gallery.
- Both gallery and timeline use `views::library::media_card()` for card rendering.
- Items are selectable with the same card style as gallery cards.
- Timeline rendering does not apply a hidden hard item cap.
- Timeline scrubber uses precomputed anchor metadata; it does not inspect rendered rows.
- Programmatic scrolling uses Iced absolute scrolling (`operation::scroll_to`) with relative fallback (`operation::snap_to`) from the same anchor-position model.

## Details pane

- Shows preview, filename, file info, tags, and actions as distinct sections.
- Sections are separated by horizontal dividers.
- File info shows human-readable metadata: type, size (KB/MB/GB), modified date, dimensions, path.
- Formatting is centralized in `format.rs` with `format_file_size`, `format_timestamp`, `format_dimensions`.
- Tags section uses chip-based add/edit/remove for app/game tags with deterministic colors.
- Tag chips distinguish inherited library tags from manual media tags, while keeping both visible in a compact layout.
- Actions section provides open, show-in-folder, copy-file, and copy-path commands.
- Details actions are responsive (single-column, 2x2 grid, or one-row depending available width) to prevent clipped buttons.
- Details pane scrollable uses embedded scrollbar spacing so scrollbar gutter does not overlap controls/text.

## Startup behavior

- On launch, app restores persisted state from storage and config.
- If library roots exist, the app auto-indexes and loads gallery/timeline projections.
- Startup restore is triggered via `Task::done(Message::StartupRestore)` from the init function.
- Runtime activity is shown in the expanded sidebar footer, without duplicate header status text.
- Startup restore also triggers a non-blocking GitHub release check after first render.

## Background activity

- Structured runtime activity belongs to application orchestration and is rendered in the expanded sidebar footer; the compact rail deliberately exposes only global actions.
- Real item totals support determinate feedback; work without a measured total uses a working treatment rather than a fabricated percentage.
- Startup readiness and deferred thumbnail catch-up remain distinct. Snapshot, indexing, projection/search, and thumbnail work retain their existing background scheduling and completion paths.

## Auto-refresh

- Gallery and timeline are auto-refreshed after indexing completes.
- Gallery is auto-refreshed after adding or removing a library root.
- Filesystem watching is active for root changes, and newly indexed files can trigger an in-app announcement dialog.
- Manual refresh/search/filter updates are also background-task driven so the UI thread remains responsive on large libraries.
- Update status checks re-run once every 24 hours while the app remains open.

## Size-based exclusion

- Min-size exclusion is configured in the Settings dialog alongside chip-based ignore-rule management.
- Users configure a minimum file size in KB with an Apply button in that section.
- When applied, the next indexing run skips files below the threshold.
- Files previously indexed that fall below the threshold are marked missing on re-index.
- The setting is session-local; config persistence is a future extension.

## Header branding

- Blue logo icon (from `assets/logo/blue/`) displays at 32×32.
- "Libra" displays in primary text color, "Pix" in accent color, creating a split-color product identity with minimal spacing.
- Header includes an About dialog entry point for project/creator context.
- GitHub link button uses a theme-aware icon and localized tooltip to open the project repository.
- The header follows the media-first neutral palette in Light, Dark, and System themes.

## Selection performance

- On media selection, the app first checks a preloaded `media_cache` (HashMap of read-model data).
- If the selected media is cached, details are loaded from memory without a storage roundtrip.
- The cache is populated during gallery/timeline projection builds.
- Only on cache miss does the app fall through to a storage query.

## UX goals

- Media-first browsing experience.
- Media-first, shadcn-inspired visual hierarchy using native Iced composition.
- Low-clutter desktop workflow.
- Coherent action placement.
- Product-oriented language throughout.
- Desktop-native interactions (double-click, folder picker, human-readable metadata).
- Filtering as lightweight chips, not admin forms.

### Final presentation refinements

- The expanded library pane shares its Browse heading row with the localized collapse control. Root rows remain content-sized.
- The Details edge centers its localized title as one word rotated 90 degrees. Iced 0.14's general Transformation only supports scale/translation, so the app-local label uses native geometry `Frame::rotate` and `fill_text` through Iced's `canvas` feature. Native paragraph measurements provide swapped layout bounds; widget operations expose the unmodified localized word. The existing full-height native button and localized Expand Details tooltip retain interaction ownership. There is no custom renderer, pre-rasterized text, or stacked-letter fallback.
- Filters use native `Row::wrap` with content-sized buttons instead of estimated wrapping against a fixed dialog width. Short choices retain usable padding; arbitrary names can wrap completely without overlapping neighboring hit areas.
- Managed tag removal accepts caller-provided localized tooltip text. Settings rules retain the existing deterministic per-pattern chip tones in both themes: pattern, status and actions share one compact line when wide and reflow at narrow widths. Decluttering changes density and hierarchy, not color identity; enabled/disabled rules keep their keyed tone and explicit status/actions. Input submit, edit/toggle/remove messages, diagnostics and scroll ownership are unchanged.
- About uses localized product and safety descriptions, a logo/name/version group, and creator metadata. No new actions, claims or links are introduced.
- Timeline year markers emphasize the current anchor's year, and the native slider uses a two-pixel neutral track and semantic handle states. The 168-pixel reserved panel and 96-pixel date-label lane remain unchanged to prevent viewport/cache churn or jumps on drag. The quiet date lane now sits beside the year/slider controls; its empty area is not a raised panel.

Official APIs checked: [Iced 0.14](https://docs.rs/iced/0.14.0/iced/), [native Text](https://docs.rs/iced_widget/0.14.2/iced_widget/text/type.Text.html), and installed upstream `iced_widget` 0.14.2 sources for `Column`, `Row::wrap`, `Scrollable`, `Responsive`, `Float`, and slider styling. App-local `advanced` and `canvas` features support the documented sizing/rotation adapters; only Iced's optional Lyon geometry dependencies were added, with no MSRV declaration or renderer change.
