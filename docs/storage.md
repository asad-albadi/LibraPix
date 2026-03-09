# Storage (Alias)

This file exists as a stable alias for storage architecture documentation.

- Canonical document: `docs/architecture/storage.md`
- Implementation crate: `crates/librapix-storage`

Key current behavior:

- SQLite remains the system of record for Librapix-managed metadata.
- Projection snapshots are persisted in `projection_snapshots` for fast startup hydrate.
- Read-model query paths include unbounded aggregate reads plus chunked batched lookups for large ID/path sets.
- Missing/corrupt snapshot payloads degrade safely to background reconcile.
