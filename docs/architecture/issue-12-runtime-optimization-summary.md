# Issue #12 Runtime Optimization Summary

GitHub issue: `#12`
Branch: `feat/catalog-first-architecture`
Scope: final runtime, rendering, and interaction behavior for the catalog-first branch.

This document is the final source of truth for issue `#12`. It consolidates the branch outcome into one coherent explanation and intentionally replaces the earlier audit-by-audit narrative.

## 1. Why This Work Was Needed

The catalog-first branch established the right storage and data-model direction, but the runtime still felt wrong on large real libraries.

The main problem was not correctness. The app could load the right data, but it still behaved too much like a full-library preload:

- startup felt heavier than the visible UI needed
- route and filter changes rebuilt more than the active surface needed
- large gallery and timeline surfaces could still stall after projection finished
- first-open details and scrollbar-thumb drags could still hit UI-thread work that felt hung
- failing video thumbnails still created noisy post-ready background pressure

Issue `#12` was therefore a runtime-optimization pass, not a feature rewrite.

## 2. User-Visible Symptoms Before The Fix

- `Loading library snapshot` still felt like broad startup work instead of a quick restore.
- `startup.ready` could stay late because thumbnail work was still treated like startup-critical work.
- Unchanged launches could either waste time on a redundant full gallery rebuild or leave the restored 160-card slice as the permanent gallery state.
- Opening Timeline, changing filters, or switching routes could feel hung even after projection finished quickly.
- First-open details could block on synchronous detail-thumbnail generation.
- Dragging the media scrollbar thumb on large surfaces could feel sticky and over-processed.
- Repeated failing video thumbnails could keep re-entering later projection generations.

## 3. Root Causes Discovered

The branch exposed five root-cause groups.

### Startup was still too eager

- broad snapshot history had to be discarded and replaced with a bounded gallery-only snapshot
- startup work still needed a clear ready-enough boundary
- unchanged startup still needed explicit gallery continuation after the fast snapshot restore

### Projection ownership was too broad

- post-startup refreshes rebuilt both Gallery and Timeline even when only one surface was visible
- startup projection still needed to prioritize only the current surface

### Thumbnail work stayed too central

- projection-time reuse was too strict and initially trusted only exact ready `gallery-400` artifact rows
- compatible `detail-800` fallback and deterministic on-disk reuse were not used early enough
- video failures could re-enter the next projection generation without backoff

### Rendering was still too expensive after projection

- large gallery/timeline/search surfaces still needed viewport-bounded rendering
- Timeline virtualization originally stopped at the date-group level, so one intersecting group could still build hundreds or thousands of justified rows

### Interaction hot paths still did too much work

- first-open details could still hit synchronous detail-thumbnail generation
- scrollbar-thumb drag still processed too many stale intermediate targets and let drag-time width/range churn invalidate layout work
- blocking timer subscriptions delayed thumbnail-completion messages before they reached `update`

## 4. Final Runtime / Rendering / Interaction Model After The Fix

### Startup model

- The shell renders first.
- `StartupRestore` hydrates a bounded version-2 startup snapshot in the background.
- Snapshot apply restores only a recent gallery slice plus filter-tag metadata.
- Reconcile runs after snapshot apply.
- Startup projection refreshes only the current surface.
- `startup.ready` is reached when snapshot apply, reconcile, and current-surface projection settle.
- Thumbnail work continues honestly in background after ready.
- If unchanged startup restored the default gallery snapshot, the app skips the redundant blocking gallery rebuild, becomes ready, and schedules a non-blocking gallery continuation so the 160-card slice is not permanent.

### Projection and interaction model

- Post-startup route, filter, search, and filesystem refreshes are current-surface-first.
- The non-visible surface is deferred until the user opens it.
- First-open details stay placeholder-first:
  - reuse existing `detail-800` when present
  - otherwise reuse the already-visible browse thumbnail
  - never synchronously generate a detail thumbnail on the selection path

### Thumbnail model

- Projection-time browse reuse now follows this order:
  1. exact ready `gallery-400`
  2. deterministic on-disk `gallery-400`
  3. compatible ready `detail-800`
  4. deterministic `detail-800` fallback for visible-priority items
  5. placeholder plus background generation
