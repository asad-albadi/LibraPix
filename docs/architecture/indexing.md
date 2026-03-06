# Indexing Architecture

Indexing is a dedicated subsystem (`librapix-indexer`) isolated from UI rendering.

## Baseline decisions

- Indexing reads source media metadata in read-only mode.
- Ignore rules are applied before metadata extraction.
- Index data is stored in Librapix-managed storage only.
- Indexing events are consumed by search and presentation layers through explicit application flow.
- Missing source files are expected operationally and must be handled as state transitions, not destructive actions.

## Baseline components

- Source root selection from storage (`active` lifecycle roots only)
- Ignore matcher via centralized `IgnoreEngine` and glob rules
- Size-based exclusion via `ScanOptions.min_file_size_bytes` (skips files below threshold)
- Filesystem traversal with recursive walk
- Media-kind detection by supported extension set
- Candidate writer to app-managed `indexed_media` table
- Missing-root reconciliation delegated to storage lifecycle updates
- Metadata extraction stage:
  - file size and modified timestamp
  - image dimensions when available
  - extraction status (`ok` / `partial` / `unreadable`)
- Incremental strategy:
  - compare by `absolute_path + file_size_bytes + modified_unix_seconds`
  - classify `new`, `changed`, `unchanged`
  - mark non-seen files under scanned roots as `missing`

## Baseline pipeline

1. Reconcile source-root availability.
2. Load eligible roots.
3. Load enabled ignore rules.
4. Scan filesystem and filter ignored entries.
5. Exclude files below configured minimum size (if set).
6. Detect incremental change class (`new` / `changed` / `unchanged`).
6. Extract baseline metadata for new/changed entries.
7. Persist/upsert candidates and mark missing files for scanned roots.
8. Query read-model rows for verification or downstream browsing/search surfaces.

No indexing logic should be embedded inside view widgets.
