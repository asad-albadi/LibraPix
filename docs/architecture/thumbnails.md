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

## Reuse policy

Browse projection now follows this order:

1. exact valid `gallery-400` artifact row
2. exact deterministic `gallery-400` file already present on disk
3. compatible `detail-800` artifact row as gallery fallback
4. compatible deterministic `detail-800` file for current visible-priority items
5. placeholder immediately, then background generation if nothing reusable exists

Important behavior:

- Browse/search cards do not require thumbnail generation before they can render.
- A compatible `detail-800` thumbnail is preferred over unnecessary browse-tier regeneration.
- Broken `ready` artifact rows are rejected when the recorded file is missing or the stored path is absent.
- Background thumbnail batches also check for compatible detail-tier fallback before generating a new gallery-tier file.

## Artifact catalog status

Implemented:

- ready and failed thumbnail variants are written to `derived_artifacts`
- browse/search projections resolve thumbnails from:
  - exact ready artifact rows
  - deterministic on-disk browse files
  - compatible detail-tier fallbacks when exact browse tier is unavailable
- startup/runtime logging records artifact lookup timing, exact reuse counts, fallback reuse counts, placeholder counts, scheduled-generation counts, and rejected-artifact reasons

Partially implemented:

- detail thumbnails are still generated from the background projection flow instead of a dedicated thumbnail job subsystem
- browse-tier generation is split into:
  - startup-priority background work for the first visible slice
  - delayed background catch-up for the remaining browse-tier backlog
- startup-ready no longer waits for any thumbnail batch to finish
- background catch-up batches run more lightly than the startup-priority batches to reduce runtime pressure
- reconcile/projection refresh requests cancel in-flight thumbnail work instead of waiting behind it

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
