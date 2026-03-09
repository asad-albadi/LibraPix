# Indexing Architecture

Indexing is a dedicated subsystem (`librapix-indexer`) isolated from rendering.

## Core behavior

- Indexing is read-only with respect to source media files.
- Ignore rules are centralized (`IgnoreEngine`) and applied before metadata extraction.
- Min-size filtering remains part of scan options (`ScanOptions.min_file_size_bytes`).
- Recursive traversal has no depth cap (`walkdir`).
- Incremental change detection compares `absolute_path + file_size_bytes + modified_unix_seconds`.
- Non-seen files under scanned roots are marked `missing` in Librapix storage.

## Staged runtime integration

Indexing now runs as one stage of the staged background runtime:

1. `HydrateSnapshot`
2. `ScanRootsIncremental`
3. `RefreshProjection` (conditional)
4. `GenerateThumbnailBatch` (queued/coalesced)

`ScanRootsIncremental` (`do_scan_job`) performs:

1. Root availability reconciliation.
2. Ignore-rule loading.
3. Eligible-root scan via `scan_roots`.
4. Incremental writes via `apply_incremental_index`.
5. Tag maintenance (`ensure_media_kind_tags_attached`, root auto-tags).
6. Maintained statistics refresh (`refresh_source_root_statistics`).
7. Thumbnail candidate derivation:
   - changed/new media rows
   - currently visible media IDs (for startup snapshot and active UI recovery)

## Correctness and responsiveness guarantees

- Indexing runs on background tasks (`Task::perform`) and never in widgets.
- Results are generation-guarded when applied; stale completions are ignored.
- Filesystem watcher bursts are coalesced into pending reconcile requests.
- Projection refresh is conditional:
  - runs when media changed
  - runs when projection is missing/stale/explicitly requested
  - otherwise skipped to avoid unnecessary full projection rebuilds

## Query cardinality policy

- Aggregation-critical paths use unbounded read-model APIs (no hidden result cap).
- Large ID/path batched lookups are chunked under SQLite parameter limits for scale safety.
- Bounded APIs (`limit/offset`) remain available for explicitly paginated call sites only.
