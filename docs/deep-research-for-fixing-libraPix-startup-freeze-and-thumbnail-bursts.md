# Deep research for fixing LibraPix startup freeze and thumbnail bursts

## Executive summary

Your remaining startup “freeze” is highly consistent with **UI-thread starvation**, not background indexing: if the main thread does not pump window messages for “more than several seconds,” Windows considers the top‑level window “not responding” and may replace it with a ghost window. citeturn14view0 This aligns with your observation that you can’t move the window and progress only appears at the start/end.

In an entity["company","Iced","rust gui library"] app, the most common ways to starve the UI thread at startup with large libraries are:

- Building **a full gallery widget tree** (thousands of children) and/or repeatedly re-laying it (non‑virtualized scrollables).
- Doing **filesystem I/O or image handle churn** in `view()` for many items: `image::Handle::from_path` represents a file handle whose “image data will be read from the file path,” and it “examines the data in the file” to guess format. citeturn9view1
- Recreating image handles or reading image bytes repeatedly during frequent update/view cycles can trigger pathological behavior (lockups/flicker) in Iced; multiple upstream issues document this class of problem. citeturn8view2turn10view1

The fix needs to be **narrow and brutal**: “Loading library snapshot” must *not* cause the gallery view to render thousands of cards (or touch the filesystem for each card). The highest‑leverage change is **viewport-based virtualization/windowing for the gallery grid** using `Scrollable::on_scroll(Viewport)` to know what’s visible. citeturn11view2turn6view1 Then, schedule thumbnails based on the viewport (prefetch buffer), using subscriptions/ticks only for retry timing, not as a wakeup hack. Subscriptions are designed to be state-driven (start/stop based on app state) and can be timed with `time::every`. citeturn6view3turn16view1

## Evidence-based diagnosis summary

### Startup freeze mechanism

When the UI becomes unresponsive (can’t move/resize window), Windows is effectively not receiving timely message processing. Microsoft’s Win32 documentation explains that if a top‑level window “stops responding to messages for more than several seconds,” Windows considers it “not responding.” citeturn14view0

In Iced, the easiest way to create this condition is to make `view()` and the resulting layout/draw pass *too expensive* (e.g., thousands of children in one scrollable/grid).

The Iced community has explicitly observed that inserting ~1000 items in a default scrollable can “lag a bunch” because “it renders all of the content at once,” and they point to a dedicated `List` widget / visible-content rendering as the solution direction. citeturn8view0

### Image handle / I/O amplification

Iced’s image handle types are explicit about `Path` being a file-backed handle: “The image data will be read from the file path,” and `from_path` “examines the data in the file” to guess format. citeturn9view1 If you create thousands of image handles at startup (or even just cause the renderer to try to allocate them), disk I/O and GPU allocations can balloon and stall.

There are also real upstream Iced bugs/regressions around repeatedly creating image handles in dynamic layouts and frequent view cycles:

- Lockup at 100% CPU when creating multiple images from bytes inside `responsive`. citeturn8view2  
- Flicker/disappear behavior when `Handle::from_bytes` is recreated during frequent view calls driven by subscriptions/events; the report explicitly notes that recreating handles repeatedly can cause continuous resource churn. citeturn10view1  

Even if LibraPix uses `from_path` rather than `from_bytes`, the principle is the same: **avoid per-frame/per-view handle recreation and avoid any file reads or `Path::exists()` checks in view** for large collections.

### Thumbnail bursts

The burst behavior (“some thumbnails appear only after the next screenshot”) is consistent with retry progression being tied to “the next event” rather than being self-driven. Iced’s model for self-driven periodic work is `Subscription` with `time::every`, which produces messages at a set interval (requires tokio/smol/wasm). citeturn16view1turn6view3

Viewport-driven scheduling also matters: `Scrollable::on_scroll` provides the current `Viewport`, enabling near-viewport prefetch. citeturn11view2turn6view1 For per-item “near visible” signals, `Sensor::anticipate` is specifically documented as useful to “lazily load items in a long scrollable…before the user can notice it.” citeturn6view2

## Prioritized concrete fixes to implement

This is the narrowly-scoped, high-leverage order for entity["organization","LibraPix","desktop photo manager"].

### Make first paint minimal and decouple it from full gallery realization

Goal: show an interactive shell fast, then progressively load *data*, but do not progressively “grow the widget tree.”

- Ensure snapshot hydrate/apply does **not** force an O(N) gallery widget build. (N = full library size)
- Ensure `view()` does not loop through all `gallery_items` to build widgets. Instead, compute a visible range and render only that window.

