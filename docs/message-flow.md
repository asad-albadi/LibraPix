# Message Flow (Alias)

This file exists as a stable alias for message-flow architecture documentation.

- Canonical document: `docs/architecture/message-flow.md`
- Runtime implementation: `crates/librapix-app/src/main.rs`

Key current behavior:

- Startup is two-phase (`HydrateSnapshot` then delayed `StartupReconcileKickoff`).
- If hydrate has no persisted browse snapshot, projection bootstraps first and reconcile follows from pending state.
- Background work is staged (`ScanJobComplete`, `ProjectionJobComplete`, `ThumbnailBatchComplete`), not a single monolithic completion.
- Generation guards prevent stale stage completions from overwriting newer UI state.
- Filesystem watcher events are coalesced through pending reconcile/projection coordination.
- Thumbnail retry wakeups are self-driven via timer ticks (`ThumbnailRetryTick`) and rebase to the current thumbnail generation when valid.
