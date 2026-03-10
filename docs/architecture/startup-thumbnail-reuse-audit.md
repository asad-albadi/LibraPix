# Startup Thumbnail Reuse Audit

## Status

- Date: 2026-03-10
- Branch: `feat/catalog-first-architecture`
- Scope: why startup still schedules thumbnail work for media that already has reusable thumbnail data

## Why This Audit Exists

Startup snapshot restore is no longer the main blocker. Recent startup traces show the remaining perceived startup lag is dominated by thumbnail work that still happens before the app reports `startup.ready`.

This audit documents the current startup thumbnail path in code, explains why reuse is being missed or bypassed, and defines the corrected policy for this final branch pass.

## Current Startup Scheduling Path

The current startup thumbnail path is:

1. `do_projection_job(...)` in `crates/librapix-app/src/main.rs`
   - refreshes `media_catalog`
   - loads all catalog rows for the current filter
   - loads ready derived artifacts only for the exact browse variant `gallery-400`
   - builds browse items
   - marks every row without an exact ready `gallery-400` artifact row as a thumbnail candidate
2. `apply_projection_job_result(...)`
   - splits candidates into:
     - startup-priority items
     - deferred catch-up items
   - starts the startup-priority thumbnail batch immediately
3. `do_thumbnail_batch(...)`
   - calls `ensure_image_thumbnail(...)` / `ensure_video_thumbnail(...)`
   - those functions reuse an existing deterministic on-disk file if it already exists
   - otherwise they generate a new thumbnail and then write/repair the `derived_artifacts` row
4. `finalize_background_flow(...)`
   - only marks startup ready after the startup-priority thumbnail queue has drained

## Current Reuse Rules

### Exact browse reuse

Current projection-time browse reuse requires all of the following:

- a `derived_artifacts` row exists
- `artifact_kind = 'thumbnail'`
- `artifact_variant = 'gallery-400'`
- `status = 'ready'`
- the row is returned by `Storage::list_ready_derived_artifacts_for_media_ids(...)`

### Detail-tier compatibility

Current startup browse projection does not use `detail-800` as a compatible fallback for gallery cards, even though it is a usable larger thumbnail.

### On-disk deterministic reuse

Current startup browse projection does not trust the deterministic thumbnail file path directly.

The only place that currently reuses an existing deterministic thumbnail file without a matching catalog row is `do_thumbnail_batch(...)`, because `ensure_*_thumbnail(...)` checks whether the output file already exists before generating it.

## Exact Reasons Reuse Fails Or Is Bypassed

### 1. Startup projection trusts only exact ready catalog rows

`do_projection_job(...)` loads only exact `gallery-400` rows from `derived_artifacts`.

If the exact row is missing, startup projection assumes the item needs thumbnail work even when the deterministic thumbnail file already exists on disk.

Result:

- existing thumbnails can be present and valid
- startup still schedules thumbnail work for them
- reuse happens too late, inside thumbnail batches instead of at projection/apply time

### 2. Compatible fallback logic is too strict

Browse projection currently does not reuse `detail-800` for gallery cards when `gallery-400` is unavailable.

Result:

- a compatible already-generated thumbnail can exist
- startup still schedules new browse-tier work anyway
- this is especially wasteful when startup only needs a usable card image, not the preferred tier

### 3. Startup-ready still waits on thumbnail queue settlement

`apply_projection_job_result(...)` starts startup-priority thumbnail work immediately, and `finalize_background_flow(...)` only transitions to ready after the active thumbnail queue drains.

Result:

- first useful gallery can already be visible
- the app still does not report ready or feel finished
- video thumbnails are especially harmful here because ffmpeg work stretches the startup boundary

### 4. Reconcile/projection requests are still coupled to thumbnail activity

`request_reconcile(...)` and `request_projection_refresh(...)` currently treat `thumbnail_in_flight` as a blocker.

Result:

- later startup/runtime refresh requests can be delayed behind thumbnail work
- the thumbnail subsystem remains too central to overall responsiveness

### 5. Existing artifact rows are not validated broadly enough

Startup projection currently assumes a returned `ready` row is reusable without broader compatibility/fallback reasoning.

There is no startup-time policy that says:

- exact ready row missing -> try deterministic exact file
- exact unavailable -> try compatible ready fallback
- compatible fallback unavailable -> use placeholder and background generation

## Video Thumbnail Behavior

Video thumbnails are treated the same as image thumbnails in the startup-priority queue, but they are operationally more expensive because `ensure_video_thumbnail(...)` shells out to `ffmpeg`.

That is acceptable only as background work. It is not acceptable as a startup-ready dependency.

## Confirmed Root Causes For The Remaining Startup Lag

The remaining startup lag is caused by the combination of:

- exact-match-only browse reuse during projection
- deterministic on-disk thumbnails not being trusted early enough
- compatible fallback reuse being too narrow
- startup-ready waiting for startup-priority thumbnail batch completion

## Corrected Policy

### Reuse policy

Startup and route projection should follow this order:

1. exact valid `gallery-400` artifact
2. exact deterministic `gallery-400` file on disk, even if the artifact row is missing
3. compatible valid `detail-800` artifact as gallery fallback
4. compatible deterministic `detail-800` file for currently visible priority items
5. placeholder immediately
6. background generation only when no reusable exact or compatible thumbnail is available

### Startup boundary policy

Startup-ready should require only:

- shell render
- snapshot hydrate/apply
- reconcile
- current-surface projection

Startup-ready must not wait for any thumbnail batch, including visible-surface startup-priority work.

### Runtime honesty policy

After startup-ready:

- visible-surface thumbnails may continue loading as background work
- non-visible thumbnail catch-up remains deferred
- runtime activity continues to report thumbnail work honestly
- route switches or later refreshes must not become blocked on thumbnail batch completion

## Concrete Fix Plan

1. Add explicit projection-time thumbnail lookup instrumentation.
2. Reuse exact ready `gallery-400` artifacts immediately.
3. Reuse deterministic exact browse files immediately when they already exist on disk.
4. Reuse compatible `detail-800` artifacts as fallback instead of forcing browse-tier regeneration.
5. Use placeholders immediately for unresolved visible items.
6. Exclude reusable exact/fallback items from generation scheduling.
7. Mark startup ready after projection completes, before any thumbnail queue needs to finish.
8. Keep thumbnail batches background-only and uncouple later reconcile/projection requests from thumbnail activity.
9. Add regression tests covering:
   - exact reuse
   - compatible fallback reuse
   - placeholder-plus-background scheduling
   - startup-ready before thumbnail completion

## What May Still Legitimately Block Startup

The only legitimate startup blockers after this pass are:

- snapshot hydrate/apply required for initial state
- reconcile/index correctness for the current root set
- current-surface projection needed for first useful content

Thumbnail generation is explicitly not in that list.

## Implemented Outcome

Implemented from this audit:

- projection-time thumbnail lookup now reuses:
  - exact ready `gallery-400` artifact rows
  - deterministic on-disk `gallery-400` files
  - compatible `detail-800` artifact rows
  - deterministic `detail-800` fallback for visible-priority items
- unresolved visible items render placeholders immediately and then continue with background thumbnail work
- startup-ready is now recorded before any thumbnail batch needs to finish
- later projection/reconcile refreshes cancel thumbnail work instead of waiting behind it
- startup logs now record artifact lookup start/end timing, reuse counts, placeholder counts, scheduled-generation counts, rejected-artifact reasons, and video slow/failure events