This directly addresses the root freeze cause described by Windows: starved message processing. citeturn14view0

### Add viewport-based virtualization (windowing) for the gallery grid

Implement a “VirtualGrid” for gallery that renders: top spacer + visible rows + bottom spacer.

Use `Scrollable::on_scroll(|viewport| ...)` to store viewport in app state. citeturn11view2  
Use `Viewport::absolute_offset()` / `bounds()` / `content_bounds()` to compute visible rows. citeturn6view1

The Iced community thread makes it explicit that the default approach renders all content and falls apart for thousands of items; virtualization is the fix. citeturn8view0

### Schedule thumbnails based on viewport (prefetch buffer)

Once the visible range exists, use it to drive thumbnail queue priority:

- Visible + 1–3 rows prefetch buffer should be highest priority.
- Selected item should always be highest priority.

If you want per-element events, `Sensor::anticipate` is designed for this “lazy load before visible” use case. citeturn6view2  
However, the recommended narrow approach is **viewport math** (fewer widgets, simpler).

### Cache image handles and avoid per-frame allocations

- Never call `fs::read`, `Path::exists`, or other filesystem operations in `view()` for every card.
- For visible items only, use `Handle::from_path` *once per thumbnail path* and cache it (HashMap/LRU keyed by thumbnail path + size bucket + mtime hash). This avoids repeated file/format examination and resource ID churn. citeturn9view1turn10view1
- Do not recreate handles in a tight loop under `responsive` or frequent message ticks, because Iced has documented lockups/flicker patterns when handles are recreated frequently. citeturn8view2turn10view1

### Keep retry progression subscription-driven, not watcher-driven

Retain the existing retry tick strategy: timed retries should be driven by `time::every` subscriptions (no sleeps). citeturn16view1turn6view3  
Ensure retry scheduling does not require a filesystem event to “wake up” processing.

## Implementation options and trade-offs

| Option | What it is | Pros | Cons | Estimated effort |
|---|---|---|---|---|
| COSMIC List approach | Port/adapt the virtualized `List` widget design used in entity["organization","libcosmic","cosmic toolkit"]: it computes offsets in batches (`MAX_BATCH_SIZE`) and maintains `visible_elements`/`visible_layouts` so only visible widgets exist. citeturn4view0turn4view1 | Proven virtualization strategy in the Iced ecosystem; amortized layout in batches; clear reference implementation. citeturn4view0turn4view1 | It’s a **list**, not a **grid**; adapting to a grid adds complexity (row/col layout, height assumptions). Risk of subtle widget-tree bugs when transplanting custom widgets. | Medium (2–5 days) |
| Custom viewport slicing | Implement a VirtualGrid at the application level using `Scrollable::on_scroll(Viewport)` and fixed row height: top spacer + render only visible items + bottom spacer. citeturn11view2turn6view1 | Smallest change set; easy to reason about; grid-friendly; avoids introducing a custom widget tree engine. | Requires stable row height (or approximate) for correct mapping; variable-height cards complicate it; needs careful selection identity handling. | Small–Medium (1–3 days) |

Recommendation for LibraPix’s current urgency: **Custom viewport slicing first**, because it is the narrowest path to “window becomes responsive quickly.”

## Implementation-grade task prompt for the project agent

### Objective

Fix LibraPix startup freeze on large libraries and remaining thumbnail-burst inconsistencies by removing UI-thread starvation at startup. Do this by implementing gallery grid virtualization and eliminating filesystem/image handle churn from `view()`.

### Concise diagnosis summary (to guide the agent)

- The Windows startup freeze is likely caused by UI-thread starvation: a huge gallery widget-tree is being materialized and/or repeatedly laid out/drawn during “Loading library snapshot,” preventing normal window message processing. citeturn14view0turn8view0  
- The gallery uses a non-virtualized approach (default scrollable/grid), which is known to fall apart with thousands of items because it renders everything. citeturn8view0  
- Image handles can trigger file-backed I/O and format probing (`from_path` examines file data; Path handle reads from file path). citeturn9view1 Creating/allocating too many at startup amplifies stalls.  
- Recreating image handles frequently can cause severe issues (lockups/flicker) in Iced; avoid per-view handle recreation and per-frame allocations. citeturn8view2turn10view1  
- Thumbnail retries and “burst” behavior must be driven by subscriptions/ticks (`time::every`) rather than requiring watcher events for wakeup. citeturn16view1turn6view3  

### Prioritized concrete fixes to implement now

