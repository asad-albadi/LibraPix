# Indexing Architecture

Indexing is a dedicated subsystem (`librapix-indexer`) isolated from UI rendering.

## Baseline decisions

- Indexing reads source media metadata in read-only mode.
- Ignore rules are applied before metadata extraction.
- Index data is stored in Librapix-managed storage only.
- Indexing events are consumed by search and presentation layers through explicit application flow.
- Missing source files are expected operationally and must be handled as state transitions, not destructive actions.

## Baseline components

- Source root selection from storage (`active` lifecycle roots only; registered/edited via unified library dialog)
- Ignore matcher via centralized `IgnoreEngine` and glob rules
- Size-based exclusion via `ScanOptions.min_file_size_bytes` (skips files below threshold)
- Filesystem traversal with recursive walk
- Media-kind detection by supported extension set
- Candidate writer to app-managed `indexed_media` table
- Missing-root reconciliation delegated to storage lifecycle updates
- Metadata extraction stage:
  - file size and modified timestamp
  - image dimensions when available
  - extraction status (`ok` / `partial` / `unreadable`)
- Incremental strategy:
  - compare by `absolute_path + file_size_bytes + modified_unix_seconds`
  - classify `new`, `changed`, `unchanged`
  - re-extract dimensions for unchanged images with missing width/height
  - mark non-seen files under scanned roots as `missing`

## Baseline pipeline

1. Reconcile source-root availability.
2. Load eligible roots.
3. Load enabled ignore rules.
4. Scan filesystem and filter ignored entries.
5. Exclude files below configured minimum size (if set).
6. Detect incremental change class (`new` / `changed` / `unchanged`).
6. Extract baseline metadata for new/changed entries.
7. Persist/upsert candidates and mark missing files for scanned roots.
8. Ensure media-kind tags are attached (`kind:image`, `kind:video`).
9. Ensure root-level auto-tags (configured in library dialog) exist in the tags table and apply them to media under their root.
10. Query read-model rows for verification or downstream browsing/search surfaces.

## Execution model

Indexing runs as background work via `Task::perform`, keeping the UI thread responsive.
The `do_background_work` function opens its own `Storage` connection and runs the full pipeline (scan, index, thumbnails, projections) off the main thread.
Results are applied atomically to app state via `BackgroundWorkComplete`.
The same worker also supports projection-only refresh mode (no filesystem scan/index writes) for search/filter/manual refresh so large read-model work does not block UI updates.
When triggered by filesystem watch events, app orchestration compares previous/current media ids to surface new-file modal announcements (preview + metadata + actions) without mutating source files.

## Query limits

Indexing-side projection/thumbnail/search hydration now uses `list_all_media_read_models()` (no SQL `LIMIT`)
to avoid silently truncating aggregate multi-root/media-kind browse behavior.

Bounded APIs (`list_media_read_models(limit, offset)`) remain available for explicit pagination call sites.

No indexing logic should be embedded inside view widgets.
