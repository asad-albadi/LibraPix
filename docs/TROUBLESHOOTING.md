# Troubleshooting

Issue `#12` is summarized in `docs/architecture/issue-12-runtime-optimization-summary.md`. This file keeps the recurring regression patterns and the final fixes that matter if those problems return.

## Catalog-first startup still feels like a preload or reaches ready too late

- Symptoms
  - `Loading library snapshot` feels broad instead of giving a quick first useful gallery.
  - `startup.ready` stays late even though the visible surface is already usable.
  - Unchanged launches either waste time on a redundant gallery rebuild or stay stuck on the restored 160-item gallery slice.
- Affected area
  - Startup/runtime orchestration in `crates/librapix-app/src/main.rs`.
- Confirmed cause
  - Earlier runtime states kept too much work inside the startup-critical path:
    - broad snapshot restore
    - eager offscreen surface refresh
    - startup-critical thumbnail work
    - redundant unchanged-launch gallery projection
  - Startup also needed an explicit continuation path so the fast snapshot slice did not become the permanent gallery state.
- Resolution
  - Keep storage open and migrations off the pre-render bootstrap path.
  - Restore only the bounded version-2 recent-gallery snapshot during startup.
  - Treat startup-ready as complete after snapshot apply, reconcile, and current-surface projection settle.
  - Skip the redundant unchanged-launch gallery rebuild when the restored default gallery snapshot is still valid.
  - Schedule a later non-blocking gallery continuation so the 160-card startup slice expands to the full gallery without blocking readiness.
  - Leave the non-visible route deferred until the user opens it.
- Prevention guidance
  - Keep startup-critical work limited to the first useful visible surface.
  - Do not move storage open or migrations back into the pre-render path.
  - Do not make thumbnail backlog or offscreen route refresh part of the startup-ready boundary.
  - If startup snapshots stay intentionally narrow, always pair them with an explicit continuation path.

## Background thumbnail work still makes the app feel hung after ready

- Symptoms
  - Existing thumbnails are not reused early enough, so startup or route refreshes schedule avoidable thumbnail work.
  - The same failing video items reappear on later projection generations.
  - A thumbnail worker batch finishes, but UI apply happens much later than worker completion.
- Affected area
  - Thumbnail scheduling/runtime policy in `crates/librapix-app/src/main.rs`.
  - Video extraction in `crates/librapix-thumbnails/src/lib.rs`.
  - Timer subscriptions in the Iced runtime path.
- Confirmed cause
  - Earlier projection-time reuse trusted exact ready `gallery-400` rows too narrowly and missed deterministic-file or compatible `detail-800` fallback reuse.
  - Failed items could re-enter the next projection refresh without runtime backoff.
  - Video work was still too eager and too coarse immediately after startup-ready.
  - Blocking sleep-based timer subscriptions could delay `ThumbnailBatchComplete` before it reached `update`.
- Resolution
  - Reuse browse thumbnails in this order:
    1. exact ready `gallery-400`
    2. deterministic on-disk `gallery-400`
    3. compatible ready `detail-800`
    4. deterministic `detail-800` fallback for visible items
    5. placeholder plus background generation
  - Keep startup-ready independent from all thumbnail batches.
  - Defer visible videos into slower background catch-up, throttle video batches to one item, and make in-flight video extraction cancellation-aware.
  - Apply runtime backoff and session disable for repeated or global video failures.
  - Use `iced::time::every(...)` instead of blocking sleep-based subscriptions so thumbnail completion messages are forwarded promptly.
  - Keep handoff logs explicit:
    - worker complete
    - dispatch to UI
    - message received
    - apply start/end
- Prevention guidance
  - Keep exact and compatible reuse ahead of generation.
  - Treat video work as placeholder-first background catch-up, not startup-critical work.
  - Never reintroduce blocking timer subscriptions built around `std::thread::sleep`.
  - Keep backoff/session-disable policy observable in logs so retry storms stay provable.

## Route switches, filter changes, or first-open details still stall

- Symptoms
  - Opening Timeline, changing filters, or switching back to Gallery briefly feels like a full refresh.
  - First-open details after startup can feel hung before the details pane populates.
- Affected area
  - Projection ownership and detail loading in `crates/librapix-app/src/main.rs`.
