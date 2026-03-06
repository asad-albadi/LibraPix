# ADR 0005: Indexing and ignore engine baseline

## Status

Accepted

## Context

Librapix needs an initial indexing foundation that can scan persisted roots, apply ignore rules consistently, and persist minimal candidate records without overbuilding metadata extraction.

## Decision

- Introduce `librapix-indexer` as a dedicated crate.
- Use `walkdir` for recursive traversal.
- Use `globset` for centralized ignore-rule matching.
- Scan only `active` source roots from storage.
- Persist baseline candidates to `indexed_media` with:
  - `source_root_id`
  - `absolute_path`
  - `media_kind` (`image`/`video`)
- Keep media detection extension-based in this phase.

## Alternatives considered

- Embedding indexing in app crate: violates boundary clarity.
- Skipping dedicated ignore engine: increases coupling and drift risk.
- Full metadata extraction now: too broad for foundation scope.

## Consequences

- End-to-end indexing flow is testable with clear subsystem boundaries.
- Ignore behavior remains centralized and reusable.
- Future metadata enrichment can extend candidate records without replacing the foundation.
