# Message Flow

Current baseline follows Iced's explicit state/update/view loop.

Issue `#12` runtime rationale and evidence are consolidated in `docs/architecture/issue-12-runtime-optimization-summary.md`. This document describes the final runtime flow only.

## Flow

1. `view` renders controls from current `AppState`.
2. User action emits a UI message.
3. `update` maps UI message to `librapix-core::app::AppMessage`.
4. `AppState::apply` performs an explicit transition.
5. Next `view` reflects updated state.

The current shell uses header/sidebar/main/details regions to separate navigation, management, browsing, and item actions.

## Library root orchestration baseline

- Add root (library dialog, add mode)
  - Browse-first picker (with optional manual path input) captures path in dialog state.
  - Save normalizes path, upserts source root, applies display name, and syncs root-level tags.
  - Save can keep dialog open for batch add (`Save + Add Another`) or close after commit.
- Edit root (library dialog, edit mode)
  - Explicit `Edit` action opens dialog with current path/display-name/tags preloaded.
  - Save updates normalized path and lifecycle to `active`, updates display name, and syncs root-level tags.
- View root stats (library statistics dialog)
  - Explicit `Stats` action opens a dedicated read-only dialog for the selected root.
  - Dialog reads maintained persisted rows from storage (`get_source_root_statistics`) and does not trigger re-indexing or heavy recomputation.
- Deactivate/reactivate/remove root
  - Lifecycle updates are explicit storage operations (available from edit dialog actions).
  - Remove deletes only Librapix-managed records.
- Refresh roots
  - Reconciliation runs and the root list is reloaded into app state.
- Ignore-rule management
  - UI captures ignore pattern text and chip edit state.
  - App adds/toggles/removes/edits ignore-rule rows in storage.
  - App refreshes ignore-rule chip rows with enabled/disabled semantics preserved.
- Run indexing baseline
  - App loads eligible roots and enabled ignore patterns from storage.
  - `librapix-indexer` scans and emits media candidates.
  - App persists candidates incrementally to `indexed_media`, marks missing records, and records indexing summary in state.
  - App refreshes maintained `source_root_statistics` for scanned roots.
  - App refreshes `media_catalog` so browse/search/timeline work reads a normalized catalog layer instead of shaping raw source facts directly.
  - App runs thumbnail generation for catalog rows and records named thumbnail variants in `derived_artifacts`.
- Run read-model query baseline
  - App schedules projection/search background work via `Task::perform`.
  - Worker queries normalized catalog rows from storage with optional root filtering.
  - App executes fuzzy search over the full in-memory catalog document set (no hidden fixed 20-result cap).
  - App applies active kind/extension/tag filters to resulting hits.
- Run timeline projection baseline
  - App schedules projection-only background work.
  - Worker loads catalog rows, delegates grouping to `librapix-projections`, and applies kind/extension/tag filters.
  - Projection prefers persisted timeline keys from `media_catalog` and falls back to timestamp conversion only when needed.
  - Worker derives `TimelineAnchor` metadata from timeline buckets (`build_timeline_anchors`).
  - UI renders selectable timeline items grouped by route panel when staged projection results are applied.
- Timeline scrubber interaction
  - Timeline pane owns a stable scrollable `Id` (`media-pane-scrollable`).
  - Timeline/gallery media scrollable uses embedded vertical scrollbar spacing so scrollbar gutter is outside card content.
  - Drag/click on scrubber emits `TimelineScrubChanged` (continuous) and `TimelineScrubReleased`.
  - Scrub value is continuous (`0.0..=1.0`) and nearest-anchor selection is derived from ordered `TimelineAnchor.normalized_position` values.
  - Scrubber date-chip vertical placement follows continuous scrub value while label selection uses nearest anchor; chip lane width is stable across pointer-down/drag state.
  - App issues Iced widget operations (`operation::scroll_to`, with relative `snap_to` fallback) so scrub dragging and programmatic jumps share one mapping model.
  - Scroll updates emit `MediaViewportChanged`; scrub value tracks viewport offset continuously while anchor selection tracks nearest projection anchor.
- Run gallery projection baseline
  - App schedules projection-only background work.
  - Worker loads catalog rows, delegates filtering/sorting to `librapix-projections`, and applies `GalleryQuery`.
  - UI renders selectable gallery items by route panel when staged projection results are applied.
  - Large gallery/timeline/search surfaces now render through a viewport-bounded window with preserved scroll extent, so full logical result sets can apply without composing every card in one frame.
- Direct media selection
  - Selection is explicit app state (`selected_media_id`).
  - Selecting from search/gallery/timeline loads details and enables actions/tags.
