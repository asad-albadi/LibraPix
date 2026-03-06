# ADR 0001: Workspace and crate boundaries

## Status

Accepted

## Context

The project started as a single `main.rs` Iced starter. Librapix requires long-term maintainability, strict separation of concerns, and documentation-driven architecture.

## Decision

Adopt a Rust workspace with separate crates:

- `librapix-app`: executable and presentation/runtime wiring
- `librapix-core`: domain and application orchestration primitives
- `librapix-i18n`: localization keys and locale resolution

Dependency direction:

- `app` depends on `core` and `i18n`
- `core` does not depend on UI framework crates
- `i18n` remains framework-agnostic

## Alternatives considered

- Single crate with modules: simpler short-term, but weaker boundary enforcement.
- More granular crates immediately: stronger isolation but unnecessary complexity this early.

## Consequences

- Improved architectural clarity and future scalability.
- Slightly more setup overhead for small changes.
- Easier to enforce non-destructive and subsystem boundaries as storage/indexing/search are added.
