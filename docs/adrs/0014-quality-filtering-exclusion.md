# ADR 0014: Quality, Filtering, and Exclusion Milestone

## Context

Thumbnails appeared blurry, the media pane toolbar was partially hidden by the scrollbar, and no filtering or size-based exclusion existed.

## Decisions

### Thumbnail quality

- Switched from `image.thumbnail()` (nearest-neighbor) to `image.resize()` with `Lanczos3` filter.
- Included `max_edge` in the thumbnail cache key hash so different sizes produce distinct cache files.
- Increased gallery thumbnail size from 256px to 400px and detail preview from 512px to 800px for HiDPI.

### Media pane layout

- Moved the content toolbar (title, refresh, item count, filters) outside the scrollable region.
- Only browse content and search results scroll; the toolbar stays fixed.

### Filtering

- Type filter: `All` / `Images` / `Videos` as chip buttons in the media pane toolbar.
- Extension filter: common extensions as chip buttons, context-sensitive to active type.
- Filters apply to gallery, timeline, and search results through the app orchestration layer.
- Filter state: `filter_media_kind: Option<String>`, `filter_extension: Option<String>`.
- Changing type resets extension.

### Size-based exclusion

- Added `ScanOptions` struct to the indexer with `min_file_size_bytes`.
- Files below the threshold are skipped during scanning (counted as ignored).
- UI: text input in the sidebar indexing section with KB unit and Apply button.
- Session-local for now; config persistence is a future extension.

## Alternatives considered

- Extension filter as dropdown: rejected as more complex UI for little benefit.
- Size exclusion in storage queries: rejected; filtering at scan time is cleaner and prevents unwanted data from entering the index.
- Hardcoded size threshold: rejected per project rules; must be configurable.

## Consequences

- Old thumbnails are orphaned; they will be regenerated with better quality on next indexing.
- New `ScanOptions` parameter on `scan_roots` is a breaking change to the indexer API.
- Gallery projection `GalleryQuery` now has an `extension` field; all callers updated.