- Load media details
  - UI provides selected media id.
  - App resolves media details from storage read-model lookup.
  - If startup snapshot state has not warmed the selected media into `media_cache`, details now stay placeholder-first:
    - prefer an already-existing `detail-800` file when present
    - otherwise reuse the current browse thumbnail immediately
    - do not synchronously generate a new detail thumbnail on the selection path
  - UI renders metadata lines and action status.
- Tag actions
  - UI provides selected media id + tag text or chip action (edit/remove).
  - App attaches/detaches app or game tags through storage APIs.
  - Tag edit flow is explicit (`detach old` + `attach new` with preserved kind).
  - Updated tags are reflected by reloading selected media details and chip state.
- Open/copy actions
  - App resolves selected media path from storage.
  - App invokes platform-specific open/clipboard commands.
  - Windows copy-file uses native `CF_HDROP` clipboard payload writes (`SetClipboardData`) instead of shelling out to PowerShell.
  - Copy flow supports both path clipboard and file-object clipboard actions.
- Make Short action (video-only)
  - Details pane exposes `Make Short` only when the selected media kind is video.
  - Dialog captures output path, script-equivalent effects, crop, fade, speed, CRF, and preset.
  - App builds `ShortGenerationRequest` and dispatches background tasks with `Task::perform`.
  - Stage-based dialog status transitions are explicit (`Preparing` -> `Probing` -> `Building filters` -> `Generating` -> `Finalizing`).
  - ffprobe/ffmpeg validation/probe/filter/args/process logic stays in `librapix-video-tools`.
  - Completion feeds explicit success/failure dialog state with open-file/open-folder follow-up actions.
- Keyboard shortcuts
  - App subscribes to ignored keyboard events via `keyboard::listen`.
  - `Cmd/Ctrl+C` routes to copy-selected-file.
  - `Cmd/Ctrl+Shift+C` routes to copy-selected-path.
  - Ignored-only subscription prevents conflicts with focused text input widgets.
- Startup restore
  - `Task::done(Message::StartupRestore)` fires after the first render.
  - Bootstrap now loads config/theme/path overrides only; storage open and migrations were removed from the pre-render path.
  - Startup restore first hydrates the persisted startup snapshot on a background task.
  - If a compatible snapshot exists, app applies it incrementally through `SnapshotApplyTick`, but only for a bounded recent-gallery slice instead of the full gallery+timeline browse state.
  - Incompatible legacy snapshots are discarded and rebuilt after the next successful unfiltered gallery projection instead of being eagerly rehydrated.
  - Startup then schedules a delayed reconcile kickoff (`StartupReconcileKickoff`) when hydrated roots exist.
  - Reconcile and projection now run as explicit staged jobs:
    - `ScanJobComplete`
    - `ProjectionJobComplete`
    - `ThumbnailBatchComplete`
  - Startup restore also schedules a non-blocking GitHub latest-release check task.
  - Startup projection now follows a ready-enough policy:
    - the startup-critical projection refresh prioritizes the currently visible browse surface
    - non-visible route refresh can remain deferred until explicitly requested or later background catch-up
    - startup cache warm-up is bounded to a small visible slice instead of the full catalog
    - if reconcile finds no catalog changes and the current route is already the restored default gallery snapshot, startup skips the redundant startup-blocking gallery projection, becomes ready from that snapshot, and schedules a non-blocking gallery continuation so the snapshot restore does not become the permanent gallery state
  - Startup projection now performs explicit thumbnail lookup before scheduling generation:
    - exact ready `gallery-400` artifact rows
    - deterministic on-disk `gallery-400` files
    - compatible `detail-800` fallback rows
    - deterministic `detail-800` fallback for visible-priority items
  - Thumbnail work is now split:
    - startup-priority background thumbnail work for the first visible image slice
    - delayed background catch-up for visible videos and the remaining browse-tier backlog
  - Startup/runtime instrumentation now writes a timestamped log file with:
    - bootstrap config-load timing
    - storage open + migration timing
    - snapshot hydrate/apply timing
    - reconcile/projection timing
    - thumbnail start/end timing
    - thumbnail batch dispatch/start/end/cancel timing
    - thumbnail worker-complete -> dispatch-to-UI -> message-received handoff timing
    - thumbnail apply start timing plus apply duration
    - thumbnail apply timing in the app state
    - thumbnail result-message rate during active background work
    - artifact lookup start/end timing
    - exact/fallback reuse counts
    - placeholder and scheduled-generation counts
    - rejected-artifact reasons
    - video command/exit/timeout/stderr details on failure
    - startup-ready and first-usable-gallery milestones
  - UI remains interactive while background work proceeds; sidebar activity state reflects the real stage currently in flight.
  - Ready-enough state is restored after snapshot apply, reconcile, and current-surface projection; thumbnail work continues honestly in background instead of blocking startup-ready.
  - Unchanged startup launches no longer force gallery-loading ownership after reconcile:
    - `apply_scan_job_result(...)` skips projection when the restored gallery snapshot is still valid for the default route
    - timeline refresh remains deferred until the user opens Timeline
  - Deferred thumbnail catch-up remains visible as honest background work without keeping the whole app in startup-busy state.
  - Later projection or reconcile requests cancel queued thumbnail work and invalidate stale in-flight batches instead of waiting for them to settle first.
  - Failed thumbnails now enter runtime backoff so later projection refreshes do not immediately retry the same known-bad items.
  - ffmpeg resolution/spawn failures disable repeated video attempts for the rest of the session instead of flooding the app with broken subprocess launches.
