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

## Baseline implementation

- Ignore evaluation is centralized in `librapix-indexer::IgnoreEngine`.
- Rule matching is glob-based and compiled as a set for efficient scan-time checks.
- Baseline enabled global rules are seeded in storage:
  - `**/thumbnails/**`
  - `**/cache/**`
  - `**/*.tmp`
- Indexing consumes enabled global rules from storage and applies them consistently across scanned roots.
- Incremental runs use the same centralized ignore engine, so ignored paths are excluded consistently from both discovery and metadata refresh passes.

Library-specific overrides are reserved for a follow-up phase.
