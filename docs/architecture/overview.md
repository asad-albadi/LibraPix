# Architecture Overview

Librapix is a desktop-first Rust application with strict internal boundaries.

## Current baseline

- UI runtime: Iced (`librapix-app`).
- Config subsystem: `librapix-config`.
- Domain/app primitives: `librapix-core`.
- Indexing subsystem: `librapix-indexer`.
- Search subsystem: `librapix-search`.
- Projection subsystem: `librapix-projections`.
- Thumbnail subsystem: `librapix-thumbnails`.
- Media-actions flow: app-orchestrated open/copy workflows over storage read models.
- UI shell system: app-side shell regions and reusable design tokens in `librapix-app`.
- Localization subsystem: `librapix-i18n`.
- Storage subsystem: `librapix-storage` (SQLite + migrations baseline).
- Read-model query surface: `librapix-storage` read APIs over indexed media and tags.
- Workspace orchestration: root virtual workspace.

## Architectural intent

- Keep Iced-specific types in `librapix-app`.
- Keep domain models and non-destructive rules in `librapix-core`.
- Keep user-facing text key-based and locale-resolved in `librapix-i18n`.
- Keep config path and settings behavior centralized in `librapix-config`.
- Keep storage/indexing/search as dedicated subsystems, not ad-hoc modules.
- Keep library root operations in application orchestration, not widget internals.

## Non-destructive stance

Source media is always read-only from application behavior.
All organizational metadata belongs to Librapix-managed storage.
