# ADR 0002: Config format and path policy

## Status

Accepted

## Context

Librapix needs a stable configuration layer for locale/preferences, source roots, and future path overrides. The policy must support non-destructive behavior and removable/offline media roots.

## Decision

- Use TOML config (`config.toml`) with typed Serde models in `librapix-config`.
- Resolve default config/data/cache directories with `directories::ProjectDirs`.
- Normalize paths lexically and convert relative paths to absolute paths.
- Do not require path existence during config load/save.
- Keep schema versioning explicit (`schema_version = 1` baseline).

## Alternatives considered

- JSON: functional but less ergonomic for human-edited settings.
- Strict canonicalization at config-load time: would fail for offline/removable roots.
- Unstructured key/value config: weaker validation and evolution safety.

## Consequences

- Predictable cross-platform location strategy.
- Clear config validation with low operational friction.
- Indexing subsystem must perform existence checks at runtime.
