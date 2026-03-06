# Troubleshooting

## Auto refresh does not react to file changes

- Symptoms
  - Adding/modifying media files in active roots does not update gallery/timeline automatically.
  - Manual Index + Refresh still works.
- Affected area
  - Filesystem watch subscription and runtime message delivery.
- Likely cause
  - Filesystem events are detected, but the app does not receive the refresh message.
- Confirmed cause
  - The watcher worker used a blocking `std::sync::mpsc::recv()` inside an async Iced subscription stream.
  - This blocked runtime delivery of `Message::FilesystemChanged` even though events were detected.
- Resolution
  - Switched watcher event transport to async `iced::futures::channel::mpsc::unbounded`.
  - Replaced blocking `recv()` with `next().await`.
  - On `FilesystemChanged`, app now runs incremental indexing and refreshes gallery/timeline (and active search results).
- Prevention guidance
  - Avoid blocking std channels inside async subscription workers.
  - Use async stream/channel primitives for all Iced subscription event pipelines.

## Clipboard action fails on Linux

- Symptoms
  - Copy-path action reports failure while the app is otherwise healthy.
- Affected area
  - Media actions (clipboard integration).
- Likely cause
  - `xclip` command not installed on host OS.
- Confirmed cause
  - Baseline Linux clipboard flow invokes `xclip -selection clipboard`.
- Resolution
  - Install `xclip` package and retry copy action.
- Prevention guidance
  - Keep platform action dependencies documented and validate them in release notes.

## Dimensions not showing for previously indexed files

- Symptoms
  - Dimensions display as "—" in details panel for files that were indexed before the dimension extraction feature.
  - Newly indexed files show dimensions correctly.
- Affected area
  - Storage upsert SQL for indexed_media; indexer dimension extraction logic.
- Likely cause
  - The indexer originally skipped dimension extraction for unchanged files. Files indexed before dimensions were supported retained NULL width/height.
- Confirmed cause
  - `ON CONFLICT DO UPDATE SET width_px = excluded.width_px` replaced stored dimensions with NULL for unchanged files (fixed with COALESCE).
  - Indexer only extracted dimensions for new/changed files, never backfilling unchanged files with missing dimensions.
- Resolution
  - Storage upsert uses `COALESCE(excluded.width_px, indexed_media.width_px)` to preserve existing values.
  - Indexer now checks for missing dimensions on unchanged images and re-extracts them.
  - `IndexedMediaSnapshot` and `ExistingIndexedEntry` now carry `width_px`/`height_px` so the indexer can detect missing dimensions.
- Prevention guidance
  - Use COALESCE for nullable metadata fields in upsert statements.
  - When adding new metadata extraction, ensure backfill logic for existing records.

## First-click selection lag

- Symptoms
  - Clicking a thumbnail for the first time causes a visible stutter before details appear.
  - Subsequent clicks on previously-viewed items feel faster.
- Affected area
  - Media selection path, detail-size thumbnail resolution.
- Confirmed cause
  - `load_media_details_cached` called `resolve_thumbnail` (which runs `ensure_image_thumbnail` / `ensure_video_thumbnail` I/O) synchronously for the DETAIL_THUMB_SIZE on every cache hit.
  - This meant disk I/O happened in the click handler path even when the gallery thumbnail was already cached.
- Resolution
  - Detail-size thumbnail paths are now pre-resolved during projection builds (alongside gallery thumbnails) and stored in `CachedDetails.detail_thumbnail_path`.
  - `load_media_details_cached` reads the cached path directly without I/O.
- Prevention guidance
  - Keep the click/selection path free of disk I/O, network calls, or expensive computation.
  - Pre-compute expensive data during batch operations (projections, indexing), not during interactive handlers.

## Video thumbnails not showing

- Symptoms
  - Video files show placeholder instead of thumbnail in gallery/timeline.
- Affected area
  - Thumbnail pipeline (video extraction).
- Likely cause
  - `ffmpeg` is not installed or not in the system PATH.
- Resolution
  - Install `ffmpeg` and ensure it's available in PATH: `brew install ffmpeg` (macOS), `apt install ffmpeg` (Linux), or download from ffmpeg.org (Windows).
  - Re-index library to generate video thumbnails.
- Prevention guidance
  - Video thumbnails are optional; the app degrades gracefully to placeholder display.
