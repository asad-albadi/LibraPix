# Catalog-First Architecture Checklist

- Branch: `feat/catalog-first-architecture`
- Source baseline: repository default branch `master` (the repo does not currently have a `main` branch)

## Milestones

- [x] Architecture-doc milestone
- [x] Storage/catalog milestone
- [ ] Orchestration milestone
- [x] Timeline milestone
- [x] Thumbnail milestone
- [x] Search/projection milestone
- [ ] UI adaptation milestone
- [x] Validation milestone
- [x] Documentation completion milestone

## Current Slice

- Implemented: branch creation from the reconciled default branch baseline.
- Implemented: architecture plan written before significant refactor work.
- Implemented: dedicated startup completion audit documenting the current runtime critical path, remaining eager work, and the corrected startup policy for finishing this branch.
- Implemented: additive migration `0009_catalog_first_foundation.sql` introducing `media_catalog` and `derived_artifacts`.
- Implemented: additive compatibility migrations `0010_projection_snapshots.sql` and `0011_catalog_history_reconciliation.sql` so existing real databases from the older runtime line receive both projection-snapshot support and catalog/artifact tables.
- Implemented: storage catalog materialization and derived-artifact query APIs.
- Implemented: background browse/search/timeline preparation now consumes catalog rows.
- Implemented: persisted timeline keys and named thumbnail variants (`gallery-400`, `detail-800`).
- Implemented: startup/runtime regression fix for the catalog-first branch:
  - restored staged startup activity transitions (`snapshot hydrate -> reconcile -> projection -> thumbnail batches`)
  - removed the silent monolithic `BackgroundWorkComplete` flow
  - kept ready-state transitions honest until all staged work is idle
- Implemented: targeted regression tests for startup activity state, ready-state finalization, and thumbnail-stage handoff.
- Implemented: startup runtime completion pass now distinguishes startup-critical work from deferred catch-up.
- Implemented: startup bootstrap no longer opens storage or runs migrations before the first render.
- Implemented: startup instrumentation now writes a timestamped log file and records bootstrap/storage/snapshot/reconcile/projection/thumbnail timings plus first-usable/startup-ready milestones.
- Implemented: startup log placement now supports nearby `logs/` directories for dev/portable runs and falls back to a platform-appropriate app log directory otherwise.
- Implemented: the persisted startup snapshot now stores only a bounded recent-gallery slice (`projection_snapshots.version = 2`) instead of full gallery+timeline browse state.
- Implemented: legacy broad startup snapshots are discarded and rebuilt on the next compatible projection refresh instead of being eagerly rehydrated.
- Implemented: startup projection refresh prioritizes the current surface and bounds startup cache warm-up to a visible slice.
- Implemented: startup browse-tier thumbnail work is now split into startup-priority items plus delayed background catch-up.
- Implemented: ready state no longer waits for the full browse-tier thumbnail backlog.
- Implemented: projection-time thumbnail lookup now reuses exact `gallery-400` artifacts, deterministic on-disk browse thumbnails, and compatible `detail-800` fallbacks before scheduling generation.
- Implemented: unresolved visible items now render placeholders immediately while thumbnail generation continues in background.
- Implemented: startup-ready no longer waits for any thumbnail batch, including startup-priority visible-slice work.
- Implemented: projection/reconcile refreshes now cancel thumbnail work instead of waiting for thumbnail batches to settle first.
- Implemented: startup logs now record thumbnail artifact lookup timing, exact/fallback reuse counts, placeholder counts, scheduled-generation counts, rejected-artifact reasons, and video slow/failure events.
- Implemented: post-ready thumbnail runtime now separates image and video policy so visible videos defer into slower catch-up instead of joining the first post-ready burst.
- Implemented: background thumbnail batches now throttle video work to one item per batch and make in-flight video extraction cancellation-aware.
- Implemented: failed thumbnail items now enter runtime backoff, and ffmpeg resolution/spawn failures disable repeated video attempts for the rest of the session.
- Implemented: thumbnail runtime logs now record batch dispatch/start/end/cancel timing, apply timing, refresh pressure during thumbnail work, result-message rate, and video command/exit/timeout/stderr details.
- Implemented: route switches can trigger deferred surface refresh when startup intentionally leaves a non-visible surface for later.
- Implemented: stale delayed startup-reconcile ticks are ignored after their due timestamp is cleared, preventing duplicate startup scan/projection loops.
- Implemented: targeted regression coverage now covers deferred thumbnail catch-up readiness, startup route deferral, startup thumbnail prioritization, exact thumbnail reuse, compatible fallback reuse, placeholder scheduling, video non-blocking startup, video deferral, session disable/backoff, thumbnail batch cancellation, and projection refresh cancellation of thumbnail work.
- Implemented: final verification loop (`fmt`, `check`, `clippy`, `test`) passed after the startup completion + instrumentation changes.
- Implemented: startup smoke runs now confirm log-path emission, legacy snapshot discard, version-2 startup-snapshot persistence, and fast-path version-2 startup restore.
- Partially implemented: orchestration still enters through `crates/librapix-app/src/main.rs`, even though startup/catch-up boundaries are now materially cleaner.
- Partially implemented: UI adaptation is restored for activity/status visibility and duplicate header status was removed, but full interactive product validation on a real populated library still needs manual confirmation outside this terminal environment.
- Partially implemented: smoke validation from this environment can confirm build/run invocation only; populated-library GUI behavior still requires manual confirmation.
- Planned next: manual populated-library validation before any merge to the default branch.
