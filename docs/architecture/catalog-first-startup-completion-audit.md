# Catalog-First Startup Completion Audit

## Status

- Document status: implemented for the final completion pass on `feat/catalog-first-architecture`
- Scope: startup/runtime policy on top of the existing catalog-first storage foundation
- Source of truth: current code in `crates/librapix-app/src/main.rs` and supporting storage/projection modules

## Why This Audit Exists

The catalog-first branch already restored honest runtime activity state, but startup still behaves too much like a full-library eager preload. The branch is technically correct, yet the product experience still front-loads more work than necessary before the app feels usefully interactive.

This audit documents the exact startup pipeline on the current branch, identifies what is still incorrectly eager, and defines the completion policy for the final runtime pass.

## Current Startup Sequence

The current branch follows this sequence:

1. `init()` returns `Task::done(Message::StartupRestore)`.
2. `StartupRestore`:
   - starts `start_snapshot_hydrate(app)`
   - marks startup reconcile as queued when roots exist
   - starts the non-blocking release update check
3. `do_snapshot_hydrate(...)`:
   - opens storage
   - loads source roots and ignore rules
   - loads the persisted projection snapshot when present
4. `apply_snapshot_hydrate_result(...)`:
   - replaces roots and ignore rules in app state
   - starts `begin_snapshot_apply(...)` when a compatible snapshot exists
   - otherwise continues startup immediately
5. `begin_snapshot_apply(...)` and `apply_snapshot_chunk(...)`:
   - clear browse/search/cache state
   - incrementally apply the full saved gallery snapshot
   - incrementally apply the full saved timeline snapshot
   - restore saved timeline anchors and available filter tags
6. `continue_startup_after_snapshot_hydrate(...)`:
   - schedules the delayed startup reconcile only after snapshot apply is done
7. `request_reconcile(...)`:
   - blocks on active snapshot apply / projection / thumbnail work
   - starts `do_scan_job(...)`
8. `do_scan_job(...)`:
   - reconciles root availability
   - loads ignore rules
   - scans every eligible root
   - applies incremental index writes
   - refreshes root tags and statistics
9. `apply_scan_job_result(...)`:
   - applies root/indexing summary updates
   - immediately requests a projection refresh
10. `do_projection_job(...)`:
    - refreshes the full catalog
    - queries all filtered catalog rows
    - derives available filter tags from all rows
    - builds the full gallery projection
    - builds the full timeline projection
    - optionally builds the search result set
    - hydrates the media cache for all rows
    - prepares a projection snapshot payload
    - schedules thumbnail work for every missing browse-tier thumbnail
11. `apply_projection_job_result(...)`:
    - replaces gallery/timeline/search state
    - updates details/cache/filter state
    - persists the new projection snapshot
    - starts thumbnail batches for the entire queued thumbnail set
12. `do_thumbnail_batch(...)` and `apply_thumbnail_batch_result(...)`:
    - generate or reuse browse-tier thumbnails in batches
    - upsert ready/failed artifact rows
    - patch browse cards as thumbnails become ready
13. `finalize_background_flow(...)`:
    - only sets `Ready` after reconcile, projection, and the thumbnail queue are all fully settled

## What Is On The Current Startup Critical Path

Today, the effective startup critical path is larger than it should be:

- shell render
- snapshot hydrate
- full snapshot apply
- startup reconcile / full-library scan
- full catalog refresh
- full projection shaping for gallery and timeline
- full media-cache warmup
- full browse-tier thumbnail queue drain before `Ready`

The shell is technically responsive because heavy work runs through `Task::perform`, but the product still feels heavy because the runtime policy treats deep catch-up as startup work.

## Non-Critical Work That Is Currently Too Eager

The following work is not required before the app is usefully interactive, but the current branch still performs it eagerly during startup:

- building the non-active browse surface during startup
  - gallery and timeline are both rebuilt even when only one route is immediately visible
- warming `media_cache` for the full catalog
  - details can already fall back to storage on selection
- scheduling browse-tier thumbnails for the full library as soon as projection completes
- waiting for the entire thumbnail backlog before reporting `Ready`
- broad snapshot application of both gallery and timeline before reconcile kickoff

## Root Causes Of Startup Lag

### 1. Startup has no explicit "ready enough" boundary

The coordinator now has honest staged activity, but it still does not separate:

