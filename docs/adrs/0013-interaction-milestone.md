# ADR 0013: Interaction Milestone Architecture Decisions

## Context

Librapix has a working architecture, indexing pipeline, projections, search, and a Fluent-inspired UI shell. However, it lacked key product-quality interaction behaviors: startup state restoration, folder picker UX, double-click open, human-readable metadata formatting, and background activity indicators.

This ADR records the architecture decisions made during the interaction milestone.

## Decisions

### 1. Startup restore via Task::done

On app initialization, the init function returns `Task::done(Message::StartupRestore)` which triggers auto-indexing and gallery/timeline projection on the first update cycle. This means the app renders once with empty state, then populates from persisted data.

Alternative considered: running indexing synchronously during `Default::default()`. Rejected because it delays the first render and provides no visual feedback.

Future improvement: async indexing would allow a loading indicator during startup restore.

### 2. Folder picker via rfd (synchronous)

Library root management uses `rfd::FileDialog::pick_folder()` which opens a native OS folder selection dialog. The synchronous API is used because folder selection dialogs are inherently modal and blocking.

Alternative considered: `rfd::AsyncFileDialog` with `Task::perform()`. This would require adding an async runtime (tokio feature on iced). Rejected as unnecessary complexity for a modal dialog.

### 3. Double-click detection at app level

Iced buttons do not have native double-click events. Double-click is detected by tracking `last_click_media_id` and `last_click_time` in app state. If the same media item is clicked within 400ms, it is treated as a double-click and the file is opened in the OS default application.

Single click remains selection-focused. Double-click opens the file. This matches standard desktop media application behavior.

### 4. Centralized formatting module

Human-readable formatting for file sizes, timestamps, and dimensions is centralized in `crates/librapix-app/src/format.rs`. This keeps formatting logic reusable and testable, separate from view composition.

`chrono` (already a workspace dependency for projections) is used for timestamp formatting with local timezone conversion.

### 5. Activity status in header

A simple `activity_status: String` field in app state is used to show background activity in the header. When non-empty, it displays as a subtle accent-colored caption next to the search bar. Cleared when the operation completes.

Alternative considered: a dedicated activity enum with structured progress. Deferred as unnecessary complexity for the current synchronous operation model.

### 6. Auto-refresh after operations

Gallery and timeline are automatically refreshed after indexing completes and after root management operations (add, remove). This ensures the user sees current data without manual refresh steps.

Periodic file-system watching is deferred. Current auto-refresh is event-driven: on startup, after root management, and after manual indexing.

## Consequences

- App startup now shows prior library state automatically.
- Users can browse for folders instead of typing paths.
- Double-click opens files naturally.
- Metadata is human-readable throughout.
- Background operations are visually indicated.
- No new async runtime dependency is required.
- Periodic file watching remains a future enhancement.
