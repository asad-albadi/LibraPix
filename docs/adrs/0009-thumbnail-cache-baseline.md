# ADR 0009: Thumbnail cache baseline

## Status

Accepted

## Context

Librapix MVP requires thumbnail-backed browsing while preserving the non-destructive guarantee.

## Decision

- Introduce `librapix-thumbnails` as a dedicated subsystem.
- Use deterministic thumbnail file names based on:
  - absolute source path
  - file size
  - modified timestamp
- Store thumbnails in app-owned cache directory (`thumbnails`).
- Generate thumbnails for image media only in this baseline.
- Treat generation failures as per-item status issues, not full indexing failures.

## Alternatives considered

- Persist thumbnails in source directories: rejected (non-destructive violation).
- Store thumbnail blobs in SQLite: deferred to avoid premature storage complexity.
- Include video thumbnails now: deferred until a clean documented pipeline is selected.

## Consequences

- Gallery/timeline flows can use stable cached image previews.
- Cache can be rebuilt safely from indexed metadata.
- Video thumbnail support remains an explicit future step.
