# Message Flow

Current baseline follows Iced's explicit state/update/view loop.

## Flow

1. `view` renders controls from current `AppState`.
2. User action emits a UI message.
3. `update` maps UI message to `librapix-core::app::AppMessage`.
4. `AppState::apply` performs an explicit transition.
5. Next `view` reflects updated state.

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
- Run indexing baseline
  - App loads eligible roots and enabled ignore patterns from storage.
  - `librapix-indexer` scans and emits media candidates.
  - App persists candidates incrementally to `indexed_media`, marks missing records, and records indexing summary in state.
  - App runs thumbnail generation for image read-model rows into app-owned thumbnail cache.
- Run read-model query baseline
  - App queries read models from storage with optional text filtering over path/tag data.
  - App renders a small preview list for verification, keeping UI logic thin.
- Run timeline projection baseline
  - App loads read-model rows.
  - App delegates grouping to `librapix-projections`.
  - UI renders bucket preview only.
- Run gallery projection baseline
  - App loads read-model rows.
  - App delegates filtering/sorting to `librapix-projections`.
  - UI renders row preview only.
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

## Rules

- Message handling remains explicit and testable.
- Side effects are introduced as tasks intentionally, not hidden in widgets.
- Storage/indexing/search side effects will be delegated to dedicated subsystems in future phases.
