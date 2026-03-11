# Storage Architecture

SQLite is the primary persistent store for Librapix-owned metadata.

## Read-model query ordering and browse semantics

- `list_media_read_models(limit, offset)` is a paginated read API for bounded callers.
- `list_all_media_read_models()` returns all non-missing media rows without SQL `LIMIT`.
- Ordering is `modified_unix_seconds DESC, absolute_path ASC` for most-recent-first display.
- Browse/index/search orchestration paths that must aggregate all active roots and both media kinds use the unbounded API.
- "All" browse semantics are implemented as no media-kind predicate (no implicit image-only mapping).

## Scope of app-managed storage

- Library registrations
- App-side tags and game tags
- Search indexes
- Memories/resurfacing data
- Ignore rules
- UI/app preferences

## Hard constraints

- Never write app metadata into source media files.
- Never modify user media file names or locations.
- Keep persistence concerns isolated from view code.

## Baseline implementation

- Crate: `librapix-storage`
- Database: SQLite through `rusqlite`
- Migration tracking table: `schema_migrations`
- Baseline migration: `0001_baseline.sql`

## Baseline schema scope

- `source_roots`
  - normalized absolute source paths
  - lifecycle state (`active`, `unavailable`, `deactivated`)
  - availability check timestamp
  - optional `display_name` for user-defined library labels (migration 0007)
- `app_settings`
  - key/value settings that fit DB-backed storage
- `ignore_rules`
  - scope + pattern + enabled marker
- `indexed_media`
  - indexed path records and metadata baseline:
    - `absolute_path`
    - `media_kind`
    - `source_root_id`
    - `file_size_bytes`
    - `modified_unix_seconds`
    - optional `width_px` / `height_px`
    - `metadata_status`
    - `last_seen_at` / `missing_since`
- `tags` / `media_tags`
  - minimal tag-readiness schema for search-facing read models
- `source_root_statistics` (migration `0008`)
  - persisted per-library summary/indexing metrics:
    - `total_size_bytes`
    - `total_media_count`
    - `total_images_count`
    - `total_videos_count`
    - `total_image_size_bytes`
    - `total_video_size_bytes`
    - `missing_count`
    - `oldest_modified_unix_seconds`
    - `newest_modified_unix_seconds`
    - `last_indexed_unix_seconds`

This schema is intentionally minimal to avoid overbuilding before indexing and search are implemented.

## Catalog-first foundation

Implemented in `feat/catalog-first-architecture`:

- `indexed_media` remains the source-facts table.
- `media_catalog` materializes normalized browse/search/timeline fields:
  - `file_name`
  - `file_extension`
  - `tags_csv` (serialized tag payload; legacy column name)
  - `search_text`
  - `timeline_day_key`
  - `timeline_month_key`
  - `timeline_year_key`
- `derived_artifacts` records app-owned generated outputs such as thumbnail variants.

Partially implemented:

- catalog refresh is currently a storage-owned materialization pass (`refresh_catalog()`) that rebuilds normalized rows from source facts and tag joins.

Deferred:

- fully incremental catalog maintenance for every mutation path
- aggregate timeline rollup tables
- alternate search indexes or FTS storage

## Source root ownership policy

- Current policy: source roots are persisted in storage as operational records.
- Config may provide bootstrap roots, but storage is the persistence system of record for library root rows.
- Startup orchestration may sync configured roots into storage using idempotent upsert.

## Read-model baseline

- Storage exposes read-model queries over `indexed_media` joined with `tags`.
- Baseline query surface supports:
  - paginated list of non-missing indexed media
  - search by path/tag text filter
  - media-by-id lookup for details/action orchestration
- App orchestration derives the tag-filter chip list from read-model tag joins (excluding internal `kind:*` tags).
- App-level top media stats (`Total`, `Images`, `Videos`) are derived from the current projected browse/search result set, not from stale persisted counters.
- Large browse/search refreshes read this query surface from background tasks (`Task::perform`) so SQLite reads/projection hydration do not block the UI thread.
- This read layer is UI-agnostic and replaceable by richer search subsystems later.
- Library statistics dialog reads maintained rows from `source_root_statistics` via `get_source_root_statistics(root_id)` and does not run expensive aggregation on dialog open.

## Catalog query surface

Implemented:

- `list_catalog_media_filtered(source_root_id)` returns normalized catalog rows ordered for browse flows.
- `list_ready_derived_artifacts_for_media_ids(...)` returns ready artifact records for named variants such as `gallery-400` and `detail-800`.

Partially implemented:

- details and tag workflows still use the direct read-model lookup path where that remains simpler and correct.

## Library statistics maintenance

- `refresh_source_root_statistics(root_ids)` is called during indexing/re-indexing runs for the scanned roots.
- Aggregation is performed in storage (SQLite), not in the UI layer.
- Current maintained values include totals by kind/size, missing count, oldest/newest modified timestamps, and `last_indexed_unix_seconds`.
- Dialog open path is read-only and fast (single-row lookup per selected root).

## Tag workflow baseline

- Tags can be listed from storage (`tags` table).
- Tags can be attached/detached to media via `media_tags`.
- Media tag rows can be listed with kind metadata (`list_media_tags`) for UI chip rendering and edit flows.
- Tag kind supports `app` and `game`.
- Tag mutations affect only Librapix-managed storage.

## Root-level auto-tags

- The `source_root_tags` table stores per-root default tags.
- Each root can have zero or more auto-tags with a name and kind (`app` or `game`).
- During indexing, `ensure_root_tags_exist()` creates any missing tag records, then `apply_root_auto_tags()` attaches them to all non-missing media under the corresponding root.
- Auto-tags are applied via `INSERT OR IGNORE`, so they do not duplicate existing manual tags with the same name.
- Root tags are managed through the unified library add/edit dialog.
- This design is non-destructive: removing a root tag removes the rule but does not strip existing tags from media.

## Ignore-rule management baseline

- Ignore rules remain scope+pattern rows with `is_enabled` semantics.
- UI chip flows use `upsert_ignore_rule` for add/toggle and `delete_ignore_rule_by_id` for explicit removal.
- Editing a rule pattern is implemented as explicit app-orchestrated upsert/remove operations, preserving enabled state where possible.

## Missing-file policy baseline

- Missing/deleted files are not treated as fatal startup errors.
- Source roots remain recorded and transition to `unavailable` when the path is missing.
- User-driven deactivation transitions roots to `deactivated`.
- Removal is explicit and deletes only Librapix-managed records.
- Future indexing phases will define lifecycle transitions for missing media under existing roots.
- Indexed media missing-file reconciliation marks records as `missing` rather than deleting source data.

## Path handling policy baseline

- Paths are normalized lexically in config before persistence.
- Storage expects absolute normalized paths and rejects empty/relative source roots.
- Canonicalization requiring path existence is intentionally avoided in the config stage to support missing/offline volumes.

## Cache and thumbnails ownership policy

- Cache and thumbnail files are app-owned artifacts only.
- Default location: project cache directory under `thumbnails`.
- Source media directories are never used as cache locations.
- Path overrides are possible through config, but ownership remains app-side only.
- Thumbnail cache file naming uses deterministic derived keys from source path + size + modified timestamp.
- Ready/failed thumbnail variants are now also recorded in `derived_artifacts` so cache state is not inferred from disk presence alone.
