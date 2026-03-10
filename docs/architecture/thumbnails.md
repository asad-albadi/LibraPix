# Thumbnail Architecture

Thumbnail generation is an app-owned, non-destructive cache subsystem.

## Boundary

- Generation logic lives in `librapix-thumbnails`.
- Indexing/application orchestration decides when to generate.
- Storage remains source of media metadata, and `derived_artifacts` records thumbnail readiness by variant.
- Source media files are read-only and never modified.

## Supported media

- **Images**: thumbnails generated via `image` crate with Lanczos3 resampling.
- **Videos**: thumbnails generated via `ffmpeg` command-line extraction (first frame at ~1 second).
- Cache key includes:
  - absolute source path
  - file size
  - modified timestamp
  - requested max edge size
- Cache location uses config-resolved app thumbnails directory.

## Generation behavior

- If a deterministic thumbnail path already exists, it is reused.
- For images: decodes source and writes a PNG thumbnail using `Lanczos3` filter for high-quality downsampling.
- For videos: runs `ffmpeg -ss 00:00:01 -frames:v 1 -vf scale=<max>:<max>:force_original_aspect_ratio=decrease` to extract a representative frame.
- Gallery thumbnails default to 400px max edge (`gallery-400`); detail previews default to 800px max edge (`detail-800`).
- Detail-size thumbnails are also reused by the in-app new-file modal dialog preview.
- Failures are counted for status output but do not abort full indexing flow.

## Artifact catalog status

Implemented:

- ready and failed thumbnail variants are written to `derived_artifacts`
- browse/search projections resolve gallery thumbnails from the artifact catalog

Partially implemented:

- detail thumbnails are still generated from the background projection flow instead of a dedicated thumbnail job subsystem
- startup browse-tier generation is now split into:
  - startup-priority thumbnails for the first visible slice
  - delayed background catch-up for the remaining browse-tier backlog
- background catch-up batches run more lightly than the startup-priority batches to reduce startup pressure

Deferred:

- deeper thumbnail pyramids beyond the current two named variants
- artifact cleanup/rebuild coordination as a separate runtime job family

## Video thumbnail requirements

- `ffmpeg` must be installed and available on the system PATH.
- If `ffmpeg` is not available, video thumbnails fail gracefully (placeholder shown in UI).
- No Rust dependency on ffmpeg bindings; extraction is via process invocation.
- **Windows**: The app invokes `ffmpeg.exe` explicitly. Paths are normalized to forward slashes before passing to ffmpeg, since ffmpeg accepts these on Windows and avoids backslash escaping issues.

## Ownership model

- Thumbnail files are app-managed cache data.
- Rebuilding is safe because thumbnails are derived data.
