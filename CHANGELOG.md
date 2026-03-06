# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

### Added
- Introduced a workspace layout with `librapix-app`, `librapix-core`, and `librapix-i18n`.
- Added initial architecture documentation set under `docs/architecture/`.
- Added roadmap documentation for MVP and future phases.
- Added ADR `0001` covering workspace boundaries and dependency direction.
- Added an i18n-ready app shell where user-facing text is key-based.
- Added `librapix-config` crate with typed config schema, TOML persistence, validation, and path normalization.
- Added `librapix-storage` crate with SQLite connection handling, migration tracking, and source-root persistence API.
- Added baseline SQLite migration `0001_baseline.sql` with `source_roots`, `app_settings`, and `ignore_rules`.
- Added `docs/architecture/config.md` and a phase checklist for config/storage foundation.
- Added ADR `0002` (config/path policy) and ADR `0003` (SQLite migration baseline).
- Added ADR `0004` defining source-root lifecycle reconciliation policy.
- Added storage migrations:
  - `0002_source_root_lifecycle.sql`
  - `0003_indexed_media_baseline.sql`
- Added `librapix-indexer` crate with scan roots pipeline, media candidate generation, and ignore engine tests.
- Added ADR `0005` for indexing + ignore-rule baseline.
- Added metadata extraction baseline for indexed media (size, modified time, image dimensions where supported).
- Added `docs/architecture/metadata.md`.
- Added ADR `0006` for metadata extraction, incremental indexing policy, and read-model baseline.
- Added tag-readiness baseline tables (`tags`, `media_tags`) and read-model queries.
- Added `librapix-search` crate with replaceable search contracts and baseline fuzzy ranking strategy.

### Changed
- Migrated from a single-crate starter to a multi-crate workspace.
- Declared MSRV `1.85` in workspace metadata and repository docs.
- App bootstrap now loads persisted config, opens storage, and syncs configured source roots.
- Storage now tracks root lifecycle states (`active`, `unavailable`, `deactivated`) and supports availability reconciliation.
- App orchestration now supports explicit root management flows (add, update, deactivate/reactivate, remove, refresh, select).
- App now runs an indexing baseline flow against persisted roots and stores candidates in `indexed_media`.
- Ignore rules are now centrally evaluated through the indexer engine.
- Indexing now applies incremental change detection (`new` / `changed` / `unchanged`) and marks missing files without destructive deletion.
- App can run storage read-model queries over indexed media and tags for verification-focused UI output.
- Search queries now run through the dedicated search subsystem instead of UI-owned matching logic.

### Docs
- Established baseline documentation for dependencies, troubleshooting, architecture, and repository map.
- Recorded dependency decisions for `serde`, `toml`, `directories`, and `rusqlite`.
- Recorded dependency decisions for `globset` and `walkdir`.
- Recorded dependency decisions for `imagesize`.
- Recorded dependency decisions for `strsim`.
