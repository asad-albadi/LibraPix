# Librapix

Librapix is a cross-platform, desktop-first, non-destructive local media gallery/manager for screenshots and recordings.

## Status

Project phase: metadata and incremental indexing foundation.

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
- `crates/librapix-storage`: SQLite storage and migrations subsystem.
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

## Documentation Index

See `docs/README.md` for the full documentation map.