1) **Implement VirtualGrid for gallery** (custom viewport slicing approach).  
   - Add `Message::GalleryScrolled(Viewport)` using `Scrollable::on_scroll`. citeturn11view2  
   - Store latest `Viewport` in state. Use `Viewport::absolute_offset()` + `bounds()` to compute visible row range. citeturn6view1  
   - Render only a bounded range of items (visible + small buffer), with top/bottom spacers to preserve scroll height.

2) **Remove filesystem I/O from `view()` for gallery items and details**.  
   - Do not call `Path::exists()` (or any file stat) per item per view.  
   - Derive thumbnail presentation purely from in-memory `thumbnail_states` populated by background work.  
   - Only when `thumbnail_states[id] == Ready(path)` should `view()` attempt to show an image.

3) **Cache `image::Handle` for thumbnails used in visible range**.  
   - Cache by `(thumb_path, size_bucket)` (or equivalent stable key).  
   - This avoids repeated identity churn and repeated “examine file data” costs from repeated `from_path` calls. citeturn9view1turn10view1  
   - Avoid handle recreation patterns implicated in upstream issues. citeturn8view2turn10view1  
   - Use a small LRU or capped HashMap to avoid unbounded growth.

4) **Viewport-driven thumbnail scheduling**.  
   - On `GalleryScrolled`, compute `(visible_range + prefetch_buffer)` and enqueue thumbnails for those media IDs if missing/queued/failed-retryable.  
   - Optional: Use `Sensor::anticipate` later, but prefer viewport math first (simpler, fewer widgets). citeturn6view2  

5) **Keep subscription-driven retry ticks**; ensure retries progress without requiring filesystem events.  
   - Continue using `iced::time::every` for retry tick scheduling (tokio/smol feature as needed). citeturn16view1turn6view3  
   - Do not add sleeps or blocking logic.

### Files and functions to inspect/change in the repo

Primary (must inspect):

- `crates/librapix-app/src/main.rs`  
  - Startup snapshot hydrate/apply: `HydrateSnapshotComplete`, `apply_snapshot_hydrate_result`, any chunk-apply state/logic (`PendingSnapshotApply`, `SnapshotApplyTick`, etc.).  
  - View and gallery rendering: the function(s) that build the gallery grid (search for `Gallery`, `gallery_items`, `scrollable`, `render_media_card`, and any loop over all items).  
  - Scroll handling: wherever `Scrollable` is constructed—add `.on_scroll(Message::GalleryScrolled)` and state updates. citeturn11view2  
  - Thumbnail queueing + state: `thumbnail_states` / presentation mapping; ensure `view()` does not call filesystem checks.  
  - New-file modal: ensure it uses cached/presentational state and does not trigger per-item I/O.

- `crates/librapix-thumbnails/src/lib.rs`  
  - `ensure_image_thumbnail`, `ensure_video_thumbnail`: confirm they are only called in background workers, and that ready outcomes update `thumbnail_states`.

- `crates/librapix-core/src/app/mod.rs`  
  - State apply helpers that may currently cause bulk materialization or derived-state rebuilds; confirm virtualization changes only affect rendering, not correctness.

- Storage snapshot APIs  
  - Verify persisted snapshot read/write remains; no need to change schema for this task, but ensure startup doesn’t eagerly “expand” snapshot into a fully rendered UI.

### Implementation guidance with options and trade-offs

Mainline path: **Custom viewport slicing VirtualGrid**

- Use `Scrollable::on_scroll` + `Viewport` to keep a current scroll position. citeturn11view2turn6view1  
- Compute:
  - `columns = max(1, floor((available_width + spacing) / (tile_width + spacing)))`
  - `row_height = tile_height + vertical_spacing` (prefer fixed tile height)
  - `start_row = floor(scroll_offset_y / row_height)` (from `Viewport::absolute_offset().y`)
  - `end_row = ceil((scroll_offset_y + viewport_height)/row_height)`
  - Convert row range to item index range: `[start_row*columns, end_row*columns)`
- Render only items in that index range:
  - Build rows of `columns` using `Row::push`.
  - Use `Space` elements for missing items in last row.
- Use top/bottom `Space` to simulate full content height:
  - `top_spacer = start_row * row_height`
  - `bottom_spacer = (total_rows - end_row) * row_height`

Alternative reference: COSMIC `List` design

If you need a reference for how serious virtualization is implemented in the Iced ecosystem, study COSMIC’s `List` widget: it computes offsets in batches (`MAX_BATCH_SIZE`) and only keeps `visible_elements` and `visible_layouts`, splicing in/out as the viewport changes. citeturn4view0turn4view1  
Use this as an algorithmic reference even if you don’t port it directly.

