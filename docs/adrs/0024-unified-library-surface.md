# ADR 0024: Unified Library surface (single canonical projection)

## Status

Accepted

## Context

ADR 0008 (timeline/gallery projections) and ADR 0015 (media-view architecture) modeled Gallery and Timeline as two sidebar tabs backed by two projection passes: `project_gallery` (flat, modified-desc) and `project_timeline` (the same filtered set, date-bucketed + scrubber anchors). In practice they are the same media rendered two ways — a flat justified grid vs. the same items grouped by date with a scrubber. Maintaining two tabs and two filter/sort passes for one conceptual surface added duplication and an inconsistent mental model (the app is "a single home for media," Google/Windows-Photos style).

Key existing constraints to preserve: the justified-row layout math (`build_justified_row_layouts`), per-surface virtualization windows, the drag width-freeze, both layout caches, and the startup snapshot fast-path that restores a bounded flat gallery slice for a fast first frame.

## Decision

- Present Gallery + Timeline as one **Library** surface: a single sidebar entry plus a segmented **Grid | Timeline** toggle in the content header. The toggle reuses the existing `Message::OpenGallery` / `Message::OpenTimeline` (and `AppMessage::OpenGallery/OpenTimeline`), so the `Message` enum, `Route` enum, and `update()` flow are essentially unchanged.
- Run a single canonical projection: `project_timeline` produces the date buckets. From those buckets, the projection builds the grouped `timeline_items` (with per-day headers) and derives the flat `gallery_items` as the same items with headers stripped, in the same chronological order. The duplicate `project_gallery` filter/sort pass and its `GalleryQuery`/`GallerySort` use in `librapix-app` are removed.
- Rendering stays as-is per mode: the flat grid renders `gallery_items` (the existing flat renderer already skips group headers), the grouped timeline renders `timeline_items`; search results keep using the flat renderer. The scrubber shows only in Timeline mode.
- Layout caches (flat + grouped slots), virtualization, drag width-freeze, and the startup snapshot fast-path are kept byte-for-byte in behavior.

## Alternatives considered

- Keep two separate tabs and two full projection passes (status quo): rejected — duplicated work and a split mental model for one surface.
- Make `timeline_items` the only runtime feed and render the flat grid directly from it (drop `gallery_items` entirely): rejected for now — it would force reworking the startup snapshot fast-path (which persists a flat slice) and several tests, for little gain. Deriving `gallery_items` cheaply from the single timeline pass removes the duplicate computation while leaving the perf-critical startup path intact.
- Presentational-only merge (keep both projections, just merge the IA): rejected — it leaves the duplicate filter/sort pass in place; the single-projection derivation is the cleaner end state.

## Consequences

- One projection pass feeds both views; no duplicate filter/sort.
- The flat grid now orders by the timeline date key (capture/date) rather than file-modified time — intentional and consistent with the grouped view and with Photos-style apps.
- The IA is simpler (one Library entry + a mode toggle); `Route::{Gallery, Timeline}` is retained internally as the grid/timeline mode selector.
- This revises the surface/IA aspects of ADR 0008 and ADR 0015; the projection read-models (`TimelineBucket`, `TimelineAnchor`) and `librapix-projections` APIs are unchanged.
- Performance characteristics (layout math, virtualization, drag width-freeze, snapshot fast-path) are preserved.
