# Storage Architecture

SQLite is the primary persistent store for Librapix-owned metadata.

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
