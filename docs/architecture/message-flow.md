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

- Add root
  - UI message captures path input.
  - App normalizes path and upserts source root in storage.
  - Reconciliation refreshes root lifecycle states.
- Update root
  - UI selects root id and submits new path.
  - Storage updates normalized path and lifecycle to `active`.
- Deactivate/reactivate/remove root
  - Lifecycle updates are explicit storage operations.
  - Remove deletes only Librapix-managed records.
- Refresh roots
  - Reconciliation runs and the root list is reloaded into app state.
- Ignore-rule management
  - UI captures ignore pattern text.
  - App enables/disables rule rows in storage.
  - App refreshes current ignore-rule list preview.
- Run indexing baseline
  - App loads eligible roots and enabled ignore patterns from storage.
  - `librapix-indexer` scans and emits media candidates.
  - App persists candidates incrementally to `indexed_media`, marks missing records, and records indexing summary in state.
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
  - Drag/click on scrubber emits `TimelineScrubChanged` (continuous) and `TimelineScrubReleased`.
  - Scrub value is continuous (`0.0..=1.0`) and nearest-anchor selection is derived from ordered `TimelineAnchor.normalized_position` values.
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
  - UI provides selected media id + tag text.
  - App attaches/detaches app or game tags through storage APIs.
  - Updated tags are reflected by reloading selected media details.
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
  - UI remains interactive while background work proceeds; activity status shown in header.
  - On completion, `BackgroundWorkComplete` message applies all results to app state atomically.
- Folder picker
  - `Message::BrowseFolder` opens a native OS folder picker via `rfd::FileDialog`.
  - Selected path is written into root input state.
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
  - Multiple handlers share this pattern: `StartupRestore`, `FilesystemChanged`, `RunIndexing`, `ApplyMinFileSize`, `AddRoot`, auto-tag operations, manual route refresh, search run, and filter changes.

## Rules

- Message handling remains explicit and testable.
- Side effects are introduced as tasks intentionally, not hidden in widgets.
- Heavy operations use `Task::perform` to avoid blocking the UI thread.
- Storage/indexing/search side effects will be delegated to dedicated subsystems in future phases.
- Double-click detection is app-level state, not widget-level behavior.
