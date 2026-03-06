# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

### Added
- Introduced a workspace layout with `librapix-app`, `librapix-core`, and `librapix-i18n`.
- Added initial architecture documentation set under `docs/architecture/`.
- Added roadmap documentation for MVP and future phases.
- Added ADR `0001` covering workspace boundaries and dependency direction.
- Added an i18n-ready app shell where user-facing text is key-based.

### Changed
- Migrated from a single-crate starter to a multi-crate workspace.
- Declared MSRV `1.85` in workspace metadata and repository docs.

### Docs
- Established baseline documentation for dependencies, troubleshooting, architecture, and repository map.
