# Catalog-First Startup Completion Audit

## Status

- Document status: implemented for the final completion pass on `feat/catalog-first-architecture`
- Scope: startup/runtime policy on top of the existing catalog-first storage foundation
- Source of truth: current code in `crates/librapix-app/src/main.rs` and supporting storage/projection modules

## Why This Audit Exists

The catalog-first branch already restored honest runtime activity state, but startup still behaves too much like a full-library eager preload. The branch is technically correct, yet the product experience still front-loads more work than necessary before the app feels usefully interactive.

This audit documents the exact startup pipeline on the current branch, identifies what is still incorrectly eager, and defines the completion policy for the final runtime pass.

## Current Startup Sequence

The current branch now follows this sequence:

1. `main()` initializes timestamped process logging before the Iced runtime starts.
2. `bootstrap_runtime()` loads config/theme/path overrides only.
   - storage open, migrations, and root reconciliation are no longer on the pre-render path
3. `init()` returns `Task::done(Message::StartupRestore)`.
4. `StartupRestore`:
   - starts `start_snapshot_hydrate(app)`
   - starts the non-blocking release update check
5. `do_snapshot_hydrate(...)`:
   - opens storage and records storage-open + migration timings
   - seeds configured roots only when the database has no roots yet
   - reconciles root availability and loads source roots / ignore rules
   - loads the persisted startup snapshot payload when present
   - only accepts snapshot version `2`; incompatible legacy snapshots are discarded without broad rehydration
6. `apply_snapshot_hydrate_result(...)`:
   - replaces roots and ignore rules in app state
   - queues reconcile when hydrated roots exist
   - starts `begin_snapshot_apply(...)` when a compatible startup snapshot exists
7. `begin_snapshot_apply(...)` and `apply_snapshot_chunk(...)`:
   - clear browse/search/cache state
   - incrementally apply only the bounded recent gallery slice stored in the startup snapshot
   - restore available filter tags from that snapshot
   - do not rebuild timeline/search browse state from the snapshot
8. `continue_startup_after_snapshot_hydrate(...)`:
   - schedules the delayed startup reconcile only after the bounded snapshot apply is done
9. `request_reconcile(...)` / `do_scan_job(...)`:
   - reconcile root availability
   - scan eligible roots
   - apply incremental index writes
   - refresh root tags and statistics
10. `do_projection_job(...)`:
    - refreshes the full catalog
    - queries catalog rows
    - refreshes the current startup-critical surface first
    - bounds startup cache warm-up to a visible slice
    - prepares a bounded gallery startup snapshot payload for future launches when the projection is unfiltered
11. `apply_projection_job_result(...)`:
    - replaces gallery/timeline/search state
    - updates details/cache/filter state
    - persists the bounded startup snapshot payload
    - performs projection-time thumbnail reuse lookup before any generation is scheduled
    - marks startup ready after startup-blocking work settles, before thumbnail batches finish
    - splits thumbnail work into startup-priority background items and deferred catch-up
12. `do_thumbnail_batch(...)` and `apply_thumbnail_batch_result(...)`:
    - reuse exact browse-tier files or compatible detail-tier fallbacks before generating new browse-tier thumbnails
    - upsert ready/failed artifact rows
    - patch browse cards as thumbnails become ready
13. `finalize_background_flow(...)`:
    - marks startup ready once snapshot apply, reconcile, and current-surface projection settle
    - schedules deferred thumbnail catch-up after ready-enough instead of keeping the app in startup-busy state
    - cancels thumbnail work when later projection/reconcile refreshes need to take priority

## What Is On The Current Startup Critical Path

The effective startup critical path is now:

- shell render
- config load
- background storage open + migrations
- startup snapshot hydrate
- bounded startup snapshot apply (recent gallery slice only)
- startup reconcile / scan
- current-surface projection refresh

The path intentionally no longer includes:

- synchronous pre-render storage open
- full gallery snapshot rehydration
- any timeline snapshot rehydration
- startup-priority thumbnail batches
- full browse-tier thumbnail backlog drain before `Ready`

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

### 4. Snapshot apply was still broad

The remaining blocker was the persisted projection snapshot itself:

- it carried full gallery browse state
- it carried full timeline browse state
- startup eagerly deserialized and re-applied both before reconcile could continue

That broad snapshot payload was the last major reason `Loading library snapshot` still felt heavy on large libraries.

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
- bootstrap no longer opens storage or runs migrations before the first render
- startup logging now writes a timestamped log file with stage start/end timing, migration timing, counts, errors, first-usable-gallery, startup-ready, and deferred-catch-up milestones
- startup log placement now prefers a dev/portable `logs/` directory when appropriate and otherwise falls back to a platform-appropriate app log directory
- startup snapshot persistence is now a bounded gallery-only payload (`version = 2`) instead of a full gallery+timeline browse snapshot
- legacy broad snapshots are discarded and rebuilt after the next compatible projection refresh
- snapshot apply now restores only the recent gallery slice needed for early interaction instead of rebuilding both browse surfaces
- startup projection refresh is narrowed to the currently visible browse surface while startup is still incomplete
- startup cache warm-up is bounded to a visible slice instead of the full catalog
- stale delayed-startup reconcile ticks are now ignored after their due timestamp is cleared, preventing duplicate startup scan/projection loops
- startup browse-tier thumbnail work is split into:
  - startup-priority background items
  - delayed background catch-up
- browse projection now reuses thumbnails in this order:
  - exact `gallery-400` artifact rows
  - deterministic on-disk `gallery-400` files
  - compatible `detail-800` artifact rows
  - deterministic `detail-800` fallback for visible-priority items
- startup logging now records thumbnail artifact lookup start/end timing, exact/fallback reuse counts, placeholder counts, scheduled-generation counts, rejected-artifact reasons, and video slow/failure events
- startup-ready no longer waits for startup-priority thumbnail batches
- later projection/reconcile refreshes cancel thumbnail work instead of waiting for thumbnail batches to settle
- deferred thumbnail catch-up starts after startup ready-enough and runs in lighter batches
- route switches can request a deferred surface refresh when startup intentionally skipped the non-visible route
- no new splash screen was added in this pass; the existing staged activity UI remains the honest startup indicator after the critical path reduction

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
