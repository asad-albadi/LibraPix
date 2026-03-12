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
- Video tools subsystem: `librapix-video-tools` (Make Short ffprobe/ffmpeg orchestration).
- Media-actions flow: app-orchestrated open/copy workflows (file + path) over storage read models.
- UI shell system: app-side shell regions, Fluent-inspired design tokens, and reusable component styles in `librapix-app`.
- Live announcement UX: in-app modal new-file dialog driven by filesystem-index deltas.
- Keyboard input flow: ignored-key subscription route for copy shortcuts integrated in app update logic.
- Background execution model: indexed work and projection/search refresh both run through Iced `Task::perform` modes to keep UI responsive on large libraries.
- Release update flow: non-blocking GitHub latest-release checks with explicit header chip state and manual/periodic scheduling policy.
- Localization subsystem: `librapix-i18n`.
- Storage subsystem: `librapix-storage` (SQLite + migrations baseline).
- Read-model query surface: `librapix-storage` read APIs over indexed media and tags.
- Workspace orchestration: root virtual workspace.

## Current branch focus

- `feat/catalog-first-architecture` introduces a catalog-first migration path documented in `docs/architecture/catalog-first-architecture.md`.
- The immediate goal is to separate source facts, normalized browse/search facts, and derived artifact facts without a destructive rewrite.

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
