# Librapix

Librapix is a cross-platform, desktop-first, non-destructive local media gallery/manager for screenshots and recordings.

## Status

Project phase: MVP complete (technical + visual shell baseline).

## Core Principles

- Non-destructive by design: source media is treated as read-only.
- Documentation-driven: architecture and repository rules are first-class deliverables.
- Clear boundaries: UI, application flow, domain logic, storage/indexing/search, i18n, and config remain isolated.
- Simplicity first: small modules, explicit state transitions, and maintainable code.

## Current Workspace Layout

- `crates/librapix-app`: Iced desktop executable (presentation + app bootstrap).
- `crates/librapix-config`: typed config models, path strategy, TOML loading/saving, and validation.
- `crates/librapix-core`: domain and application orchestration primitives.
- `crates/librapix-indexer`: indexing pipeline foundation and centralized ignore matching.
- `crates/librapix-i18n`: key-based localization layer with locale fallback behavior.
- `crates/librapix-projections`: timeline and gallery read projection subsystem.
- `crates/librapix-search`: replaceable search contracts and fuzzy strategy baseline.
- `crates/librapix-storage`: SQLite storage and migrations subsystem.
- `crates/librapix-thumbnails`: app-owned image thumbnail cache subsystem.
- `docs/`: architecture, roadmap, dependency records, and repository operational docs.

## MSRV

- Minimum Supported Rust Version (MSRV): `1.85`.
- Reason: Rust 2024 edition is used and stabilized in Rust 1.85.0.

## Development Commands

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p librapix-app`

## Foundation behavior

- Config is stored as TOML in the platform config directory (`config.toml`).
- SQLite database is stored in the platform data directory (`librapix.db`).
- Thumbnails/cache remain app-owned data under the platform cache directory.
- Startup bootstrap syncs configured library roots into storage via idempotent upsert.
- Root lifecycle state is reconciled as `active`, `unavailable`, or `deactivated`.
- Indexing baseline scans eligible roots, applies centralized ignore rules, and persists indexed media candidates.
- Metadata baseline stores file size, modified time, media kind, and image dimensions where available.
- Re-index runs apply incremental change detection and mark missing indexed files without destructive source operations.
- Search-facing read-model queries are available over indexed media and tag joins.
- Indexing/galleries now generate and reuse deterministic image thumbnails in app-owned cache.
- UI uses a desktop shell layout (header/sidebar/main/details) with media-first gallery/timeline browsing.
- Timeline mode includes a projection-driven fast date scrubber with stable anchor-index mapping for smooth large-library navigation.
- Timeline day grouping uses local timezone day boundaries from `modified_unix_seconds`.
- Browse filters include media type, extension, and tag chips (applied consistently across gallery/timeline/search).
- Search results are no longer silently capped at 20.
- Media cards show compact metadata (`kind · size · dimensions`) and corner media-type badges.
- Details/actions support both copy-file and copy-path workflows, including keyboard shortcuts (`Cmd/Ctrl+C` for file, `Cmd/Ctrl+Shift+C` for path).
- Details action layout is responsive in narrow panels (no clipped final action button).
- Media header stats show `Shown`, `Images`, and `Videos` counts derived from the currently active browse/search result set.
- Live filesystem refresh can surface an in-app modal new-file dialog with preview, metadata, and quick actions.
- Windows publisher/signing baseline is documented under `packaging/windows/`.

## MVP Usage Flow

1. Add one or more library roots.
2. Configure ignore rules as needed.
3. Run indexing/reindexing.
4. Browse gallery or timeline and select media directly.
5. Inspect details, attach app/game tags, run search, and use open/copy actions.

## Documentation Index

See `docs/README.md` for the full documentation map.
