# Storage Architecture

SQLite is the primary persistent store for Librapix-owned metadata.

## Read-model query ordering and caps

The `list_media_read_models` / `query_media_read_models` functions use a balanced query to ensure:
- **Per-root cap**: Up to 10,000 items per source root (via ROW_NUMBER() PARTITION BY source_root_id), so all active roots are represented.
- **Per-kind cap**: Up to 5,000 images and 5,000 videos per root (via ROW_NUMBER() PARTITION BY source_root_id, media_kind), so "All" includes both when both exist.
- **Ordering**: Results ordered by `modified_unix_seconds DESC, absolute_path ASC` for most-recent-first display.

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

This schema is intentionally minimal to avoid overbuilding before indexing and search are implemented.

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
- This read layer is UI-agnostic and replaceable by richer search subsystems later.

## Tag workflow baseline

- Tags can be listed from storage (`tags` table).
- Tags can be attached/detached to media via `media_tags`.
- Tag kind supports `app` and `game`.
- Tag mutations affect only Librapix-managed storage.

## Root-level auto-tags

- The `source_root_tags` table stores per-root default tags.
- Each root can have zero or more auto-tags with a name and kind (`app` or `game`).
- During indexing, `ensure_root_tags_exist()` creates any missing tag records, then `apply_root_auto_tags()` attaches them to all non-missing media under the corresponding root.
- Auto-tags are applied via `INSERT OR IGNORE`, so they do not duplicate existing manual tags with the same name.
- Root tags are managed through the sidebar UI when a root is selected.
- This design is non-destructive: removing a root tag removes the rule but does not strip existing tags from media.

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
