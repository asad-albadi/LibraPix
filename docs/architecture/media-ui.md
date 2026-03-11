# Media UI Architecture

Librapix uses a centralized media-view architecture that underpins gallery, timeline, and search browsing.

## Unified data model

All browsing views consume `BrowseItem` which carries:
- `media_id`: stable identifier for selection and detail loading.
- `title`: display filename.
- `media_kind`: kind discriminator used for card icon badges and actions.
- `metadata_line`: compact card metadata string (`kind · size · dimensions`).
- `thumbnail_path`: resolved path to cached thumbnail (image or video).
- `aspect_ratio`: width/height ratio computed from stored dimensions (default 1.5 for unknown).
- `is_group_header`: flag for timeline date separators.

## Shared primitives

- `render_media_card(item, selected, height)`: renders a single media card at a given row height.
  - includes top-right media-kind badge icon.
  - includes padded metadata row below the thumbnail.
- `resolve_thumbnail(thumbnails_dir, row, max_edge)`: returns a thumbnail path for images (Lanczos3) or videos (ffmpeg), or None.
- `aspect_ratio_from(width_px, height_px)`: computes aspect ratio from nullable stored dimensions.
- `populate_media_cache(cache, rows, thumbnails_dir)`: caches read-model data and already-ready detail-tier artifact paths without forcing eager thumbnail generation during projection startup.

## Justified row layout

Gallery and timeline group grids use the same row-building algorithm:

1. The `responsive` widget provides available width at render time.
2. Items are accumulated into a row until the computed row height drops to or below the target (200px).
3. Each item receives `FillPortion` proportional to its `aspect_ratio`.
4. Row heights are clamped between 100px and 350px.
5. The last (possibly incomplete) row is capped at the target height to avoid stretching.

This produces a Google-Photos-style justified layout where images maintain aspect ratios and rows adapt to available width.

For large libraries, the shell no longer builds every justified row at once:
- Gallery, timeline, and search render only the current viewport plus overscan.
- Top/bottom spacer blocks preserve the full scroll extent, so correctness/completeness stays intact without forcing the UI thread to instantiate thousands of cards in one frame.
- Runtime logs record the large-surface render window as `interaction.surface_render.window`.
- Gallery and Timeline also cache justified layouts per surface and responsive width, so scrollbar-thumb drags do not rebuild full row math for every intermediate viewport update.
- The media viewport now has an explicit drag/settle lifecycle; active drag uses tighter overscan and suppresses per-position render-window logs, then the settled viewport restores the normal overscan and full diagnostics.

## View modes

### Gallery
- Renders a single flat justified grid of all non-header browse items.
- No group headers.
- Uses full projected item set (no hidden UI cap).
- When startup restores only the bounded snapshot slice first, the UI can stay immediately usable from that slice while a non-blocking gallery continuation refresh fills in the full logical gallery afterward.

### Timeline
- Renders date-grouped sections.
- Each section has a group header (date label plus image/video count chips) followed by a justified mini-grid.
- Uses full projected item set (no hidden UI cap).
- Timeline mode includes a right-side fast scrubber driven by projection anchors.
- Timeline rendering is viewport-bounded at two levels:
  - only intersecting date sections are considered
  - inside each intersecting section, only the visible justified rows plus overscan are composed
- Section-local spacer blocks preserve the full scroll extent even when a single date bucket contains hundreds or thousands of rows.
- Runtime logs record Timeline window diagnostics (`interaction.timeline_render.window`) including total groups/rows, visible groups/rows, first/last visible row, and spacer sizes so Windows render regressions remain measurable.

### Search
- Renders search results as a justified grid using the same layout as gallery.
- Shown above the main browse content when a search query is active.
- Search no longer truncates results to an implicit 20-hit cap; full matching set is returned for the current read-model snapshot.
- Top media-pane stats switch to the active search result set while a query is present (`Total`, `Images`, `Videos`).
- Search refresh runs through projection background work (`Task::perform`) instead of synchronous UI-thread query execution.

## Selection model

- Selection is app-level state (`selected_media_id`).
- On click, `load_media_details_cached()` checks `media_cache` first, then falls back to storage.
- Cache hits prefer pre-resolved `detail_thumbnail_path`; if the detail tier is not ready yet, selection/details fall back to the currently available browse thumbnail path.
- Double-click opens the file in the OS default app.
- Keyboard shortcuts are routed through ignored-key subscriptions to avoid text-input conflicts:
  - `Cmd/Ctrl+C` copies selected file
  - `Cmd/Ctrl+Shift+C` copies selected path
- Selection changes do not trigger layout recomputation, only a visual update of the selected card border.

## Timeline scrubber model

- Scrubber anchors are precomputed from timeline projection buckets (`TimelineAnchor`).
- Anchor model includes date label + date parts + stable group index + normalized position.
- Anchor normalized positions are structure-weighted from projection bucket sizes, so marker/scroll mapping reflects actual timeline distribution.
- Scrub interactions maintain a continuous normalized value; nearest anchor selection is derived from anchor normalized positions.
- Programmatic scrolling uses absolute scroll operations (`operation::scroll_to`) when viewport max offset is known, with relative fallback (`operation::snap_to`) during initialization.
- While dragging, UI shows a floating date chip sourced from the active anchor label.
- Date-chip vertical placement follows the live continuous scrub value so thumb and chip stay visually synchronized.
- Scrubber reserves a stable chip lane width before/after drag start so pointer-down does not shift slider position laterally.
- Year marker labels are placed on a position-aligned track using the same anchor normalized positions used for scroll targets.
- Scrub interactions do not rebuild projections; they reuse cached timeline anchors.

## New-file announcement UX

- Filesystem-triggered indexing deltas can open a modal in-app dialog for the newest new file.
- Modal layout is centered and constrained (max width/height) instead of stretching with window height.
- Dialog content includes preview thumbnail, compact metadata, and path/modification details.
- Quick actions: view/select, open file, copy file, dismiss.
- Dialog state is app-level (`new_media_announcement`) and remains outside card/grid rendering primitives.

## Runtime activity state

- Startup/runtime activity is product state, not a view-local spinner.
- The shell now reflects staged background work:
  - snapshot hydrate/apply
  - library reconcile/scan
  - gallery/timeline/search projection refresh
  - thumbnail batches
- Product placement is intentional: the structured activity panel lives in the left sidebar footer; the header does not duplicate the same runtime status text.
- Startup uses a ready-enough policy rather than a full-library preload policy:
  - current-route browse state is prioritized first
  - non-visible route refresh can remain deferred until later
  - startup thumbnail work is bounded to a first useful slice
- Deferred thumbnail catch-up can continue after the app is already usable.
- Activity text remains visible while real work is in flight, but startup readiness no longer waits for the full browse-tier thumbnail backlog.
