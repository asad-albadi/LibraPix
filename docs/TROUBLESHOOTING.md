# Troubleshooting

## "All" filter misses videos or only shows part of the library

- Symptoms
  - "All" can appear image-only while "Videos" still shows video files.
  - Gallery/timeline can show only a subset of indexed media across registered roots.
- Affected area
  - App browse pipeline (read-model hydration + projection input), gallery/timeline rendering.
- Confirmed cause
  - Hidden truncation was applied in multiple layers:
    - Gallery rendering used `.take(120)` before layout.
    - Timeline rendering capped processing to `min(200)` items.
    - Browse/index/search hydration paths used paginated reads with a hard upper bound.
  - When recent images dominated earlier slices, videos and older media were pushed out of "All".
- Resolution
  - Removed gallery and timeline UI caps so projected items are fully renderable.
  - Added storage API `list_all_media_read_models()` (no SQL `LIMIT`).
  - Updated browse/index/search hydration paths to use the unbounded read-model API.
  - Added regression tests for:
    - unbounded read-model retrieval including older video rows
    - recursive multi-root indexing across deeply nested folders
- Prevention guidance
  - Avoid hidden hard caps in aggregate browse surfaces.
  - If pagination is needed for performance, make it explicit and user-visible.
  - Keep correctness tests for "All includes images+videos" and deep multi-root recursion.

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

## App freezes or shows "Not Responding" on startup (Windows)

- Symptoms
  - App window appears but becomes unresponsive ("Not Responding") for seconds to minutes while indexing runs.
  - Especially noticeable with large libraries or multiple roots containing thousands of images.
- Affected area
  - Startup restore path, indexing, thumbnail generation, projection builds.
- Confirmed cause
  - `StartupRestore` handler called `run_indexing`, `run_gallery_projection`, and `run_timeline_projection` synchronously inside the `update` function, blocking the UI thread for the entire duration of filesystem scanning, SQLite writes, thumbnail generation, and projection computation.
- Resolution
  - All heavyweight startup work now runs via `Task::perform` on a background thread.
  - The UI renders immediately with persisted state, and background work results are applied asynchronously via `BackgroundWorkComplete` message.
  - `FilesystemChanged`, `RunIndexing`, `ApplyMinFileSize`, `AddRoot`, and auto-tag operations also use the async path.
- Prevention guidance
  - Never perform blocking I/O (filesystem, SQLite, thumbnail generation) inside the `update` function.
  - Use `Task::perform` for any work that takes more than a few milliseconds.
  - Keep the click/update path free of synchronous heavy operations.

## "All" filter shows only images, not videos

- Symptoms
  - Clicking "All" shows only images; videos appear only when "Videos" filter is selected.
- Affected area
  - Read-model query ordering and limits.
- Confirmed cause
  - Query used `ORDER BY modified_unix_seconds DESC` with a 50k limit. When images vastly outnumber videos and have more recent timestamps, the top 50k by date were all images.
- Resolution
  - Read-model query now uses per-root and per-media-kind caps: up to 10k items per root, and up to 5k images and 5k videos per root. This guarantees both kinds appear in "All" when both exist.
- Prevention guidance
  - For "All" browse mode, ensure query design balances representation across media kinds and roots.

## Gallery or timeline shows media from only one or two libraries

- Symptoms
  - Multiple library roots are registered, but gallery/timeline appear to show media from only one or two of them.
- Affected area
  - Read-model query ordering.
- Confirmed cause
  - Query used `ORDER BY absolute_path ASC` with a 50,000-row limit. Paths sort alphabetically, so roots whose paths sort first (e.g. `C:\A\...` before `C:\B\...`) filled the limit before media from other roots appeared.
- Resolution
  - Query now uses ROW_NUMBER() with PARTITION BY source_root_id to cap at 10,000 items per root, then orders by modified_unix_seconds DESC. This guarantees all active roots are represented in the 50k result set.
- Prevention guidance
  - Ordering for unified multi-library views should prioritize recency or interleaving, not alphabetical path order.

## Gallery or timeline shows only a subset of media from multiple libraries

- Symptoms
  - Only a fraction of indexed media appears in gallery or timeline views.
  - Adding more library roots does not increase visible media proportionally.
- Affected area
  - Read-model query limits, gallery projection limits, thumbnail generation limits.
- Confirmed cause
  - Hard-coded query limits truncated results:
    - `list_media_read_models(200, 0)` during thumbnail generation — only 200 images got thumbnails.
    - `list_media_read_models(500, 0)` for projections — only 500 items in timeline/gallery source data.
    - `GalleryQuery.limit: 120` — gallery display truncated to 120 items regardless of how many matched.
    - `list_media_read_models(200, 0)` for search — only 200 items searchable.
- Resolution
  - All query limits increased to 50,000 (`MEDIA_QUERY_LIMIT`), effectively removing artificial truncation.
  - Gallery display limit also uses `MEDIA_QUERY_LIMIT`.
- Prevention guidance
  - Do not hard-code low query limits for aggregate views.
  - When limits are needed for performance, make them configurable or document them clearly.
  - Multi-library aggregation is a core product requirement; limits must not silently exclude data.

## Video thumbnails not showing on Windows

- Symptoms
  - Video files show placeholder instead of thumbnail in gallery/timeline/details.
  - Images show thumbnails correctly.
- Affected area
  - Video thumbnail generation via ffmpeg subprocess.
- Likely cause
  - ffmpeg not in PATH when app is launched from Explorer/Start Menu.
  - Path format (backslashes) causing ffmpeg to fail on Windows.
- Resolution
  - App now uses `ffmpeg.exe` explicitly on Windows.
  - Paths are normalized to forward slashes before passing to ffmpeg (ffmpeg accepts these on Windows).
- Prevention guidance
  - Install ffmpeg and add to system PATH, or ensure it is in PATH for GUI-launched apps.
  - See "Video thumbnails not showing" below for general ffmpeg requirements.

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

## Intermittent missing media in "All" / multi-root browse views

- Symptoms
  - "All" sometimes appeared to miss videos or showed disproportionate results from some roots.
  - With large libraries, users reported that visible items did not always match expectations.
- Affected area
  - Read-model query strategy, projection inputs, and diagnostics visibility.
- What we tried so far
  - Increased aggregate browse/search/projection limits to `50,000` (`MEDIA_QUERY_LIMIT`) to remove low truncation ceilings.
  - Introduced per-kind balancing (images/videos) to force representation in "All".
  - Introduced per-root balancing to force multi-root representation.
  - Added a diagnostics panel in the sidebar (counts + filter state + status).
  - Added an event log in diagnostics to show processed app messages with timestamps.
- Current confirmed findings
  - Indexer traversal is recursive across nested folders (`WalkDir::new(...).into_iter()` with no `max_depth`), so deep subfolder depth is not currently capped by scan logic.
  - Per-kind/per-root balancing logic was removed again to avoid artificial shaping of results; browse now uses straightforward ordering plus global `LIMIT/OFFSET`.
  - Missing items are more likely explained by filter state, ignore rules, eligibility/lifecycle of roots, min-size threshold, or media-type recognition than by shallow directory traversal.
- Resolution status
  - Partial: observability improved (diagnostics + event log), and hard low limits were removed.
  - Ongoing: continue validating root eligibility, ignore matches, and filter/min-size configuration against user datasets.
- Prevention guidance
  - Keep diagnostics enabled when changing query/indexing behavior.
  - Prefer explicit instrumentation over heuristic query shaping when debugging cross-root/media-kind visibility.
  - When introducing balancing logic, document tradeoffs and verify it does not hide real underlying causes.
