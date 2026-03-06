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
  - Dimensions display as "—" in details panel for files that were indexed before the fix.
  - Newly indexed files show dimensions correctly.
- Affected area
  - Storage upsert SQL for indexed_media.
- Likely cause
  - The indexer sets width/height to NULL for unchanged files. The upsert SQL overwrites stored values.
- Confirmed cause
  - `ON CONFLICT DO UPDATE SET width_px = excluded.width_px` replaces stored dimensions with NULL for unchanged files.
- Resolution
  - Changed to `COALESCE(excluded.width_px, indexed_media.width_px)` to preserve existing values.
  - A re-index of affected files (modify + save, or delete and re-add root) will restore dimensions.
- Prevention guidance
  - Use COALESCE for nullable metadata fields in upsert statements where the incoming value may be intentionally absent.

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
