# ADR 0017: Async Startup and Background Work

## Status

Accepted

## Context

When Librapix is launched with multiple library roots containing thousands of images, the startup sequence blocks the UI thread while running:
1. Filesystem scanning across all roots
2. Incremental indexing and SQLite writes
3. Thumbnail generation for all indexed media
4. Gallery and timeline projection computation

On Windows, this manifests as "Not Responding" in the title bar until all work completes. The same blocking pattern affects `FilesystemChanged`, `RunIndexing`, `ApplyMinFileSize`, `AddRoot`, and auto-tag operations.

Additionally, hard-coded query limits (200 for thumbnails, 500 for projections, 120 for gallery display) silently truncated multi-library results, undermining the core multi-library aggregation feature.

## Decision

### Non-blocking startup via Task::perform

All heavyweight operations are moved off the UI thread using Iced's `Task::perform`:

- `spawn_background_work(app)` captures current app inputs (database path, thumbnails dir, filters, etc.) as owned/cloned values.
- `do_background_work(...)` is a standalone function that opens its own `Storage` connection and performs all scanning, indexing, thumbnail generation, and projection computation.
- Results are returned in a `BackgroundWorkResult` struct and applied atomically via `apply_background_result` when the `BackgroundWorkComplete` message arrives.

### Unified limit constant

A single `MEDIA_QUERY_LIMIT` constant (50,000) replaces all hard-coded query limits across thumbnail generation, projections, and search.

### Sync vs async split

Lightweight operations (filter changes, manual projection refresh) remain synchronous since they only perform fast DB queries. Only heavyweight operations that involve filesystem scanning, thumbnail I/O, or full re-indexing use the async path.

## Alternatives Considered

1. **std::thread::spawn with channels**: Would work but bypasses Iced's task infrastructure and requires manual channel management.
2. **Splitting startup into multiple Task::done phases**: Doesn't help because `Task::done` runs synchronously in the next update cycle.
3. **Incremental/streaming results**: More complex; deferred to future phases if needed for very large libraries.

## Consequences

- The UI is immediately interactive on startup; users see persisted state while background work proceeds.
- Multiple concurrent background work invocations are safe since each opens its own Storage connection; the last result to complete wins.
- Gallery and timeline now correctly aggregate all active libraries without artificial truncation.
- The `run_indexing` function was removed as dead code since all callers now use `spawn_background_work`.