Image-handle caching rules

- Treat `Handle::from_path` as potentially expensive because it reads from file path / examines file data. citeturn9view1  
- Never rebuild/allocate image handles for items outside visible range.  
- Cache handles for visible items; reuse them across view calls. This reduces churn patterns implicated by Iced image issues. citeturn8view2turn10view1  

### Verification steps and metrics

Metrics to collect (report in the agent output):

- **Time-to-interactive (TTI)** on the large Windows library:  
  - Start timer at process start.  
  - TTI ends when the window can be moved/resized smoothly and sidebar interactions are responsive.  
  - Target ≤ 2 seconds (stop condition below).  
  - Use simple logging timestamps; Windows guidance recommends clearly defining scenarios and adding start/stop events for measurement. citeturn7view1turn7view0  

- **First paint**: time until shell frame renders and is interactive (even if gallery still loading). This aligns with Microsoft’s definition of startup ending when user can interact in a meaningful way. citeturn7view0  

- **Single screenshot thumbnail latency**:  
  - Take one screenshot; measure time until its thumbnail appears in gallery and in the “new file detected” modal (if shown).  
  - Target ≤ 3 seconds without needing another filesystem event (stop condition).

Regression tests to add (unit-level, not UI automation):

- Visible-range math tests:
  - Given a viewport offset + bounds + known tile sizes, assert correct `start/end` indices.
- Prefetch range tests:
  - Given visible range and buffer constants, assert correct `prefetch` indices and no overflows.
- Handle cache behavior tests:
  - Given repeated render cycles, ensure handle cache returns same handle key and is bounded (LRU/cap).

Manual smoke checks (must be reported):

- `cargo run -p librapix-app --release` on Windows.  
- Launch with large library; confirm window remains movable during snapshot load.  
- Scroll the gallery: confirm frame stability and that only visible cards allocate thumbnails (watch logs if instrumented).  
- Take a single screenshot: confirm thumbnail appears without a second screenshot.

### UI/UX constraints (must not regress)

- Preserve staged runtime and generation guards.
- Do not load original media files in the gallery grid; grid must remain thumbnail-first.
- Preserve selection identity across projection refreshes (virtualization must not “lose” selection when item is out of view).
- Preserve non-destructive behavior and multi-root semantics.

### Deliverables expected from agent

- Code changes with exact file paths and clean diffs.
- New regression tests for viewport slicing and thumbnail prefetch logic.
- Smoke-run report including timing metrics (TTI, first paint, thumbnail latency).
- One or more focused commits with clear messages.
- Clean working tree.
- Short PR description summarizing the fix and the measured improvements.

### Stop condition

This task is done only when all are true:

- On the large Windows dataset, **interactive shell/TTI ≤ 2 seconds** (window movable, sidebar clickable) while background work continues.  
- A single screenshot creates a thumbnail that appears in gallery (and modal preview if applicable) **within 3 seconds**, without needing another watcher event.  
- No filesystem I/O occurs in `view()` per item (no `exists`, no `read`, no handle churn across the full dataset).  
- All verification steps pass, commits created, working tree clean.

## Required reading order and prioritized sources to consult

Read these first (primary sources):

- Iced subscriptions and lifetime/identity: runtime starts subscriptions asynchronously and kills a stream when you stop returning it. citeturn6view3  
- Iced timing: `iced::time::every` produces periodic messages; available on tokio/smol/wasm. citeturn16view1  
- Iced scroll telemetry: `Scrollable::on_scroll` provides `Viewport`. citeturn11view2  
- Iced viewport geometry: `absolute_offset`, `bounds`, `content_bounds`. citeturn6view1  
- Iced sensor for lazy “near viewport” loading: `Sensor::anticipate` described explicitly for lazy loading. citeturn6view2  
- Iced image handle semantics: `Path` reads from file path; `from_path` examines file data. citeturn9view1  

Then consult these for “how others solved it”:

- Iced community on large scrollables: default approach lags with thousands because it renders everything; list widget direction. citeturn8view0  
- COSMIC virtual list reference implementation: batch offset computation and `visible_elements` splicing. citeturn4view0turn4view1  
- Iced image regression issues demonstrating handle recreation hazards and lockups: citeturn8view2turn10view1  

Windows performance guidance for goal-setting and measuring:

- Startup should end when users can interact meaningfully; defer work; do long-running work independently. citeturn7view0  
- Windows responsiveness optimization recommends defining scenarios and adding start/stop events for measurements. citeturn7view1  
- Win32 message queue behavior explains why long operations prevent responsiveness. citeturn14view1turn14view0