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
- For videos: runs `ffmpeg -nostdin -hide_banner -loglevel error -ss 00:00:01 -frames:v 1 -vf scale=<max>:<max>:force_original_aspect_ratio=decrease` to extract a representative frame.
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

## Runtime policy

- Startup-ready still does not wait for any thumbnail batch.
- Missing browse thumbnails render as placeholders immediately.
- Visible image thumbnails may run in a lightweight startup-priority background queue after ready.
- Visible video thumbnails are now deferred into slower background catch-up instead of joining the first post-ready burst.
- Background catch-up batches are shaped differently for images and videos:
  - images can move in small batches
  - videos are throttled to one item per batch
- Route/projection refreshes cancel queued thumbnail work and invalidate stale in-flight batches.
- In-flight video extraction is cancellation-aware while waiting on ffmpeg.
- Thumbnail batch completion is applied as soon as `ThumbnailBatchComplete` reaches the app update loop; it is no longer indirectly dependent on later timer wakes.

## Failure policy

- Failed items now enter session backoff and are not immediately retried on the next projection refresh.
- ffmpeg resolution failures and ffmpeg spawn failures disable repeated video attempts for the rest of the session.
- Corrupt/problem media keep their placeholder and stop triggering aggressive retry loops.
- Failed artifacts are still written to `derived_artifacts`, but scheduling correctness now depends on runtime backoff rather than blindly retrying every failed row on every refresh.

## Artifact catalog status

Implemented:

- ready and failed thumbnail variants are written to `derived_artifacts`
- browse/search projections resolve thumbnails from:
  - exact ready artifact rows
  - deterministic on-disk browse files
  - compatible detail-tier fallbacks when exact browse tier is unavailable
- startup/runtime logging records artifact lookup timing, exact reuse counts, fallback reuse counts, placeholder counts, scheduled-generation counts, and rejected-artifact reasons
- startup/runtime logging now also records:
  - thumbnail batch dispatch/start/end/cancel timing
  - thumbnail batch dispatch-to-UI and message-received timing
  - slow completion-to-receive handoff warnings
  - thumbnail apply start timing
  - thumbnail apply duration on the app state side
  - result-message rate during active thumbnail work
  - route/projection refresh pressure while thumbnails are active
  - video failure command/exit/timeout/stderr details
  - session disable and item backoff decisions

Partially implemented:

- detail thumbnails are still generated from the background projection flow instead of a dedicated thumbnail job subsystem
- browse-tier generation is split into:
  - startup-priority background work for the first visible slice
  - delayed background catch-up for the remaining browse-tier backlog
- startup-ready no longer waits for any thumbnail batch to finish
- background catch-up batches run more lightly than the startup-priority batches to reduce runtime pressure
- reconcile/projection refresh requests cancel queued thumbnail work and invalidate stale in-flight work instead of waiting behind it

Deferred:

- deeper thumbnail pyramids beyond the current two named variants
- artifact cleanup/rebuild coordination as a separate runtime job family

## Video thumbnail requirements

- `ffmpeg` must be installed and available on the system PATH.
- If `ffmpeg` is not available, video thumbnails fail gracefully (placeholder shown in UI).
- No Rust dependency on ffmpeg bindings; extraction is via process invocation.
- **Windows**: The app invokes `ffmpeg.exe` explicitly. Paths are normalized to forward slashes before passing to ffmpeg, since ffmpeg accepts these on Windows and avoids backslash escaping issues.
- Video extraction now uses a bounded timeout and logs the resolved ffmpeg path plus command details when failures occur.

## Ownership model

- Thumbnail files are app-managed cache data.
- Rebuilding is safe because thumbnails are derived data.
