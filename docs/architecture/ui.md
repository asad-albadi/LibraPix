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
- Button styles: primary (accent), subtle (transparent), action (card bg), nav (active/inactive), card (selection border).
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

## Gallery rendering

- Gallery uses a grid layout with configurable column count (default 4).
- Each card contains a thumbnail (cover-fit) and caption text.
- Selected cards show an accent-colored border.
- Empty grid slots are padded with invisible spacers.

## Timeline rendering

- Timeline items are rendered with date group headers and media rows.
- Each media row contains a thumbnail and metadata text.
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

## UX goals

- Media-first browsing experience.
- Fluent-inspired visual hierarchy.
- Low-clutter desktop workflow.
- Coherent action placement.
- Product-oriented language throughout.
- Desktop-native interactions (double-click, folder picker, human-readable metadata).
