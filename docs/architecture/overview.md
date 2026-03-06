# Architecture Overview

Librapix is a desktop-first Rust application with strict internal boundaries.

## Current baseline

- UI runtime: Iced (`librapix-app`).
- Domain/app primitives: `librapix-core`.
- Localization subsystem: `librapix-i18n`.
- Workspace orchestration: root virtual workspace.

## Architectural intent

- Keep Iced-specific types in `librapix-app`.
- Keep domain models and non-destructive rules in `librapix-core`.
- Keep user-facing text key-based and locale-resolved in `librapix-i18n`.
- Add storage/indexing/search as dedicated subsystems, not ad-hoc modules.

## Non-destructive stance

Source media is always read-only from application behavior.
All organizational metadata belongs to Librapix-managed storage.
