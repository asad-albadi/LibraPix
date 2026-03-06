# Media UI Architecture

Librapix uses a centralized media-view architecture that underpins gallery, timeline, and search browsing.

## Unified data model

All browsing views consume `BrowseItem` which carries:
- `media_id`: stable identifier for selection and detail loading.
- `title`: display filename.
- `subtitle`: media kind and size or tag summary.
- `thumbnail_path`: resolved path to cached thumbnail (image or video).
- `aspect_ratio`: width/height ratio computed from stored dimensions (default 1.5 for unknown).
- `is_group_header`: flag for timeline date separators.

## Shared primitives

- `render_media_card(item, selected, height)`: renders a single media card at a given row height.
- `resolve_thumbnail(thumbnails_dir, row, max_edge)`: returns a thumbnail path for images (Lanczos3) or videos (ffmpeg), or None.
- `aspect_ratio_from(width_px, height_px)`: computes aspect ratio from nullable stored dimensions.
- `populate_media_cache(cache, rows, thumbnails_dir)`: caches read-model data and pre-resolved detail-size thumbnail paths for fast selection without I/O.

## Justified row layout

Gallery and timeline group grids use the same row-building algorithm:

1. The `responsive` widget provides available width at render time.
2. Items are accumulated into a row until the computed row height drops to or below the target (200px).
3. Each item receives `FillPortion` proportional to its `aspect_ratio`.
4. Row heights are clamped between 100px and 350px.
5. The last (possibly incomplete) row is capped at the target height to avoid stretching.

This produces a Google-Photos-style justified layout where images maintain aspect ratios and rows adapt to available width.

## View modes

### Gallery
- Renders a single flat justified grid of all non-header browse items.
- No group headers.
- Uses full projected item set (no hidden UI cap).

### Timeline
- Renders date-grouped sections.
- Each section has a group header (date label) followed by a justified mini-grid.
- Uses full projected item set (no hidden UI cap).
- Timeline mode includes a right-side fast scrubber driven by projection anchors.

### Search
- Renders search results as a justified grid using the same layout as gallery.
- Shown above the main browse content when a search query is active.

## Selection model

- Selection is app-level state (`selected_media_id`).
- On click, `load_media_details_cached()` checks `media_cache` first, then falls back to storage.
- Cache hits use pre-resolved `detail_thumbnail_path` — no disk I/O on the click path.
- Double-click opens the file in the OS default app.
- Selection changes do not trigger layout recomputation, only a visual update of the selected card border.

## Timeline scrubber model

- Scrubber anchors are precomputed from timeline projection buckets (`TimelineAnchor`).
- Anchor model includes date label + date parts + stable group index + normalized position.
- Scrub interactions map to nearest anchor index and trigger programmatic scroll through Iced scroll operations.
- While dragging, UI shows a floating date chip sourced from the active anchor label.
- Scrub interactions do not rebuild projections; they reuse cached timeline anchors.
