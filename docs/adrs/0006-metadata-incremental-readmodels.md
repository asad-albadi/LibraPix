# ADR 0006: Metadata extraction and incremental indexing baseline

## Status

Accepted

## Context

Librapix needs to evolve from file discovery into a repeatable media index with usable metadata and a read model that future gallery/timeline/search layers can consume.

## Decision

- Extend indexing outputs with metadata baseline:
  - `file_size_bytes`
  - `modified_unix_seconds`
  - image dimensions (`width_px`, `height_px`) when available
  - `metadata_status` (`ok`, `partial`, `unreadable`)
- Implement incremental change detection using:
  - `absolute_path`
  - `file_size_bytes`
  - `modified_unix_seconds`
- Classify observed files as `new`, `changed`, or `unchanged`.
- Mark non-seen files in scanned roots as `missing` in app-managed storage.
- Add tag-readiness and read-model query baseline:
  - `tags` / `media_tags` tables
  - storage read APIs over indexed media + tags.

## Alternatives considered

- Full metadata extraction now (EXIF/video codecs/duration): deferred to keep baseline simple.
- Watcher-first incremental system: deferred; repeatable scan-based baseline chosen first.
- Search logic in UI layer: rejected due to layering constraints.

## Consequences

- Re-index runs are repeatable and non-destructive.
- Metadata becomes queryable for future gallery/timeline/search features.
- Architecture keeps indexing/search read models decoupled from Iced presentation code.
