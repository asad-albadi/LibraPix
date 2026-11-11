# Layers

## Layer responsibilities

- Bootstrap (`librapix-app`)
  - Process startup
  - Iced application wiring
  - Theme/window/runtime integration
  - Startup release-check scheduling (post-first-render task dispatch)
- Presentation (`librapix-app`)
  - View composition
  - Message emission from user actions
  - App-shell composition and design-token usage
  - App-local `ui/tokens` and `ui/styles` modules, re-exported through the existing `ui` facade without changing dependency direction
  - `ui/components` provides message-generic constructors over built-in Iced widgets; styles own variant/status mapping and components own compact layout. Components receive localized text and messages, never application services.
  - No direct storage/indexing logic
  - `views/shell.rs` composes product state into shell regions and existing dialog overlays; `ui/patterns/sidebar.rs` and `details_pane.rs` accept only presentation content and messages. Pane persistence and all background orchestration remain in the app update layer.
  - `ui/patterns/dialog.rs`, `field.rs`, and `status.rs` centralize bounded modal frames, labeled controls with help/error precedence, and truthful status presentation. Callers retain localization, widget IDs, messages, overlay semantics, and background state.
  - `views/dialogs/` contains one view module per product dialog and the shared modal overlay wrapper. These modules only read presentation state and emit existing messages; validation, config/storage access, request building, and process cancellation remain in application orchestration.
- Application orchestration (`librapix-core::app`)
  - Explicit app state and state transitions
  - Route-level coordination
  - Library root and media selection orchestration state
  - Header update-chip state transitions and manual/periodic check policy
- Domain (`librapix-core::domain`)
  - Product invariants and non-destructive rules
  - Media/tag concepts (expanding in next phases)
- i18n (`librapix-i18n`)
  - Text keys, locale catalogs, fallback behavior
- Config (`librapix-config`)
  - Typed config model
  - Config file persistence and validation
  - Platform path resolution for config/data/cache defaults
- Indexing (`librapix-indexer`)
  - Scan orchestration against eligible source roots
  - Centralized ignore-rule matching
  - Candidate production for storage persistence
- Search (`librapix-search`)
  - Replaceable search strategy contracts
  - Baseline fuzzy-capable ranking over read-model documents
- Projections (`librapix-projections`)
  - Timeline grouping projections (day/month/year)
  - Gallery filtering/sorting projections
- Thumbnails (`librapix-thumbnails`)
  - Deterministic cache-key/path generation
  - Read-only image thumbnail rendering pipeline
- Video tools (`librapix-video-tools`)
  - Make Short request model and validation
  - ffprobe duration probing
  - Script-parity filter and ffmpeg argument construction
  - Background process execution wrapper for short generation
- Infrastructure (`librapix-storage` + future crates/modules)
  - SQLite persistence and migrations
  - Source root and ignore-rule persistence baseline
  - Search-facing read-model queries over indexed media and tags
  - Richer search ranking/fuzzy behavior (future phases)

## Dependency direction

- `librapix-app` -> `librapix-core`, `librapix-i18n`, `librapix-config`, `librapix-storage`, `librapix-indexer`, `librapix-video-tools` (orchestration-only usage)
- `librapix-core` -> no UI/framework dependencies
- `librapix-i18n` -> no UI/framework dependencies
- `librapix-config` -> no UI/framework dependencies
- `librapix-storage` -> no UI/framework dependencies
- `librapix-indexer` -> no UI/framework dependencies

UI and domain must not depend on persistence implementation details.
