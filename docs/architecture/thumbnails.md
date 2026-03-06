# Thumbnail Architecture

Thumbnail generation is an app-owned, non-destructive cache subsystem.

## Boundary

- Generation logic lives in `librapix-thumbnails`.
- Indexing/application orchestration decides when to generate.
- Storage remains source of media metadata; thumbnail files are cache artifacts.
- Source media files are read-only and never modified.

## Baseline policy

- Supported now: image thumbnails only.
- Video thumbnails are deferred until a clean, documented extraction strategy is selected.
- Cache key includes:
  - absolute source path
  - file size
  - modified timestamp
- Cache location uses config-resolved app thumbnails directory.

## Generation behavior

- If a deterministic thumbnail path already exists, it is reused.
- If missing, Librapix decodes source image and writes a PNG thumbnail.
- Failures are counted for status output but do not abort full indexing flow.

## Ownership model

- Thumbnail files are app-managed cache data.
- Rebuilding is safe because thumbnails are derived data.