- Release update check flow
  - State is explicit in app orchestration: `Unknown`, `Checking`, `UpToDate`, `UpdateAvailable { version, url }`, `Failed`.
  - Header chip click emits `UpdateChipPressed`.
  - `UpdateChipPressed` behavior:
    - opens the latest release URL immediately when `UpdateAvailable` is active
    - otherwise attempts a manual re-check (rate-limited to 5 minutes)
  - A periodic tick subscription emits `UpdateCheckTick`.
  - Periodic runtime timers now use `iced::time::every(...)` on the Iced tokio backend instead of custom blocking `std::thread::sleep(...)` subscription loops.
  - `UpdateCheckTick` enforces a 24-hour auto re-check policy while preventing overlapping checks.
  - Background task completion emits `UpdateCheckCompleted`, which applies the new update-check state and successful-check timestamp.
- Library dialog folder picker
  - `Message::LibraryDialogBrowseFolder` opens a native OS folder picker via `rfd::FileDialog`.
  - Selected path is written into dialog path state.
- Double-click open
  - `Message::SelectMedia(id)` detects double-click by comparing click timestamp.
  - If the same media is clicked within 400ms, opens the file in the default external app.
  - Otherwise, performs normal selection and detail loading.
- Auto-refresh
  - After adding a root, indexing and gallery refresh run automatically.
  - After removing a root, gallery refreshes automatically.
  - After indexing completes, gallery projection refreshes automatically (within StartupRestore flow).
  - Filesystem-triggered background refresh compares previous and current media cache ids to detect newly indexed files.
  - New files can trigger a dismissible in-app modal dialog with preview/metadata and quick actions.
  - Post-startup route/filter/search/filesystem projection refreshes are current-surface-first:
    - the active route is rebuilt immediately
    - the non-visible route is marked deferred and rebuilt only when opened later
    - projection logs record trigger, policy, refreshed surfaces, and working-state ownership

- Background work pattern
  - Heavy operations still run off the UI thread through `Task::perform`, but the branch now uses staged job families instead of one monolithic worker result.
  - Startup/runtime stages are explicit:
    - snapshot hydrate
    - snapshot apply
    - scan/reconcile
    - catalog-backed projection/search preparation
    - thumbnail batches
  - Catalog-backed projection work refreshes `media_catalog`, queries normalized rows, reads ready derived artifacts, and then schedules missing thumbnail work separately.
  - During startup, projection refresh is intentionally narrower:
    - current route first
    - visible-slice cache warm-up
    - deferred non-visible route refresh
  - Activity state is structural runtime state, not a widget-local hint; stage text and ready transitions are driven by the coordinator state machine.
  - Periodic coordinator ticks (`UpdateCheckTick`, `StartupReconcileKickoff`, `SnapshotApplyTick`, `DeferredThumbnailCatchupKickoff`) now use Iced's timer API instead of blocking timer streams, so background task completions are not left waiting behind sleeping subscription workers.
  - Ready-enough and background catch-up are now distinct runtime concepts:
    - startup completion no longer waits for any thumbnail batch
    - deferred thumbnail catch-up is delayed and batched more lightly after startup becomes usable
    - unchanged snapshot-backed startup can still schedule a later non-blocking gallery continuation for completeness
  - Background thumbnail policy is no longer uniform:
    - images can remain higher-priority background work
    - videos are throttled more aggressively, with one-item batches plus backoff/cancellation
  - Thumbnail batch completion is now explicitly observable end-to-end:
    - worker finish
    - dispatch to UI
    - message received in `update`
    - apply start/end in app state
  - Current branch limitation: the staged coordinator still lives in `librapix-app/src/main.rs` and has not yet been extracted into smaller orchestration modules.
  - Release checks continue to follow the same non-blocking task model (`Task::perform`) through `start_update_check` -> `UpdateCheckCompleted`.

## Rules

- Message handling remains explicit and testable.
- Side effects are introduced as tasks intentionally, not hidden in widgets.
- Heavy operations use `Task::perform` to avoid blocking the UI thread.
- Storage/indexing/search side effects will be delegated to dedicated subsystems in future phases.
- Double-click detection is app-level state, not widget-level behavior.
