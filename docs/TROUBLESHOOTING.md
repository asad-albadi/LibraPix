# Troubleshooting

## Settings action clicks lost near a responsive breakpoint (resolved)

- Status: resolved by isolating natural-height measurement from live widget state.
- Symptoms: Enable/Disable, Edit, or Remove could fail when an app update
  occurred between mouse press and release at a narrow dialog width.
- Affected area: shared `ui/patterns/dialog_layout.rs` and responsive Settings
  rule rows in `views/dialogs/settings.rs`.
- Confirmed cause: measurement and bounded layout reconciled the same live tree
  at different widths. Measurement removed scrollbar spacing; `Responsive`
  rebuilt row/column structures across its 420px breakpoint and reset presses.
  Installed Iced 0.14.2 `Responsive::layout` rebuilds its internal content every
  call and diffs the supplied tree; isolating only that internal content would
  not protect live state.
- Reproduction: a reduced fixture crossed 425px/411px. The production Settings
  regression reproduces the lost Disable action at 500px outer width with an
  overflowing rule list, after press, view rebuild/layout, then release.
- Resolution: measure with a disposable `Tree`, then perform final bounded layout
  with the persistent live tree. Responsive content is rebuilt again during the
  final pass. Content-hugging frames, bounded scrolling, footer placement, and
  the Settings breakpoint/actions are unchanged.
- Prevention: the production-view test sweeps outer widths 400-519px plus
  320px/680px, preserving one live tree per width through Enable/Disable, Edit,
  and Remove press/rebuild/release sequences. Retain short-dialog and overflow
  tests, and recheck Responsive's content rebuilding when upgrading Iced.
- Validation: `cargo test --workspace -- --test-threads=1` passes, including
  native headless layout/input/scrolling tests. Serial execution keeps the
  renderer-heavy tests from competing for resources; it is not a substitute
  for native interactive Windows/Linux checks, which were not repeated for
  this fix.

## Header wordmark feels crowded despite being enlarged

- Symptoms: branding nearly touches the header edges and lacks visual balance.
- Affected area: header and About presentation.
- Confirmed cause: duplicated 180x44 image boxes left only 4px vertical room in
  the 52px header; unrelated width/height constraints obscured artwork proportions.
- Resolution: shared `ui/components/brand.rs` placement with asset-derived width,
  36px header artwork and at least 8px top/bottom clearance; proportional About art.
- Prevention: real-Iced layout tests cover both themes and narrow widths. Keep
  asset fidelity separate from placement; layout cannot correct generated glyph
  variation. Do not regenerate or reshape approved artwork without owner approval.

The centralized UI migration's confirmed regressions and resolutions are recorded
below. ADR 0025 defines the final presentation boundary. Layout/event/render tests
are prevention evidence, not a claim that all native desktop checks were performed.

## Long library names are clipped or extend outside clickable rows

- Symptoms: multi-line library names lose lines or spill beyond their row's hit area.
- Affected area: expanded sidebar root navigation after centralized-control migration.
- Confirmed cause: root rows used fixed-height tab triggers (28 px), constraining wrapped text and its clickable area.
- Resolution: use the sidebar pattern's content-sized `navigation_item`, retaining centralized tab/active styling and existing navigation messages.
- Prevention: test adjacent long labels with real Iced constrained layout, independently measure complete text height, and click both the first and last lines to verify distinct, non-overlapping hit areas.

## Make Short indicator is static or long errors crowd the actions

- Symptoms: the loading dots do not animate during ordinary short generation; lengthy ffmpeg stderr crowds or collapses Close/Generate in the result footer.
- Affected area: Make Short dialog presentation after the shared-dialog migration.
- Confirmed causes: dots used the unrelated new-media preview phase, whose subscription is disabled without a pending preview. Unrestricted stderr was placed in the non-scrolling fixed footer.
- Resolution: use a separate presentation phase/timer while the visible Make Short dialog is Running. Keep only the concise summary/actions in the footer and put stderr in the scrollable body. Generation and cancellation logic are unchanged.
- Prevention: exercise the actual app subscription/update path without preview activity and verify phase-driven frames change. Test lengthy stderr at constrained window sizes, including reachable error content and stable visible actions after scrolling.

## Dialog migration verification and hidden library validation

- Symptoms: an invalid library path leaves the dialog open without visible error feedback; Windows desktop automation may also fail to activate the validation app.
- Affected area: library dialog presentation and local Windows visual verification.
- Confirmed causes: library validation sets `root_status`, but the old dialog did not render that feedback. In the validation environment, desktop activation returned `GetCursorPos failed: Access is denied. (0x80070005)` on the initial attempt and one fresh-window retry; the capture showed desktop wallpaper instead of application content.
- Resolution: the shared library path field displays the existing localized invalid-path status. No validation/update behavior changed. For the desktop access issue, do not treat the captured wallpaper as successful verification; rerun interactive checks in an accessible desktop session.
- Prevention: retain real dialog layout/event regression tests and record desktop-access limitations explicitly. Headless light/dark rendering is supplemental, not a substitute for manual generation/cancellation checks.
- Headless test note: send `Window::RedrawRequested` before drawing a newly constructed dialog tree. Iced 0.14 initializes button interaction styles on that event; layout alone (or a cursor event) draws the default disabled style, hiding active-filter differences and producing misleading pixel comparisons.

## Media jumps to the top when a shell notice changes

- Symptoms: a layout-save error notice appearing or being dismissed resets the
  gallery/media scroll offset to zero.
- Affected area: application shell composition in `views/shell.rs`.
- Confirmed cause: switching between a bare shell container and a notice stack
  replaces Iced's persistent widget tree, discarding its scrollable state.