- Startup-ready does not wait for any thumbnail batch.
- Visible videos are deferred into slower background catch-up instead of the first startup-priority burst.
- Failed items enter runtime backoff.
- ffmpeg resolution/spawn failures can disable repeated video attempts for the session.
- Timer subscriptions now use `iced::time::every(...)`, so thumbnail completion messages are forwarded promptly.

### Rendering and drag model

- Gallery, Timeline, and Search render through viewport-bounded windows with spacer preservation.
- Timeline virtualizes rows inside each intersecting date section instead of rendering the whole section.
- Scrollbar-thumb drag uses an explicit drag/settle lifecycle:
  - latest-only preview targets
  - cadence-capped preview applies
  - frozen drag-time layout width
  - frozen drag-time effective `max_y`
  - exact final settle apply

## 5. Major Fixes Implemented

- Moved storage open and migrations off the pre-render bootstrap path.
- Replaced broad startup snapshot restore with a bounded gallery-only snapshot (`projection_snapshots.version = 2`).
- Added an explicit startup ready-enough boundary.
- Added non-blocking gallery continuation after unchanged snapshot-backed startup.
- Switched projection refresh policy to current-surface-first.
- Removed synchronous detail-thumbnail generation from first-open selection.
- Added viewport-bounded rendering for large browse surfaces.
- Added row virtualization inside large Timeline date groups.
- Added projection-time exact/fallback thumbnail reuse and placeholder-first browse behavior.
- Split thumbnail work into startup-priority and deferred background catch-up, with heavier throttling for videos.
- Added runtime backoff/session-disable policy for repeated thumbnail failures.
- Replaced blocking timer subscriptions with `iced::time::every(...)`.
- Reworked thumb-drag handling to use latest-only preview plus stable drag-time layout inputs.

## 6. Before vs After Comparison

| Area | Before | After | Evidence |
|---|---|---|---|
| Startup first usable gallery | Startup restore still felt like a broad preload and did not have a small committed fast-path metric in the older docs set. | A 160-item snapshot gallery becomes visible in `943ms` and `1346ms` on committed Windows logs, with `6416` total gallery items behind it. | `logs/librapix-startup-20260310-185811-16788.log`, `logs/librapix-startup-20260310-185012-14648.log` |
| Startup ready boundary | `startup.ready` still waited on startup-priority thumbnail work. | `startup.ready` lands at `2087ms` and `2502ms` while `13` thumbnails remain deferred to background catch-up. | same committed logs |
| Thumbnail completion handoff | One earlier Windows handoff trace recorded batch end at `+3886ms` and UI apply at `+163590ms`, a `159704ms` gap. | Final committed logs show thumbnail batch dispatch-to-receive in `8ms..49ms` for background catch-up batches. | earlier branch handoff analysis plus committed logs |
| Failed-thumbnail retries | Generation `2` immediately rescheduled the same `13` failed items. | Generation `2` suppresses all `13` with backoff and queues no new thumbnail work. | `logs/librapix-startup-20260310-185012-14648.log` |
| Timeline large-group rendering | One intersecting date group could force rendering `215` or `1,215` rows. | Timeline now renders only intersecting groups and only visible rows plus overscan inside each group. | `docs/TROUBLESHOOTING.md`, `docs/architecture/media-ui.md` |
| Route/filter/detail path | Route/filter actions rebuilt both browse routes, and first-open details could synchronously generate `detail-800`. | Refreshes are current-surface-first, offscreen work is deferred, and details reuse existing browse/detail artifacts without blocking generation. | `docs/architecture/message-flow.md`, `docs/TROUBLESHOOTING.md` |
| Scrollbar-thumb drag | One drag trace showed width churn from `438` to `1165` and `processed=38..50` stale drag updates. | Drag preview is latest-only, width/range are stabilized during preview, and settle applies one exact final viewport. | `docs/TROUBLESHOOTING.md`, `docs/architecture/media-ui.md` |

Two rows above are intentionally partly qualitative:

- the repository preserves exact pre-fix Timeline and thumb-drag failure evidence
- it does not preserve a committed post-fix interactive drag/timeline trace with equivalent numeric output

