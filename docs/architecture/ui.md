# UI Architecture

Librapix UI uses an app-shell layout with centralized styling helpers.

## App shell baseline

- Top header with title, subtitle, and integrated search.
- Left sidebar for navigation and library/indexing/ignore management.
- Main media pane for gallery/timeline/search results.
- Right details pane for metadata, tags, and actions.
- Details pane is segmented into:
  - selected media summary/preview
  - metadata section
  - tags section
  - actions section

## Styling centralization

- `librapix-app/src/ui.rs` contains reusable layout and visual tokens:
  - spacing
  - typography sizes
  - panel/card wrappers
  - shell region sizing
- UI components consume these helpers to keep visual rhythm consistent.

## Interaction model

- Selection is explicit app state (`selected_media_id`).
- Search, gallery, and timeline items all route into the same details/actions pane.
- Root/indexing controls remain outside widget internals and flow through app orchestration.

## UX goals

- media-first browsing
- clear visual hierarchy
- low-clutter desktop workflow
- coherent action placement
