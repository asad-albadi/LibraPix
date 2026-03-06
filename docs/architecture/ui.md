# UI Architecture

Librapix UI uses a Fluent-inspired design system with an app-shell layout.

## App shell baseline

- Top header with product identity and integrated pill-shaped search bar.
- Left sidebar with sectioned navigation (browse, library, indexing, ignore rules).
- Main media pane for gallery grid, timeline groups, and search result cards.
- Right details pane for preview, file info, tags, and actions.
- Details pane shows an intentional empty state when no media is selected.

## Design system

All visual presentation is centralized in `librapix-app/src/ui.rs`:

### Color palette
- Fluent-inspired neutral dark theme.
- Background hierarchy: base, layer, surface, card, hover, selected.
- Accent: Windows Fluent blue for primary actions and selection states.
- Text hierarchy: primary, secondary, tertiary, disabled.
- Semantic colors for success, warning, and dividers.

### Spacing and typography
- Spacing scale: 2xs through 2xl (2px to 32px).
- Typography: display, title, subtitle, section, body, caption sizes.
- Consistent padding and gap values across all surfaces.

### Component styles
- Button styles: primary (accent), subtle (transparent), action (card bg), nav (active/inactive), card (selection border), filter chip (pill radius, accent when active).
- Text input styles: search (pill radius) and field (standard radius) with focus accent border.
- Container styles: header, sidebar, details pane, cards, empty states, thumbnail placeholders, dividers.

### Layout helpers
- `section_heading()`: small-caps section label.
- `h_divider()`: thin horizontal divider line.

## Interaction model

- Selection is explicit app state (`selected_media_id`).
- Single click selects a media item and loads details.
- Double-click opens the media item in the OS default external app.
- Double-click detection tracks last-click media id and timestamp at app level.
- Gallery cards and timeline rows are clickable buttons with card styles.
- Search is triggered via Enter key in the header search bar.
- Root selection uses styled nav buttons with status dot indicators.
- Root management controls appear contextually when a root is selected.
- Library root addition supports native folder picker dialog via Browse button.
- Manual path input is available as a secondary flow.

## Filtering

- Media pane toolbar includes filter chips for type (All / Images / Videos) and extension (PNG, JPG, GIF, WEBP, MP4, MOV, etc.).
- Extension chip set adjusts based on active type filter: image extensions when type is Images, video extensions when type is Videos, both when All.
- Changing type resets the extension filter.
- Filters apply to gallery, timeline, and search projections simultaneously.
- Filter state lives in app state (`filter_media_kind`, `filter_extension`); presentation is in the media pane toolbar.
- Filter logic is applied at the app orchestration layer, not inside widgets.

## Media pane layout

- Media pane toolbar (title, refresh, item count, filter chips) is rendered outside the scrollable region.
- Only the browse content and search results scroll; the toolbar remains fixed at the top.
- This prevents the scrollbar from overlapping toolbar controls.

## Centralized media-view architecture

Gallery, timeline, and search views share a unified media-view architecture:

- **BrowseItem**: common data model with `media_id`, `title`, `subtitle`, `thumbnail_path`, `aspect_ratio`, `is_group_header`.
- **render_media_card()**: shared card rendering primitive used by all views.
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
- When no thumbnail exists, a placeholder with the filename is shown.

## Timeline rendering

- Timeline renders as date-grouped sections, each with a group header and a justified mini-grid.
- The mini-grid within each group uses the same justified row algorithm as the gallery.
- Both gallery and timeline use `render_media_card()` for card rendering.
- Items are selectable with the same card style as gallery cards.

## Details pane

- Shows preview, filename, file info, tags, and actions as distinct sections.
- Sections are separated by horizontal dividers.
- File info shows human-readable metadata: type, size (KB/MB/GB), modified date, dimensions, path.
- Formatting is centralized in `format.rs` with `format_file_size`, `format_timestamp`, `format_dimensions`.
- Tags section supports add/remove for app tags and game tags.
- Actions section provides open, show-in-folder, and copy-path commands.

## Startup behavior

- On launch, app restores persisted state from storage and config.
- If library roots exist, the app auto-indexes and loads gallery/timeline projections.
- Startup restore is triggered via `Task::done(Message::StartupRestore)` from the init function.
- Activity status indicator is shown in the header during restore.

## Background activity

- Activity status is tracked as a simple string in app state.
- When non-empty, a subtle accent-colored caption is shown in the header.
- Cleared when the active operation completes.
- Currently surfaces indexing and restore activity.

## Auto-refresh

- Gallery and timeline are auto-refreshed after indexing completes.
- Gallery is auto-refreshed after adding or removing a library root.
- Periodic file-system watching is deferred to a future phase.

## Size-based exclusion

- Min-size exclusion is part of the Exclusions/Ignores sidebar section (not Indexing).
- Users configure a minimum file size in KB with an Apply button alongside ignore rules.
- When applied, the next indexing run skips files below the threshold.
- Files previously indexed that fall below the threshold are marked missing on re-index.
- The setting is session-local; config persistence is a future extension.

## Header branding

- "Libra" displays in primary text color, "Pix" in accent color, creating a split-color product identity.
- A subtle "· Media Library" subtitle follows in tertiary text.
- The header maintains the Fluent-inspired dark theme aesthetic.

## Selection performance

- On media selection, the app first checks a preloaded `media_cache` (HashMap of read-model data).
- If the selected media is cached, details are loaded from memory without a storage roundtrip.
- The cache is populated during gallery/timeline projection builds.
- Only on cache miss does the app fall through to a storage query.

## UX goals

- Media-first browsing experience.
- Fluent-inspired visual hierarchy.
- Low-clutter desktop workflow.
- Coherent action placement.
- Product-oriented language throughout.
- Desktop-native interactions (double-click, folder picker, human-readable metadata).
- Filtering as lightweight chips, not admin forms.