The final docs therefore state the exact before numbers and the exact after model truthfully, without inventing missing measurements.

## 7. Measured Improvements

All timings below are environment-specific Windows validation numbers from the committed logs and should be read as branch evidence, not universal guarantees.

- First usable gallery:
  - `943ms` in `logs/librapix-startup-20260310-185811-16788.log`
  - `1346ms` in `logs/librapix-startup-20260310-185012-14648.log`
- Startup ready:
  - `2087ms` with `13` thumbnails deferred
  - `2502ms` with `13` thumbnails deferred
- Thumbnail result handoff:
  - before: `159704ms` completion-to-apply gap in the handoff audit
  - after: `8ms..49ms` dispatch-to-receive in the final committed logs
- Failed thumbnail retry suppression:
  - before: generation `2` requeued the same `13` failures
  - after: generation `2` reports `suppressed_backoff=13`, `startup_priority=0`, and `deferred=0`

## 8. What Changed Architecturally

- The catalog-first data model stayed intact, but runtime ownership became explicit:
  - snapshot hydrate/apply
  - reconcile
  - projection
  - thumbnail batches
- Startup now has a structural ready-enough boundary instead of treating thumbnail backlog as startup work.
- Projection refresh policy moved from full dual-surface rebuilds to current-surface-first ownership.
- Thumbnail scheduling now depends on artifact reuse, placeholders, runtime backoff, and video throttling instead of naive exact-row presence.
- Large-surface correctness now depends on viewport-bounded rendering in the view layer, not only on faster background projection.
- Iced runtime timer usage now follows the supported timer API instead of blocking sleep-based subscriptions.
- Scrollbar-thumb drag is now treated as a preview/settle interaction, not as a requirement to process every intermediate viewport literally.

## 9. What Was Intentionally Not Changed

- No user source media is moved, renamed, or rewritten.
- No non-destructive guarantee changed.
- No destructive storage reset or schema rewrite was introduced.
- No feature-level product scope changed to solve this issue.
- The coordinator still lives in `crates/librapix-app/src/main.rs`.
- The branch still does not introduce FTS, a new search engine, aggregate timeline tables, or a deeper thumbnail pyramid.

## 10. Validation Performed

- Reviewed the final runtime behavior against the committed Windows logs:
  - `logs/librapix-startup-20260310-185012-14648.log`
  - `logs/librapix-startup-20260310-185811-16788.log`
- Preserved the earlier handoff and retry evidence from the branch's Windows log analysis in this summary so the real pre-fix failure modes remain documented after audit-doc cleanup.
- Cross-checked the final behavior against the core architecture docs:
  - `docs/architecture/message-flow.md`
  - `docs/architecture/media-ui.md`
  - `docs/architecture/thumbnails.md`
  - `docs/TROUBLESHOOTING.md`
- Kept the existing branch validation story intact:
  - Windows runtime automation hook via `LIBRAPIX_AUTOMATION_SCRIPT`
  - regression coverage for startup readiness, thumbnail reuse/backoff/cancellation, and runtime message handoff

## 11. Final Result / Expected User Experience

On a large populated library, the app should now feel like a desktop gallery again instead of a preload pipeline.

- The shell appears quickly.
- A useful snapshot-backed gallery appears early.
- Startup becomes ready without waiting for thumbnail catch-up.
- The full gallery can continue loading without blocking the initial experience.
- Opening Timeline or changing filters no longer implies rebuilding every offscreen surface.
- First-open details no longer freeze on synchronous thumbnail generation.
- Missing or failing thumbnails stay honest placeholders instead of trapping the app in repeated expensive retry loops.
- Large surfaces stay bounded to the viewport.
- Scrollbar-thumb drags preview cheaply and settle exactly.

## 12. Final Source Of Truth

For this issue:

- final branch summary: this document
- runtime message behavior: `docs/architecture/message-flow.md`
- media rendering and drag behavior: `docs/architecture/media-ui.md`
- thumbnail policy: `docs/architecture/thumbnails.md`
- recurring regression notes: `docs/TROUBLESHOOTING.md`
- broader catalog/data-model foundation: `docs/architecture/catalog-first-architecture.md`
