# Metadata Architecture

Metadata extraction is a read-only stage inside the indexing pipeline.

## Baseline extracted fields

- `media_kind` (`image` / `video`)
- `file_size_bytes`
- `modified_unix_seconds`
- image `width_px` / `height_px` when available
- `metadata_status`:
  - `ok`
  - `partial`
  - `unreadable`
  - (`missing` is reconciliation state in storage)

## Current extraction policy

- Extraction reads source files only; it never mutates media.
- Image dimensions are extracted through lightweight header parsing (`imagesize`).
- Video-specific duration and dimensions are deferred until a clean, well-documented crate decision is made.
- Extraction failures are recorded as `partial` or `unreadable` rather than aborting the whole indexing run.

## Deferred scope

- EXIF/advanced media metadata
- video duration/codec details
- thumbnail generation and derived visual features
