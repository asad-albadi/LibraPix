# Background Thumbnail Responsiveness Audit

Date: 2026-03-10

Scope: post-`startup.ready` responsiveness on `feat/catalog-first-architecture`, with focus on Windows large-library behavior and failing video thumbnails.

## Target case

Observed log: `logs/librapix-startup-20260310-164429-10636.log`

Key evidence from that run:

- `startup.ready` reached at `+1893ms`, so startup-ready itself is no longer the main blocker.
- Startup-priority thumbnail work then began with `15` items.
- `13` of those items failed thumbnail generation.
- The same `13` failing items were retried again on projection generation `2`.
- Generation `2` shows the repeated retry clearly:
  - `startup.thumbnail_lookup.scheduled_generation | generation=2 items=13`
  - followed by the same failing video media ids as generation `1`

This means the remaining product issue is no longer startup readiness. It is post-ready thumbnail behavior.

## Current post-ready path

1. `ProjectionJobComplete` applies gallery/timeline/search results.
2. `split_startup_thumbnail_work(...)` divides missing thumbnails into:
   - startup-priority queue
   - deferred queue
3. `start_thumbnail_batches(...)` enqueues a thumbnail batch task.
4. `do_thumbnail_batch(...)` processes each item serially inside one worker task.
5. `ThumbnailBatchComplete` applies the batch result back into app state.

Relevant code:

- `crates/librapix-app/src/main.rs`
  - `apply_projection_job_result(...)`
  - `split_startup_thumbnail_work(...)`
  - `start_thumbnail_batches(...)`
  - `run_next_thumbnail_batch_if_idle(...)`
  - `do_thumbnail_batch(...)`
  - `apply_thumbnail_batch_result(...)`
  - `cancel_thumbnail_work(...)`
- `crates/librapix-thumbnails/src/lib.rs`
  - `ensure_image_thumbnail(...)`
  - `ensure_video_thumbnail(...)`

## Audit findings

### 1. Failed thumbnail artifacts are not treated as a scheduling backoff signal

Current projection-time lookup only loads ready artifacts:

- `list_ready_derived_artifacts_for_media_ids(..., "gallery-400")`
- `list_ready_derived_artifacts_for_media_ids(..., "detail-800")`

`failed` rows in `derived_artifacts` are written during `do_thumbnail_batch(...)`, but projection scheduling does not consult them.

Result:

- media that just failed thumbnail generation are eligible again on the next projection refresh
- route switches and later refreshes can immediately requeue the same failing items

This exact retry loop is visible in the Windows log between generation `1` and generation `2`.

### 2. Video failures are too opaque

Current `ensure_video_thumbnail(...)`:

- invokes `ffmpeg.exe` / `ffmpeg`
- discards stdout/stderr
- does not log resolved executable path
- does not log exit code
- does not log stderr summary
- returns one generic error string for every failure:
  - `video thumbnail extraction failed (ffmpeg may not be installed or in PATH)`

Result:

- real Windows failures are not diagnosable from logs
- missing ffmpeg, bad invocation, timeout, corrupt media, and ffmpeg exit failures all look identical
- the app cannot distinguish fast global disable conditions from per-file failures

### 3. Video work is still too eager after startup-ready

Current startup-priority scheduling can still put video thumbnail jobs into the first post-ready batch.

Even though startup-ready no longer waits for thumbnail completion, this still means:

- the app begins video subprocess work immediately after readiness
- visible video failures can flood the system with ffmpeg process launches right after startup
- placeholder rendering exists, but the scheduler still behaves as if video thumbnails deserve near-immediate catch-up

This is product-wrong for the current branch. Video thumbnails should be more deferred and more throttled than image thumbnails.

### 4. Cancellation is only partial

`cancel_thumbnail_work(...)` clears queued work and invalidates the active generation, but it does not actually preempt the currently running batch task.

Current consequence:

- route switches and refreshes stop future batches
- but the in-flight batch still runs until the worker returns
- if that batch contains several video items, stale work can keep consuming resources after the UI has moved on

### 5. Batch shape is too coarse for disruptive video work

`do_thumbnail_batch(...)` processes all items in the batch serially inside one worker future.

That is acceptable for light image batches, but it is too coarse for failing or slow video work because:

- cancellation granularity is one whole batch
- repeated ffmpeg process spawning can happen back-to-back
- route changes cannot preempt between video items unless the whole batch ends

### 6. Thumbnail result application is not fully observable

The current logs show:

- thumbnail stage start/end
- slow item logs
- generation failures

But they do not show enough to diagnose post-ready lag precisely:

- worker batch start/end timing per batch
- worker cancel timing
- apply-time duration in the main app state
- thumbnail message/result rate over time
- whether thumbnail outcomes caused projection reruns or only coincided with later user/system refreshes

This is why the `+166s` gap in the Windows log is suspicious but not yet fully attributable.

### 7. Thumbnail outcomes do not directly request projection refreshes

Code evidence:

- `apply_thumbnail_batch_result(...)` patches paths into existing browse items
- it does not request a projection refresh unless a refresh was already pending for another reason

Conclusion:

- thumbnail outcomes themselves are not directly triggering fresh projections
- the harmful churn is coming from repeated scheduling, video subprocess work, and stale in-flight batches

## Root causes to address

1. failed items are retried too aggressively because scheduling ignores recent failures
2. video thumbnail failures are not classified, so the app cannot fast-fail or back off correctly
3. video work is scheduled too eagerly right after startup-ready
4. in-flight thumbnail work is not preemptible enough
5. instrumentation is too shallow to prove where post-ready lag is spent

## Fix plan

### Scheduling and policy

- Keep image and video thumbnail policy separate.
- Visible images may remain higher priority.
- Visible videos should default to placeholder-first and background-later behavior.
- Defer video work more aggressively than image work after startup-ready.

### Fast-fail and backoff

- Distinguish video failure classes:
  - ffmpeg not found
  - ffmpeg spawn failure
  - ffmpeg timeout
  - ffmpeg non-zero exit
  - corrupt/unreadable image decode
- If ffmpeg resolution is broken, detect once and disable repeated video attempts for the session.
- Record per-media failure cooldown so repeated route/projection refreshes do not immediately retry known-bad items in the same session.

### Cancellation and throttling

- Reduce batch pressure for video jobs.
- Make in-flight video extraction cancellable/preemptible where practical.
- Cancel or ignore stale queued work on route/projection changes.
- Keep offscreen or stale video work from competing strongly with current interaction.

### Instrumentation

- Log worker batch start/end/cancel timing.
- Log video ffmpeg resolution and effective command.
- Log exit code, timeout, and stderr summary for video failures.
- Log backoff state and retry suppression.
- Log thumbnail result apply duration and result-message rate.
- Log whether projection refreshes occurred while thumbnail work was active.

## What must block startup vs what must not

Still allowed to block startup:

- snapshot hydrate/apply
- reconcile
- current-surface projection

Must never block startup or post-ready interactivity:

- image thumbnail catch-up
- video thumbnail generation
- thumbnail failure retries
- thumbnail retry storms after route switches or refreshes

## Desired end state

- The GUI is usable immediately after `startup.ready`.
- Missing thumbnails show placeholders first.
- Existing exact or compatible thumbnails still reuse immediately.
- Image catch-up remains light and non-disruptive.
- Video work becomes throttled, cancellable, and backoff-aware.
- Repeated video failures no longer flood the app or make the GUI feel hung.
