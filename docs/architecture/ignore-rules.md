# Ignore Rules Architecture

Ignore rules are a first-class, centralized subsystem.

## Rule goals

- Gitignore-like glob semantics for exclusions.
- Consistent use across indexing and scanning.
- Testable precedence and matching behavior.
- Stored as Librapix app data, not in source media metadata.

## Baseline examples

- `**/thumbnails/**`
- `**/cache/**`
- `**/*.tmp`

## Planned precedence

1. Built-in safety defaults
2. User-defined global rules
3. Library-specific overrides

Precedence details will be finalized with implementation tests.