- Resolution: always retain the same shell stack and first child; conditionally
  append only the notice layer.
- Prevention: test notice appearance and dismissal with one persistent tree,
  reconciling each new view and inspecting the actual media scroll offset.

## Windows rebuild fails while the validation app is running

- Symptom: Cargo reports `failed to remove file ... librapix-app.exe` with
  `Access is denied (os error 5)` during a check/test/build after manual validation.
- Affected area: local Windows development, not application pane behavior.
- Confirmed cause: the running executable is locked by Windows while Cargo tries
  to replace it. This occurred during the collapsible-pane validation loop.
- Resolution: close the validation app normally, then rerun the command.
- Prevention: finish manual interaction and close that build before rebuilding;
  no application or process-spawning workaround is needed.

Issue `#12` is summarized in `docs/architecture/issue-12-runtime-optimization-summary.md`. This file keeps the recurring regression patterns and the final fixes that matter if those problems return.

## Accent text loses contrast after a palette change

- Symptoms: blue text is difficult to read on dark neutral surfaces even though white text on blue buttons remains readable.
- Affected area: app-local semantic palette and accent text/button styles.
- Confirmed cause: a filled-control accent must be dark enough for white text; reusing that fill as small foreground text on dark surfaces does not meet the same contrast requirement. Component contrast tests caught this during the media-first palette update.
- Resolution: keep `accent` for filled controls and use the dedicated `accent_text` token for accent text and link buttons. Dark mode uses a brighter blue foreground; light mode retains the deeper blue.
- Prevention: test text/background pairs in both themes, including hover/selection surfaces, instead of evaluating individual swatches in isolation.

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

## Copy Path (or other clipboard text) freezes the whole UI on Windows

- Symptoms
  - Clicking "Copy Path" briefly hangs the entire app (input, scrolling, rendering all stall), sometimes with a console-window flash.
  - "Copy File" does not exhibit the same hang.
- Affected area
  - Clipboard text write in `crates/librapix-app/src/main.rs` (`copy_text_to_clipboard`).
- Confirmed cause
  - The Windows branch spawned the `clip.exe` process and blocked on `child.wait()`. That call ran synchronously inside `update()`, which is the Iced UI thread, so the app froze for the duration of process spawn + stdin pipe + wait (plus a console-window flash, since the subprocess had no `CREATE_NO_WINDOW`). Copy File never hung because it already used the native clipboard API.
- Resolution
  - Write text to the clipboard with the native Win32 API (`SetClipboardData(CF_UNICODETEXT)`) using the same `GlobalAlloc` -> `GlobalLock`/write -> `GlobalUnlock` -> `SetClipboardData` ownership flow as the file (`CF_HDROP`) path. No subprocess, no `wait()`, no console flash.
- Prevention guidance
  - Never spawn a process and `wait()` on it inside `update()` / on the UI thread. Use the native API, or move blocking work to a background `Task`/thread.
  - On Windows, any subprocess launched from the GUI must set `CREATE_NO_WINDOW` to avoid console flicker.

## Centralized UI labels, density, and dialog sizing

- Symptoms: long filter names clip or outgrow their hit targets; inline red text is hard to read; collapse controls waste a row; short dialogs show large empty regions; Settings rules have oversized saturated surfaces.
- Affected area: app-local components, dialog/pane patterns, filter and Settings views.
- Confirmed causes: fixed 28-pixel tab height plus fixed-width wrapping estimates; fill/border destructive color used as a text color; a standalone collapse-control row; `Length::Fill` on the shared dialog frame/body; tag-style rows with an unconditional secondary status line.
- Resolutions: content-sized native wrapping controls; `destructive_text`; integrated Browse header control; centered 90-degree rotated Details word with full-label hover help; compact responsive rule rows with localized remove help and preserved per-key chip tones; content-sized bounded dialogs.
- Details rotation: stacked lettering is not a rotated word. General Iced Transformation cannot rotate, but native geometry Frame can. Use `Frame::with_bounds` with absolute layout/clip coordinates and rotate the whole text through the existing renderer. Keep the intact label in text operations and the native full-height button/tooltip around it. Pixel regressions compare against a quarter-turn of native glyph outlines, not horizontal hinted glyph bitmaps (different rasterization). Test rendered pixels as well as layout: a translated local geometry frame initially produced no pixels in tiny-skia despite valid layout because its group clip was transformed twice upstream.
- Color-identity regression: the initial Settings density pass replaced keyed chip surfaces with a single neutral card style. Reuse `chip_tone_for_key` and managed-chip surface/action styles instead, preserving color identity for enabled and disabled rules. Real-Iced render tests require representative distinct tones to appear consistently in both themes, with readable text contrast; event/layout tests retain narrow-width enable/disable/edit/remove behavior.
- Dialog sizing resolution: in Iced 0.14, simply changing the outer frame to `Shrink` compresses its Column and measures the scrolling body before the footer; overflow can then leave a zero-height footer. The approved private layout-only adapter first measures natural height at the constrained width, then delegates native layout at the capped fixed height. This restores native header/footer priority while shrinking short frames. There are no guessed pixel budgets or custom drawing. Regression tests exercise short New Media spacing, native wheel scrolling with a persistent tree, and all four footer actions before/after scrolling at constrained sizes.
- Prevention: exercise real Iced layout at narrow widths and short heights, compare complete wrapped text to its hit area, dispatch first/last-line clicks, test hover overlays, and retain the persistent-tree long-error scroll tests. Check the actual error-text helper against dialog gradient stops as well as solid palette surfaces.
