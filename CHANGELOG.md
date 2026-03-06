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

### Changed
- Migrated from a single-crate starter to a multi-crate workspace.
- Declared MSRV `1.85` in workspace metadata and repository docs.
- App bootstrap now loads persisted config, opens storage, and syncs configured source roots.

### Docs
- Established baseline documentation for dependencies, troubleshooting, architecture, and repository map.
- Recorded dependency decisions for `serde`, `toml`, `directories`, and `rusqlite`.
