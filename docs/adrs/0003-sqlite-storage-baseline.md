# ADR 0003: SQLite storage and migration baseline

## Status

Accepted

## Context

Librapix requires a local persistence layer for source roots, settings, and ignore rules before indexing/search features are built. The system needs deterministic migrations and strict isolation from UI code.

## Decision

- Use SQLite via `rusqlite` in `librapix-storage`.
- Enable `rusqlite` `bundled` feature for consistent cross-platform behavior.
- Introduce explicit migration tracking through `schema_migrations`.
- Start with one baseline migration containing:
  - `source_roots`
  - `app_settings`
  - `ignore_rules`
- Keep schema minimal; avoid speculative indexing/media tables in this phase.

## Alternatives considered

- `sqlx`: stronger async/query features, more complexity than needed for local embedded baseline.
- `diesel`: strong typing and migrations, heavier setup for current scope.
- Deferring migrations: unacceptable risk for long-term schema evolution.

## Consequences

- Persistence evolves through controlled SQL migrations.
- Startup can initialize DB deterministically.
- Storage remains framework-agnostic and separated from Iced/presentation code.
