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
- Gallery cards and timeline rows are clickable buttons with card styles.
- Search is triggered via Enter key in the header search bar.
- Root selection uses styled nav buttons with status dot indicators.
- Root management controls appear contextually when a root is selected.

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
- Tags section supports add/remove for app tags and game tags.
- Actions section provides open, show-in-folder, and copy-path commands.

## UX goals

- Media-first browsing experience.
- Fluent-inspired visual hierarchy.
- Low-clutter desktop workflow.
- Coherent action placement.
- Product-oriented language throughout.