- Confirmed cause
  - Earlier post-startup refresh policy rebuilt both Gallery and Timeline even when only one surface was visible.
  - First-open details could still synchronously generate `detail-800` when the detail cache was cold.
- Resolution
  - Use a current-surface-first projection policy for route, filter, search, and filesystem refreshes.
  - Refresh only the active surface immediately and defer the non-visible surface until it is opened.
  - Keep detail loading placeholder-first:
    - reuse existing `detail-800` when present
    - otherwise reuse the browse thumbnail immediately
    - never generate a detail thumbnail synchronously on selection
- Prevention guidance
  - Do not treat every post-startup refresh as a full dual-surface rebuild.
  - Keep selection/details paths free of synchronous thumbnail generation.

## Large gallery or timeline surfaces still hang after projection

- Symptoms
  - Projection completes quickly, but the UI still stalls when Gallery or Timeline becomes visible.
  - Timeline is worst on libraries where one date section contains hundreds or thousands of justified rows.
- Affected area
  - Large-surface rendering in `crates/librapix-app/src/main.rs`.
- Confirmed cause
  - Projection work already moved off the UI thread, but the view path still tried to build too much widget tree in one frame.
  - Timeline virtualization originally stopped at the section level, so one intersecting group could still force rendering `215` or `1,215` rows at once.
- Resolution
  - Render Gallery, Timeline, and Search through a viewport-bounded window with top/bottom spacers that preserve the full scroll extent.
  - Virtualize rows inside each intersecting Timeline date section, not only the sections themselves.
  - Keep render-window diagnostics explicit:
    - `interaction.surface_render.window`
    - `interaction.timeline_render.window`
    - `interaction.timeline_render.window.anomaly`
- Prevention guidance
  - Treat view-layer rendering cost separately from background projection cost.
  - For grouped virtualized surfaces, do not stop at group-level windowing if individual groups can still become huge.

## Dragging the media scrollbar thumb still lags

- Symptoms
  - Dragging the scrollbar thumb on large Gallery or Timeline surfaces feels sticky or briefly hung.
  - Earlier Windows drag traces showed width churn from `438` to `1165` during a single drag and `processed=38..50` intermediate drag updates.
- Affected area
  - Viewport drag handling and justified-layout reuse in `crates/librapix-app/src/main.rs`.
- Confirmed cause
  - Drag-time width churn and scroll-range churn could still invalidate layout work repeatedly.
  - Active drag still processed too many stale intermediate viewport targets.
  - Drag lifecycle boundaries were too easy to fragment with sparse or near-edge events.
- Resolution
  - Treat thumb drag as an explicit preview/settle lifecycle.
  - Require a real movement burst before entering active drag mode.
  - Use latest-only pending target replacement with cadence-capped preview applies.
  - Freeze drag-time justified-layout width and effective `max_y`.
  - Skip max-only active-drag updates and apply one exact final viewport snapshot at settle.
  - Keep diagnostics explicit:
    - `interaction.viewport.drag.start`
    - `interaction.viewport.drag.update`
    - `interaction.viewport.settle.start`
    - `interaction.viewport.settle.end`
    - `interaction.surface_layout.drag_width.freeze`
    - `interaction.surface_layout.drag_width.anomaly`
- Prevention guidance
  - Do not process every thumb position literally when preview-mode behavior is sufficient.
  - Keep drag-time geometry inputs stable and restore exact correctness only at settle.
  - Avoid synchronous high-volume logging on the drag hot path.

## Startup logs are hard to find

- Symptoms
  - Runtime instrumentation exists, but it is unclear where the active log file was written.
- Affected area
  - Logging bootstrap in `crates/librapix-app/src/startup_log.rs`.
- Confirmed behavior
  - Development or portable-style runs first try a nearby `logs/` directory.
  - Other runs fall back to the platform log directory resolved from `directories::ProjectDirs`.
  - The active log path is:
    - written into the log itself
    - printed to stderr on startup
    - exposed in the diagnostics panel
- Resolution
  - Check the diagnostics panel `startup log:` line.
  - If launching from a terminal, check the stderr line starting with `Librapix log:`.
- Prevention guidance
  - Keep active log-path visibility intact whenever startup logging changes.