- startup-critical work needed for first useful interaction
- background catch-up work that should continue after the app is already usable

As a result, `finalize_background_flow(...)` continues to treat deep thumbnail backlog as part of startup completion.

### 2. Projection work is still broader than the visible product surface

`do_projection_job(...)` refreshes the catalog correctly, but it also rebuilds:

- full gallery browse items
- full timeline browse items
- optional search items
- full cache hydration

That is correct data, but it is broader than the first visible surface needs during startup.

### 3. Thumbnail policy is still all-library eager

`start_thumbnail_batches(...)` currently queues every missing browse-tier thumbnail immediately after projection. That preserves correctness, but it front-loads CPU, disk, and ffmpeg/image work at the exact moment the shell should be becoming comfortably interactive.

### 4. Snapshot apply is still broad

The snapshot system is directionally correct and already chunked, but it still restores both browse surfaces before startup reconcile can begin. This is less severe than projection/thumb generation, but it still contributes to a broader-than-necessary startup path.

## Recommended Corrected Startup Policy

The final catalog-first startup policy should be:

### 1. Minimal shell first

- show shell quickly
- restore roots/config/basic state quickly
- show honest runtime activity immediately

### 2. Minimal useful content next

- hydrate and apply persisted browse state when available
- after reconcile, refresh only the currently visible browse/search surface needed for immediate interaction
- warm only the small amount of detail/cache state that helps early interaction

### 3. Ready enough earlier

The app should become "ready enough" when the following are complete:

- snapshot hydrate/apply required for first render
- reconcile/scan
- current-surface projection refresh

The app should not wait for full-library browse-tier thumbnail catch-up before becoming ready enough.

### 4. Deferred deep catch-up

After ready enough is reached:

- remaining browse-tier thumbnail backlog continues in background catch-up
- non-visible route preparation can be progressive or on-demand
- detail-tier thumbnails remain on-demand

### 5. Honest activity throughout

- startup-critical activity remains visibly busy while startup-critical work is in flight
- once the app is ready enough, later catch-up activity is still surfaced honestly
- background catch-up must not masquerade as blocking startup

## Exact Completion Plan For This Branch

### Implement in code

1. Introduce an explicit startup-ready state in the runtime coordinator.
2. Reduce startup projection breadth so it prioritizes the currently visible route and search surface instead of rebuilding every browse surface eagerly.
3. Stop warming the full media cache on startup when storage fallback already exists for details loading.
4. Split thumbnail scheduling into:
   - startup-priority work for the first useful slice
   - deferred background catch-up for the rest
5. Allow startup to reach ready-enough before deferred thumbnail backlog finishes.
6. Keep background activity honest while deferred catch-up continues.
7. Preserve catalog refresh correctness and non-destructive storage behavior.

### Add regression coverage

- startup-ready transition does not wait for deferred thumbnail backlog
- startup thumbnail prioritization is bounded and deterministic
- startup projection policy does not rebuild non-visible browse surfaces unnecessarily
- timeline/search still refresh correctly when explicitly requested

### Update docs

- checklist
- message-flow / media-ui / catalog-first architecture docs where needed
- troubleshooting entry for the startup-heavy runtime policy and its final resolution

## Implemented Outcome In This Pass

Implemented in code:

- startup now has an explicit ready-enough boundary
- startup projection refresh is narrowed to the currently visible browse surface while startup is still incomplete
- startup cache warm-up is bounded to a visible slice instead of the full catalog
- startup browse-tier thumbnail work is split into:
  - startup-priority items
  - delayed background catch-up
- deferred thumbnail catch-up starts after startup ready-enough and runs in lighter batches
- route switches can request a deferred surface refresh when startup intentionally skipped the non-visible route

Still intentionally true:

- catalog refresh remains a full correctness step after reconcile
- the coordinator still lives in `crates/librapix-app/src/main.rs`
- full interactive confirmation on a populated real library still requires a human GUI pass outside this terminal environment

## Intentional Non-Goals For This Pass

The final completion pass does not attempt:

- a full extraction of the coordinator out of `crates/librapix-app/src/main.rs`
- a new search engine or FTS migration
- a persisted aggregate timeline-bucket table
- a deep thumbnail-pyramid redesign beyond the existing named variants

Those remain valid future directions, but they are not required to finish this branch correctly.
