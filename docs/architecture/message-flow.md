# Message Flow

Current baseline follows Iced's explicit state/update/view loop.

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
  - App runs thumbnail generation for image read-model rows into app-owned thumbnail cache.
- Run read-model query baseline
  - App schedules projection/search background work via `Task::perform`.
  - Worker queries read models from storage with optional text filtering over path/tag data.
  - App executes fuzzy search over the full in-memory read-model document set (no hidden fixed 20-result cap).
  - App applies active kind/extension/tag filters to resulting hits.
- Run timeline projection baseline
  - App schedules projection-only background work.
  - Worker loads read-model rows, delegates grouping to `librapix-projections`, and applies kind/extension/tag filters.
  - Worker derives `TimelineAnchor` metadata from timeline buckets (`build_timeline_anchors`).
  - UI renders selectable timeline items grouped by route panel when `BackgroundWorkComplete` is applied.
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
  - Worker loads read-model rows, delegates filtering/sorting to `librapix-projections`, and applies `GalleryQuery`.
  - UI renders selectable gallery items by route panel when `BackgroundWorkComplete` is applied.
- Direct media selection
  - Selection is explicit app state (`selected_media_id`).
  - Selecting from search/gallery/timeline loads details and enables actions/tags.
- Load media details
  - UI provides selected media id.
  - App resolves media details from storage read-model lookup.
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
- Keyboard shortcuts
  - App subscribes to ignored keyboard events via `keyboard::listen`.
  - `Cmd/Ctrl+C` routes to copy-selected-file.
  - `Cmd/Ctrl+Shift+C` routes to copy-selected-path.
  - Ignored-only subscription prevents conflicts with focused text input widgets.
- Startup restore
  - `Task::done(Message::StartupRestore)` fires after the first render.
  - If roots exist, spawns background work via `Task::perform` to run indexing, thumbnail generation, and gallery/timeline projections on a background thread.
  - Startup restore also schedules a non-blocking GitHub latest-release check task.
  - UI remains interactive while background work proceeds; activity status shown in header.
  - On completion, `BackgroundWorkComplete` message applies all results to app state atomically.
- Release update check flow
  - State is explicit in app orchestration: `Unknown`, `Checking`, `UpToDate`, `UpdateAvailable { version, url }`, `Failed`.
  - Header chip click emits `UpdateChipPressed`.
  - `UpdateChipPressed` behavior:
    - opens the latest release URL immediately when `UpdateAvailable` is active
    - otherwise attempts a manual re-check (rate-limited to 5 minutes)
  - A periodic tick subscription emits `UpdateCheckTick`.
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

- Background work pattern
  - Heavy operations (indexing, scanning, thumbnail generation, projections, search hydration) are encapsulated in `do_background_work`.
  - `spawn_background_work` captures current app inputs in `BackgroundWorkInput` and returns a `Task::perform` that runs the work off the UI thread.
  - Work mode is explicit:
    - `IndexAndProject`: indexing + thumbnails + projections/search
    - `ProjectOnly`: projections/search refresh without filesystem scan/index writes
  - `BackgroundWorkComplete` handler applies all returned state atomically.
  - Multiple handlers share this pattern: `StartupRestore`, `FilesystemChanged`, `RunIndexing`, `ApplyMinFileSize`, library-dialog save operations, manual route refresh, search run, and filter changes.
  - Release checks follow the same non-blocking task model (`Task::perform`) through `start_update_check` -> `UpdateCheckCompleted`.

## Rules

- Message handling remains explicit and testable.
- Side effects are introduced as tasks intentionally, not hidden in widgets.
- Heavy operations use `Task::perform` to avoid blocking the UI thread.
- Storage/indexing/search side effects will be delegated to dedicated subsystems in future phases.
- Double-click detection is app-level state, not widget-level behavior.
