# ADR 0015: Media-View Architecture, Justified Layout, and Video Thumbnails

## Context

Librapix had separate rendering paths for gallery, timeline, and search results. The gallery used a fixed 4-column grid with equal-width cells and forced cropping. Timeline used a completely different list-based rendering. Video thumbnails were not implemented. Dimensions were lost for unchanged files due to a storage SQL bug. Selection was slow because every click opened a new SQLite connection.

The Google Photos image grid article (Dan Schlosser, 2016) describes an effective approach: precomputed justified row layout where images maintain aspect ratios, rows adapt to available width, and layout computation is separated from rendering.

## Decision

### Centralized media-view architecture

Gallery, timeline, and search results now share a unified set of primitives:

- `BrowseItem` carries `aspect_ratio` computed from stored dimensions.
- `render_media_card()` is the shared card rendering function.
- `resolve_thumbnail()` handles both image and video thumbnails.
- `populate_media_cache()` caches read-model data for fast selection.
- `aspect_ratio_from()` provides a default 1.5 (landscape) when dimensions are unknown.

### Justified row layout

Gallery uses an adaptive justified layout inspired by Google Photos:

- Uses Iced `responsive` widget to obtain available width at render time.
- Greedy row-building: accumulate items until row height drops to target (200px).
- `FillPortion` proportional to aspect ratio distributes width correctly.
- Row heights clamped between 100px and 350px.
- Timeline groups use the same justified algorithm as mini-grids.

### Video thumbnails

- `ensure_video_thumbnail` in `librapix-thumbnails` extracts a frame via `ffmpeg`.
- Uses the same deterministic cache path mechanism as image thumbnails.
- `ffmpeg` is an optional system dependency; failures are graceful.

### Dimensions fix

- Storage upsert now uses `COALESCE(excluded.width_px, indexed_media.width_px)` to preserve existing dimensions when new values are NULL (unchanged files).

### Selection performance

- `media_cache` HashMap caches read-model data populated during projection builds.
- On selection, the cache is checked first; storage query is a fallback.

## Alternatives considered

- Full virtualization engine: deferred; the justified layout with 120-item limit is sufficient for now.
- Fixed-width column layout with aspect-ratio-aware heights: simpler but doesn't produce the clean justified-row look.
- `ffmpeg-next` Rust crate for video thumbnails: rejected in favor of simple command-line invocation to avoid heavy C dependency.

## Consequences

- Gallery visually resembles Google Photos with variable-height justified rows.
- Timeline feels like a sibling of gallery rather than a separate rendering system.
- Video thumbnails require `ffmpeg` on the system PATH.
- Selection is noticeably faster due to cached data.
- Dimensions display correctly for both new and previously-indexed files.
