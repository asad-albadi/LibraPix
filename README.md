# Librapix

Librapix is a cross-platform, desktop-first, non-destructive local media gallery/manager for screenshots and recordings.

## Status

Project phase: architecture baseline and workspace bootstrap.

## Core Principles

- Non-destructive by design: source media is treated as read-only.
- Documentation-driven: architecture and repository rules are first-class deliverables.
- Clear boundaries: UI, application flow, domain logic, storage/indexing/search, i18n, and config remain isolated.
- Simplicity first: small modules, explicit state transitions, and maintainable code.

## Current Workspace Layout

- `crates/librapix-app`: Iced desktop executable (presentation + app bootstrap).
- `crates/librapix-core`: domain and application orchestration primitives.
- `crates/librapix-i18n`: key-based localization layer with locale fallback behavior.
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

## Documentation Index

See `docs/README.md` for the full documentation map.
